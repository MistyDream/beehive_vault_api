use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;

use crate::application::error::AppError;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    App(#[from] AppError),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::App(err) => match err {
                AppError::NotFound => StatusCode::NOT_FOUND,
                AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
                AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
        })
    }
}
