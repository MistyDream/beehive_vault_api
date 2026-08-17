use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::pagination::PaginationError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
}

impl From<PaginationError> for ApiError {
    fn from(error: PaginationError) -> Self {
        Self::BadRequest(error.to_string())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::Database(error) => {
                tracing::error!(%error, "database operation failed");
                let database_code = error
                    .as_database_error()
                    .and_then(|database_error| database_error.code())
                    .map(|code| code.into_owned());
                match database_code.as_deref() {
                    Some("23505") => (
                        StatusCode::CONFLICT,
                        "conflict",
                        "A resource with the same unique values already exists".to_owned(),
                    ),
                    Some("23503") => (
                        StatusCode::BAD_REQUEST,
                        "bad_request",
                        "A referenced resource does not exist".to_owned(),
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "internal_error",
                        "An internal error occurred".to_owned(),
                    ),
                }
            }
        };

        (status, Json(ErrorResponse { code, message })).into_response()
    }
}

pub fn required_text(value: String, field: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return Err(ApiError::BadRequest(format!(
            "{field} must contain between 1 and 100 characters"
        )));
    }
    Ok(trimmed.to_owned())
}
