use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Categorization of application errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum AppErrorKind {
    NotFound,
    BadRequest,
    ValidationError,
    Conflict,
    DatabaseError,
    Unauthorized,
    Forbidden,
    RateLimited,
    InternalError,
}

impl fmt::Display for AppErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppErrorKind::NotFound => write!(f, "NotFound"),
            AppErrorKind::BadRequest => write!(f, "BadRequest"),
            AppErrorKind::ValidationError => write!(f, "ValidationError"),
            AppErrorKind::Conflict => write!(f, "Conflict"),
            AppErrorKind::DatabaseError => write!(f, "DatabaseError"),
            AppErrorKind::Unauthorized => write!(f, "Unauthorized"),
            AppErrorKind::Forbidden => write!(f, "Forbidden"),
            AppErrorKind::RateLimited => write!(f, "RateLimited"),
            AppErrorKind::InternalError => write!(f, "InternalError"),
        }
    }
}

/// Structured application error used across server and client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub field_errors: HashMap<String, String>,
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::NotFound,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::BadRequest,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::RateLimited,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn validation(message: impl Into<String>, field_errors: HashMap<String, String>) -> Self {
        Self {
            kind: AppErrorKind::ValidationError,
            message: message.into(),
            field_errors,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::Conflict,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::DatabaseError,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::Unauthorized,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::Forbidden,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            kind: AppErrorKind::InternalError,
            message: message.into(),
            field_errors: HashMap::new(),
        }
    }

    /// Parse an AppError from a ServerFnError message string (client-side).
    ///
    /// `ServerFnError::to_string()` wraps the payload like:
    ///   `error running server function: {"kind":"Unauthorized",...} (details: None)`
    /// This method extracts the embedded JSON and parses it.
    pub fn from_server_error(error_message: &str) -> Option<Self> {
        // Try direct parse first (in case the string is raw JSON)
        if let Ok(err) = serde_json::from_str::<Self>(error_message) {
            return Some(err);
        }
        // Extract the JSON object embedded between the first `{` and last `}`
        let start = error_message.find('{')?;
        let end = error_message.rfind('}')?;
        if end > start {
            serde_json::from_str(&error_message[start..=end]).ok()
        } else {
            None
        }
    }

    /// Extract per-field validation errors from a `ServerFnError.to_string()`.
    ///
    /// Parses the embedded `AppError` JSON and returns its `field_errors` map.
    /// Returns an empty map if parsing fails or no field errors exist.
    pub fn parse_field_errors(error_string: &str) -> HashMap<String, String> {
        Self::from_server_error(error_string)
            .map(|e| e.field_errors)
            .unwrap_or_default()
    }

    /// Extract a user-friendly error message from a `ServerFnError.to_string()`.
    ///
    /// Parses the embedded `AppError` JSON and returns its `message` field.
    /// Falls back to a generic message if parsing fails.
    pub fn friendly_message(error_string: &str) -> String {
        if let Some(app_error) = Self::from_server_error(error_string) {
            app_error.message
        } else {
            "Something went wrong. Please try again.".to_string()
        }
    }

    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    fn status_code_u16(&self) -> u16 {
        match self.kind {
            AppErrorKind::NotFound => 404,
            AppErrorKind::BadRequest => 400,
            AppErrorKind::ValidationError => 422,
            AppErrorKind::Conflict => 409,
            AppErrorKind::DatabaseError => 500,
            AppErrorKind::Unauthorized => 401,
            AppErrorKind::Forbidden => 403,
            AppErrorKind::RateLimited => 429,
            AppErrorKind::InternalError => 500,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for AppError {}

#[cfg(feature = "validation")]
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut field_errors = HashMap::new();
        for (field, errs) in errors.field_errors() {
            if let Some(first) = errs.first() {
                let msg = first
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Invalid value for {}", field));
                field_errors.insert(field.to_string(), msg);
            }
        }
        AppError::validation("Validation failed", field_errors)
    }
}

#[cfg(feature = "server")]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = axum::http::StatusCode::from_u16(self.status_code_u16())
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        (status, axum::Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_server_error_parses_raw_json() {
        let json = r#"{"kind":"Unauthorized","message":"Invalid token"}"#;
        let err = AppError::from_server_error(json).unwrap();
        assert_eq!(err.kind, AppErrorKind::Unauthorized);
        assert_eq!(err.message, "Invalid token");
    }

    #[test]
    fn from_server_error_parses_wrapped_json() {
        let wrapped = r#"error running server function: {"kind":"NotFound","message":"User not found"} (details: None)"#;
        let err = AppError::from_server_error(wrapped).unwrap();
        assert_eq!(err.kind, AppErrorKind::NotFound);
        assert_eq!(err.message, "User not found");
    }

    #[test]
    fn from_server_error_returns_none_for_garbage() {
        assert!(AppError::from_server_error("not json at all").is_none());
        assert!(AppError::from_server_error("").is_none());
    }

    #[test]
    fn friendly_message_extracts_message_field() {
        let json = r#"{"kind":"Forbidden","message":"Premium required"}"#;
        assert_eq!(AppError::friendly_message(json), "Premium required");
    }

    #[test]
    fn friendly_message_fallback_for_unparseable() {
        assert_eq!(
            AppError::friendly_message("garbage input"),
            "Something went wrong. Please try again."
        );
    }

    #[test]
    fn not_found_error_has_correct_kind() {
        let err = AppError::not_found("missing item");
        assert_eq!(err.kind, AppErrorKind::NotFound);
        assert_eq!(err.message, "missing item");
        assert!(err.field_errors.is_empty());
    }

    #[test]
    fn validation_error_includes_field_errors() {
        let mut fields = HashMap::new();
        fields.insert("email".to_string(), "invalid format".to_string());
        let err = AppError::validation("Validation failed", fields);
        assert_eq!(err.kind, AppErrorKind::ValidationError);
        assert_eq!(err.field_errors.get("email").unwrap(), "invalid format");
    }

    #[test]
    fn status_code_mapping() {
        assert_eq!(AppError::not_found("").status_code_u16(), 404);
        assert_eq!(
            AppError::validation("", HashMap::new()).status_code_u16(),
            422
        );
        assert_eq!(AppError::database("").status_code_u16(), 500);
        assert_eq!(AppError::unauthorized("").status_code_u16(), 401);
        assert_eq!(AppError::forbidden("").status_code_u16(), 403);
        assert_eq!(AppError::internal("").status_code_u16(), 500);
    }

    #[test]
    fn display_impl_formats_correctly() {
        let err = AppError::unauthorized("bad credentials");
        assert_eq!(format!("{}", err), "Unauthorized: bad credentials");
    }

    #[test]
    fn error_roundtrip_through_json() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), "too short".to_string());
        let err = AppError::validation("Validation failed", fields);
        let json = serde_json::to_string(&err).unwrap();
        let parsed: AppError = serde_json::from_str(&json).unwrap();
        assert_eq!(err, parsed);
    }
}
