use actix_web::{HttpRequest, HttpResponse, ResponseError, http::StatusCode};
use actix_web::error::{InternalError, JsonPayloadError, PathError};
use garde_actix_web::error::Error as GardeError;

use crate::application::error::AppError;
use crate::infrastructure::http::dto::response::error_response::{FieldError, ProblemDetail};
use crate::infrastructure::http::request_context::current_path;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    App(#[from] AppError),

    #[error("Validation failed")]
    Validation(Vec<FieldError>),

    #[error("Unsupported media type")]
    UnsupportedMediaType,

    #[error("Payload too large")]
    PayloadTooLarge(String),

    #[error("Unauthorized")]
    Unauthorized,
}

/// Map a path-extraction failure to a problem+json 400. Triggered when a
/// typed path param (`web::Path<Uuid>`, `web::Path<Isin>`) fails to
/// deserialize — wrong format, malformed UUID, etc. Without this, actix
/// returns a plain-text 400 that bypasses the API's error contract.
pub fn path_error_handler(err: PathError, req: &HttpRequest) -> actix_web::Error {
    let api_error: ApiError = AppError::BadRequest(err.to_string()).into();

    tracing::warn!(
        method = %req.method(),
        path = %req.path(),
        error = %api_error,
        "request rejected at path extraction"
    );

    let response = api_error.error_response();
    InternalError::from_response(api_error, response).into()
}

pub fn garde_error_handler(err: GardeError, req: &HttpRequest) -> actix_web::Error {
    let api_error = match err {
        GardeError::ValidationError(report) => {
            let fields: Vec<FieldError> = report
                .iter()
                .map(|(path, error)| FieldError {
                    field: path.to_string(),
                    message: error.to_string(),
                })
                .collect();
            ApiError::Validation(fields)
        }
        GardeError::JsonPayloadError(JsonPayloadError::ContentType) => ApiError::UnsupportedMediaType,
        GardeError::JsonPayloadError(
            e @ (JsonPayloadError::OverflowKnownLength { .. } | JsonPayloadError::Overflow { .. }),
        ) => ApiError::PayloadTooLarge(e.to_string()),
        GardeError::JsonPayloadError(e) => AppError::BadRequest(e.to_string()).into(),
        other => AppError::BadRequest(other.to_string()).into(),
    };

    tracing::warn!(
        method = %req.method(),
        path = %req.path(),
        error = %api_error,
        "request rejected at validation"
    );

    let response = api_error.error_response();
    InternalError::from_response(api_error, response).into()
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::App(err) => match err {
                AppError::NotFound => StatusCode::NOT_FOUND,
                AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
                AppError::Conflict(_) => StatusCode::CONFLICT,
                AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        let (problem_type, title, detail, errors) = match self {
            ApiError::App(AppError::NotFound) => (
                "/problems/not-found",
                "Not Found".to_string(),
                Some("The requested resource was not found".to_string()),
                None,
            ),
            ApiError::App(AppError::BadRequest(msg)) => (
                "/problems/bad-request",
                "Bad Request".to_string(),
                Some(msg.clone()),
                None,
            ),
            ApiError::App(AppError::Conflict(msg)) => (
                "/problems/conflict",
                "Conflict".to_string(),
                Some(msg.clone()),
                None,
            ),
            ApiError::App(AppError::Internal(err)) => {
                tracing::error!(error = %err, "internal server error");
                (
                    "/problems/internal",
                    "Internal Server Error".to_string(),
                    None,
                    None,
                )
            }
            ApiError::Validation(fields) => (
                "/problems/validation",
                "Validation Error".to_string(),
                Some("One or more fields are invalid".to_string()),
                Some(fields.clone()),
            ),
            ApiError::UnsupportedMediaType => (
                "/problems/unsupported-media-type",
                "Unsupported Media Type".to_string(),
                Some("Request Content-Type must be application/json".to_string()),
                None,
            ),
            ApiError::PayloadTooLarge(msg) => (
                "/problems/payload-too-large",
                "Payload Too Large".to_string(),
                Some(msg.clone()),
                None,
            ),
            ApiError::Unauthorized => (
                "/problems/unauthorized",
                "Unauthorized".to_string(),
                Some("Missing or invalid API key".to_string()),
                None,
            ),
        };

        let mut builder = HttpResponse::build(status);
        builder.content_type("application/problem+json");
        // RFC 9110 §15.5.2 / RFC 6750 §3: 401 MUST carry WWW-Authenticate.
        if matches!(self, ApiError::Unauthorized) {
            builder.insert_header(("WWW-Authenticate", "Bearer realm=\"api\""));
        }
        builder.json(ProblemDetail {
            problem_type: problem_type.to_string(),
            title,
            status: status.as_u16(),
            detail,
            instance: current_path(),
            errors,
        })
    }
}
