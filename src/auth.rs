//! Authentication flow for Alarm.com systems.
//!
//! Uses the standard www.alarm.com login portal with ASP.NET ViewState tokens,
//! which is compatible with all Alarm.com-powered providers.
//!
//! The login process:
//! 1. GET the login page to extract ViewState hidden fields
//! 2. POST credentials with ViewState tokens
//! 3. Follow redirects to establish the session
//! 4. Extract the `afg` anti-forgery cookie for subsequent API requests
//!
//! For accounts with 2FA enabled, a trusted device cookie (`twoFactorAuthenticationId`)
//! must be provided to bypass 2FA. This cookie can be obtained from your browser's
//! cookies for `www.alarm.com` after completing 2FA manually.

use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{debug, info};

use crate::error::{AlarmError, Result};

pub const URL_BASE: &str = "https://www.alarm.com/";
const LOGIN_PAGE_URL: &str = "https://www.alarm.com/login";
const LOGIN_POST_URL: &str = "https://www.alarm.com/web/Default.aspx";

/// ASP.NET WebForms hidden field names needed for login.
const VIEWSTATE: &str = "__VIEWSTATE";
const VIEWSTATE_GENERATOR: &str = "__VIEWSTATEGENERATOR";
const PREVIOUS_PAGE: &str = "__PREVIOUSPAGE";
const EVENT_VALIDATION: &str = "__EVENTVALIDATION";

/// Hidden form fields extracted from the ASP.NET login page.
pub struct LoginFormFields {
    viewstate: String,
    viewstate_generator: String,
    previous_page: String,
    event_validation: String,
}

/// Fetch the login page and extract ViewState tokens.
pub async fn fetch_login_page(client: &Client) -> Result<LoginFormFields> {
    info!("Fetching login page: {LOGIN_PAGE_URL}");

    let resp = client
        .get(LOGIN_PAGE_URL)
        .header("User-Agent", user_agent())
        .send()
        .await?;

    let body = resp.text().await?;
    let doc = Html::parse_document(&body);

    let viewstate = extract_hidden_field(&doc, VIEWSTATE)
        .ok_or_else(|| AlarmError::HtmlParse("missing __VIEWSTATE".to_string()))?;
    let viewstate_generator = extract_hidden_field(&doc, VIEWSTATE_GENERATOR).unwrap_or_default();
    let previous_page = extract_hidden_field(&doc, PREVIOUS_PAGE).unwrap_or_default();
    let event_validation = extract_hidden_field(&doc, EVENT_VALIDATION).unwrap_or_default();

    debug!("Extracted ViewState ({} chars)", viewstate.len());

    Ok(LoginFormFields {
        viewstate,
        viewstate_generator,
        previous_page,
        event_validation,
    })
}

/// Submit credentials via the ASP.NET WebForms login page.
///
/// Returns the final response (URL + body) to check for login failure / 2FA indicators.
pub async fn submit_credentials(
    client: &Client,
    username: &str,
    password: &str,
    form_fields: &LoginFormFields,
) -> Result<(String, String)> {
    info!("Submitting credentials to: {LOGIN_POST_URL}");

    let form = [
        ("__EVENTTARGET", ""),
        ("__EVENTARGUMENT", ""),
        ("__VIEWSTATEENCRYPTED", ""),
        (VIEWSTATE, form_fields.viewstate.as_str()),
        (
            VIEWSTATE_GENERATOR,
            form_fields.viewstate_generator.as_str(),
        ),
        (PREVIOUS_PAGE, form_fields.previous_page.as_str()),
        (EVENT_VALIDATION, form_fields.event_validation.as_str()),
        ("IsFromNewSite", "1"),
        ("JavaScriptTest", "1"),
        (
            "ctl00$ContentPlaceHolder1$loginform$txtUserName",
            username,
        ),
        ("txtPassword", password),
    ];

    let resp = client
        .post(LOGIN_POST_URL)
        .header("User-Agent", user_agent())
        .header("Referer", LOGIN_PAGE_URL)
        .header("Connection", "keep-alive")
        .form(&form)
        .send()
        .await?;

    let final_url = resp.url().to_string();
    let body = resp.text().await?;
    debug!("Login redirect URL: {final_url}");

    // Check for login failure indicators
    if final_url.contains("m=login_fail") || final_url.contains("err=login_fail") {
        return Err(AlarmError::AuthenticationFailed(
            "invalid username or password".to_string(),
        ));
    }
    if final_url.contains("m=LockedOut") || final_url.contains("err=locked") {
        return Err(AlarmError::AccountLockedOut);
    }

    Ok((final_url, body))
}

/// Extract a hidden form field value from an HTML document.
fn extract_hidden_field(doc: &Html, name: &str) -> Option<String> {
    let selector = Selector::parse(&format!(r#"input[name="{name}"]"#)).ok()?;
    doc.select(&selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .map(|v| v.to_string())
}

/// Extract the AFG anti-forgery token from an HTML page body.
///
/// The Alarm.com portal sets the `afg` cookie via client-side JavaScript
/// (e.g. `$.cookie('afg','TOKEN',{...})`), so it never appears as a
/// `Set-Cookie` header.
pub fn extract_afg_from_html(html: &str) -> Option<String> {
    let re = Regex::new(r#"\$\.cookie\(\s*['"]afg['"]\s*,\s*['"]([^'"]+)['"]\s*"#).ok()?;
    if let Some(caps) = re.captures(html) {
        return caps.get(1).map(|m| m.as_str().to_string());
    }

    let doc = Html::parse_document(html);
    let selector = Selector::parse(r#"input[name="afg"]"#).ok()?;
    if let Some(el) = doc.select(&selector).next() {
        if let Some(val) = el.value().attr("value") {
            return Some(val.to_string());
        }
    }

    None
}

pub(crate) fn user_agent() -> &'static str {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36"
}
