/// Errors that can occur when interacting with the Alarm.com API.
#[derive(Debug, thiserror::Error)]
pub enum AlarmError {
    /// Login credentials were rejected or the session is invalid.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Two-factor authentication is required but no trusted device cookie was provided.
    ///
    /// Set the `twoFactorAuthenticationId` cookie via
    /// [`AlarmDotCom::set_trusted_device_cookie`](crate::AlarmDotCom::set_trusted_device_cookie)
    /// to bypass 2FA. You can obtain this cookie from your browser after completing
    /// 2FA manually on www.alarm.com.
    #[error("two-factor authentication required (set ALARM_MFA_COOKIE to bypass)")]
    TwoFactorRequired,

    /// The session has expired and a re-login is needed.
    #[error("session expired")]
    SessionExpired,

    /// The Alarm.com service returned a server error or was unreachable.
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    /// The account does not have permission for the requested operation.
    #[error("not authorized: {0}")]
    NotAuthorized(String),

    /// The API returned a response that could not be parsed.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    /// The requested device was not found.
    #[error("unknown device: {0}")]
    UnknownDevice(String),

    /// The requested operation is not supported for this device.
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// A network-level error occurred.
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// An error occurred while parsing HTML.
    #[error("html parse error: {0}")]
    HtmlParse(String),

    /// An error occurred while parsing JSON.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// The account is locked out.
    #[error("account locked out")]
    AccountLockedOut,
}

pub type Result<T> = std::result::Result<T, AlarmError>;
