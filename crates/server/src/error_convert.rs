use dioxus::prelude::ServerFnError;
use shared_types::AppError;

/// Convert a sqlx::Error into an AppError.
pub fn sqlx_to_app_error(err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::RowNotFound => AppError::not_found("Resource not found"),
        sqlx::Error::Database(db_err) => {
            // PostgreSQL unique constraint violation (error code 23505)
            if db_err.code().as_deref() == Some("23505") {
                let detail = db_err.message();
                let friendly = if detail.contains("email") {
                    "An account with this email already exists"
                } else if detail.contains("username") {
                    "This username is already taken"
                } else if detail.contains("phone") {
                    "This phone number is already in use"
                } else {
                    "A record with this value already exists"
                };
                return AppError::conflict(friendly);
            }
            AppError::database(err.to_string())
        }
        _ => AppError::database(err.to_string()),
    }
}

/// Convert an AppError into a ServerFnError by serializing as JSON.
pub fn app_error_to_server_fn_error(err: AppError) -> ServerFnError {
    let json = serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone());
    ServerFnError::new(json)
}

/// Extension trait providing `.into_app_error()` on sqlx::Error.
pub trait SqlxErrorExt {
    fn into_app_error(self) -> AppError;
}

impl SqlxErrorExt for sqlx::Error {
    fn into_app_error(self) -> AppError {
        sqlx_to_app_error(self)
    }
}

/// Extension trait providing `.into_server_fn_error()` on AppError.
pub trait AppErrorExt {
    fn into_server_fn_error(self) -> ServerFnError;
}

impl AppErrorExt for AppError {
    fn into_server_fn_error(self) -> ServerFnError {
        app_error_to_server_fn_error(self)
    }
}

/// Trait for validating request DTOs before processing.
pub trait ValidateRequest {
    fn validate_request(&self) -> Result<(), AppError>;
}

impl<T: validator::Validate> ValidateRequest for T {
    fn validate_request(&self) -> Result<(), AppError> {
        self.validate().map_err(AppError::from)
    }
}
