use actix_web::{HttpResponse, ResponseError, http::StatusCode};

use crate::application::error::AppError;
use crate::infrastructure::http::dto::response::error_response::{FieldError, ProblemDetail};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    App(#[from] AppError),

    #[error("Validation failed")]
    Validation(Vec<FieldError>),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::App(err) => match err {
                AppError::NotFound => StatusCode::NOT_FOUND,
                AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        let (title, detail, errors) = match self {
            ApiError::App(AppError::NotFound) => (
                "Not Found".to_string(),
                Some("The requested resource was not found".to_string()),
                None,
            ),
            ApiError::App(AppError::Internal(_)) => (
                "Internal Server Error".to_string(),
                None,
                None,
            ),
            ApiError::Validation(fields) => (
                "Validation Error".to_string(),
                Some("One or more fields are invalid".to_string()),
                Some(fields.clone()),
            ),
        };

        HttpResponse::build(status)
            .content_type("application/problem+json")
            .json(ProblemDetail {
                problem_type: "about:blank".to_string(),
                title,
                status: status.as_u16(),
                detail,
                instance: None,
                errors,
            })
    }
}
