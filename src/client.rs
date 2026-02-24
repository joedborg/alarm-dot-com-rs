//! HTTP client for the Alarm.com API.
//!
//! Manages the HTTP session (cookie jar), anti-forgery tokens, authentication,
//! and provides typed `get`/`post` helpers for the JSON:API endpoints.

use std::sync::Arc;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::header::HeaderValue;
use reqwest::{Client, Response};
use tracing::{debug, info, warn};

use crate::auth;
use crate::error::{AlarmError, Result};
use crate::models::device::ResourceType;
use crate::models::jsonapi::{ApiResponse, ErrorDocument};

const URL_BASE: &str = "https://www.alarm.com/";
const API_URL_BASE: &str = "https://www.alarm.com/web/api/";
const REQUEST_RETRY_LIMIT: u32 = 3;
const SUBMIT_RETRY_LIMIT: u32 = 2;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                           (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";

/// The main HTTP client for interacting with Alarm.com's API.
pub struct AlarmClient {
    client: Client,
    cookie_jar: Arc<Jar>,
    ajax_key: Option<String>,
    username: String,
    password: String,
    trusted_device_cookie: Option<String>,
    logged_in: bool,
}

impl AlarmClient {
    /// Create a new `AlarmClient` with the given credentials.
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        let cookie_jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .expect("failed to build HTTP client");

        AlarmClient {
            client,
            cookie_jar,
            ajax_key: None,
            username: username.into(),
            password: password.into(),
            trusted_device_cookie: None,
            logged_in: false,
        }
    }

    /// Whether the client has completed authentication.
    pub fn is_logged_in(&self) -> bool {
        self.logged_in
    }

    /// Set a trusted device cookie to bypass 2FA on login.
    ///
    /// This cookie value (`twoFactorAuthenticationId`) is set by Alarm.com
    /// after a successful 2FA completion. Providing it marks this client as
    /// a "trusted device" so 2FA is not required on subsequent logins.
    ///
    /// You can obtain this value from your browser's cookies for `www.alarm.com`
    /// (the `twoFactorAuthenticationId` cookie) after completing 2FA manually.
    pub fn set_trusted_device_cookie(&mut self, cookie: impl Into<String>) {
        self.trusted_device_cookie = Some(cookie.into());
    }

    /// Get the current trusted device cookie value, if available.
    pub fn trusted_device_cookie(&self) -> Option<&str> {
        self.trusted_device_cookie.as_deref()
    }

    /// Perform the full login flow.
    ///
    /// Returns `Ok(())` if login succeeds.
    /// Returns `Err(AlarmError::TwoFactorRequired)` if 2FA is needed and no
    /// trusted device cookie was provided.
    pub async fn login(&mut self) -> Result<()> {
        info!("Starting login flow");
        self.logged_in = false;
        self.ajax_key = None;

        // Pre-set the trusted device cookie if available (bypasses 2FA).
        if let Some(ref cookie) = self.trusted_device_cookie {
            let url = URL_BASE.parse::<url::Url>().unwrap();
            self.cookie_jar.add_cookie_str(
                &format!("twoFactorAuthenticationId={cookie}; Path=/"),
                &url,
            );
            info!("Pre-set trusted device cookie");
        }

        // Step 1: Fetch the login page and extract ViewState tokens.
        let form_fields = auth::fetch_login_page(&self.client).await?;

        // Step 2: Submit credentials.
        let (_redirect_url, redirect_body) = auth::submit_credentials(
            &self.client,
            &self.username,
            &self.password,
            &form_fields,
        )
        .await?;

        info!("Credentials accepted");

        // Read cookies set during the login redirect chain.
        self.refresh_cookies_from_jar();

        // Step 3: Extract the AFG anti-forgery token.
        if self.ajax_key.is_none() {
            if let Some(afg) = auth::extract_afg_from_html(&redirect_body) {
                debug!("Extracted AFG from login response HTML");
                self.ajax_key = Some(afg);
            }
        }

        if self.ajax_key.is_none() {
            debug!("AFG not found in login response, loading home page");
            let resp = self
                .client
                .get(format!("{URL_BASE}web/system/home"))
                .header("User-Agent", USER_AGENT)
                .send()
                .await?;

            self.update_afg_from_response(&resp);
            self.refresh_cookies_from_jar();

            if self.ajax_key.is_none() {
                let body = resp.text().await?;
                if let Some(afg) = auth::extract_afg_from_html(&body) {
                    debug!("Extracted AFG from home page HTML");
                    self.ajax_key = Some(afg);
                }
            }
        }

        if self.ajax_key.is_none() {
            warn!("Could not find AFG anti-forgery token");
        }

        // Step 4: Verify the session works by making a test API call.
        let verify_url = format!("{API_URL_BASE}systems/availableSystemItems");
        let verify_resp = self
            .client
            .get(&verify_url)
            .headers(self.build_headers(true))
            .send()
            .await?;

        if verify_resp.status().as_u16() == 409 {
            return Err(AlarmError::TwoFactorRequired);
        }

        if !verify_resp.status().is_success() {
            let status = verify_resp.status();
            let body = verify_resp.text().await.unwrap_or_default();
            return Err(AlarmError::AuthenticationFailed(format!(
                "session verification failed with status {status}: {body}"
            )));
        }

        self.logged_in = true;
        info!("Login complete");
        Ok(())
    }

    /// Fetch a resource from the API (GET).
    pub async fn get(
        &mut self,
        resource_type: ResourceType,
        id: Option<&str>,
    ) -> Result<ApiResponse> {
        let url = self.build_url(resource_type.api_path(), id, None);
        self.api_get(&url).await
    }

    /// Fetch a resource from a raw API path (GET).
    pub async fn get_path(&mut self, path: &str) -> Result<ApiResponse> {
        let url = format!("{API_URL_BASE}{path}");
        self.api_get(&url).await
    }

    /// Send a command to a device (POST).
    pub async fn post(
        &mut self,
        resource_type: ResourceType,
        id: &str,
        action: &str,
        body: serde_json::Value,
    ) -> Result<ApiResponse> {
        let url = self.build_url(resource_type.api_path(), Some(id), Some(action));
        self.api_post(&url, body).await
    }

    /// Send a keep-alive signal to prevent session timeout.
    pub async fn keep_alive(&mut self) -> Result<bool> {
        let url = format!("{URL_BASE}web/KeepAlive.aspx");

        let resp = self
            .client
            .post(&url)
            .headers(self.build_headers(true))
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if resp.status().as_u16() == 403 {
            self.logged_in = false;
            return Ok(false);
        }

        Ok(resp.status().is_success())
    }

    // ---- Internal helpers ----

    fn build_url(&self, path: &str, id: Option<&str>, action: Option<&str>) -> String {
        let mut url = format!("{API_URL_BASE}{path}");
        if let Some(id) = id {
            url.push('/');
            url.push_str(id);
        }
        if let Some(action) = action {
            url.push('/');
            url.push_str(action);
        }
        url
    }

    fn build_headers(&self, use_ajax_key: bool) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", USER_AGENT.parse().unwrap());
        headers.insert(
            "Referer",
            format!("{URL_BASE}web/system/home").parse().unwrap(),
        );
        headers.insert("Connection", "keep-alive".parse().unwrap());
        headers.insert("Accept", "application/vnd.api+json".parse().unwrap());

        if use_ajax_key {
            if let Some(ref key) = self.ajax_key {
                headers.insert("Ajaxrequestuniquekey", key.parse().unwrap());
            }
        }

        headers
    }

    async fn api_get(&mut self, url: &str) -> Result<ApiResponse> {
        let mut retries = 0u32;

        loop {
            debug!("GET {url} (attempt {})", retries + 1);

            let resp = self
                .client
                .get(url)
                .headers(self.build_headers(true))
                .send()
                .await?;

            self.update_afg_from_response(&resp);

            let status = resp.status();
            let body = resp.text().await?;

            if status.is_success() {
                return self.parse_response(&body);
            }

            if status.as_u16() == 401 || status.as_u16() == 403 {
                if retries == 0 {
                    info!("Got {status}, attempting session repair");
                    if self.try_relogin().await.is_ok() {
                        retries += 1;
                        continue;
                    }
                }
                return Err(AlarmError::SessionExpired);
            }

            if status.as_u16() == 409 {
                return Err(AlarmError::TwoFactorRequired);
            }

            if status.as_u16() == 406 || status.as_u16() == 423 {
                return Err(AlarmError::NotAuthorized(format!(
                    "GET {url} returned {status}"
                )));
            }

            if retries >= REQUEST_RETRY_LIMIT {
                return Err(AlarmError::ServiceUnavailable(format!(
                    "GET {url} failed after {REQUEST_RETRY_LIMIT} retries with status {status}"
                )));
            }

            retries += 1;
        }
    }

    async fn api_post(&mut self, url: &str, body: serde_json::Value) -> Result<ApiResponse> {
        let mut retries = 0u32;

        loop {
            debug!("POST {url} (attempt {})", retries + 1);

            let resp = self
                .client
                .post(url)
                .headers(self.build_headers(true))
                .header("Content-Type", "application/json; charset=UTF-8")
                .json(&body)
                .send()
                .await?;

            self.update_afg_from_response(&resp);

            let status = resp.status();
            let body_text = resp.text().await?;

            if status.is_success() {
                return self.parse_response(&body_text);
            }

            if status.as_u16() == 401 || status.as_u16() == 403 {
                if retries == 0 {
                    info!("Got {status}, attempting session repair");
                    if self.try_relogin().await.is_ok() {
                        retries += 1;
                        continue;
                    }
                }
                return Err(AlarmError::SessionExpired);
            }

            if status.as_u16() == 409 {
                return Err(AlarmError::TwoFactorRequired);
            }

            if status.as_u16() == 422 {
                warn!("POST {url} returned 422: {body_text}");
            }

            if retries >= SUBMIT_RETRY_LIMIT {
                return Err(AlarmError::ServiceUnavailable(format!(
                    "POST {url} failed after {SUBMIT_RETRY_LIMIT} retries with status {status}"
                )));
            }

            retries += 1;
        }
    }

    fn parse_response(&self, body: &str) -> Result<ApiResponse> {
        let response = match ApiResponse::parse(body) {
            Ok(r) => r,
            Err(e) => {
                let preview = if body.len() > 500 { &body[..500] } else { body };
                warn!("Failed to parse response ({} bytes): {preview}", body.len());
                return Err(e.into());
            }
        };

        if let ApiResponse::Error(ref err_doc) = response {
            return self.handle_error_document(err_doc);
        }

        Ok(response)
    }

    fn handle_error_document(&self, err_doc: &ErrorDocument) -> Result<ApiResponse> {
        let codes: Vec<u32> = err_doc
            .errors
            .iter()
            .filter_map(|e| e.code.as_ref()?.parse().ok())
            .collect();

        if codes.contains(&401) || codes.contains(&403) {
            return Err(AlarmError::SessionExpired);
        }
        if codes.contains(&406) || codes.contains(&423) {
            return Err(AlarmError::NotAuthorized(
                "server returned permission error".to_string(),
            ));
        }
        if codes.contains(&409) {
            return Err(AlarmError::TwoFactorRequired);
        }

        let detail = err_doc
            .errors
            .first()
            .and_then(|e| e.detail.clone())
            .unwrap_or_else(|| format!("error codes: {:?}", codes));

        Err(AlarmError::UnexpectedResponse(detail))
    }

    fn refresh_cookies_from_jar(&mut self) {
        let url = format!("{URL_BASE}web/").parse::<url::Url>().unwrap();
        let cookies: Option<HeaderValue> = self.cookie_jar.cookies(&url);

        if let Some(cookie_header) = cookies {
            let cookie_str: &str = cookie_header.to_str().unwrap_or("");
            for part in cookie_str.split(';') {
                let part: &str = part.trim();
                if let Some(value) = part.strip_prefix("afg=") {
                    self.ajax_key = Some(value.to_string());
                    debug!("Extracted AFG cookie from jar");
                }
                if let Some(value) = part.strip_prefix("twoFactorAuthenticationId=") {
                    if !value.is_empty() {
                        self.trusted_device_cookie = Some(value.to_string());
                        debug!("Extracted trusted device cookie from jar");
                    }
                }
            }
        }
    }

    fn update_afg_from_response(&mut self, resp: &Response) {
        for cookie in resp.cookies() {
            if cookie.name() == "afg" {
                self.ajax_key = Some(cookie.value().to_string());
                debug!("Updated AFG key from response cookie");
            }
            if cookie.name() == "twoFactorAuthenticationId" {
                let value = cookie.value().to_string();
                if !value.is_empty() {
                    self.trusted_device_cookie = Some(value);
                    debug!("Updated trusted device cookie from response");
                }
            }
        }
    }

    async fn try_relogin(&mut self) -> Result<()> {
        info!("Attempting automatic re-login");
        self.logged_in = false;

        if let Some(ref cookie) = self.trusted_device_cookie {
            let url = URL_BASE.parse::<url::Url>().unwrap();
            self.cookie_jar.add_cookie_str(
                &format!("twoFactorAuthenticationId={cookie}; Path=/"),
                &url,
            );
        }

        let form_fields = auth::fetch_login_page(&self.client).await?;
        auth::submit_credentials(&self.client, &self.username, &self.password, &form_fields)
            .await?;
        self.refresh_cookies_from_jar();
        self.logged_in = true;

        info!("Re-login successful");
        Ok(())
    }
}
