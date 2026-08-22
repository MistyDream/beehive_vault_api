use std::error::Error;

use axum::{
    Json,
    extract::{
        FromRequest, FromRequestParts, Path, Query,
        path::ErrorKind,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
};
use serde::de::DeserializeOwned;
use serde_path_to_error::{Path as SerdePath, Segment};

use crate::error::{ApiError, InvalidParameter, InvalidParameterLocation, ProblemKind};

pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(
        request: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(request, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(json_rejection)
    }
}

pub struct ApiPath<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Path::<T>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(path_rejection)
    }
}

pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(query_rejection)
    }
}

fn json_rejection(rejection: JsonRejection) -> ApiError {
    match rejection {
        JsonRejection::JsonDataError(error) => {
            let pointer =
                find_error_source::<serde_path_to_error::Error<serde_json::Error>>(&error)
                    .map(|error| json_pointer(error.path()))
                    .unwrap_or_else(|| "#/".to_owned());
            ApiError::body_validation(
                pointer,
                "invalid_value",
                "The value has an invalid format or type.",
            )
        }
        JsonRejection::JsonSyntaxError(_) => ApiError::new(ProblemKind::InvalidRequest)
            .with_detail("The request body contains invalid JSON."),
        JsonRejection::MissingJsonContentType(_) => {
            ApiError::new(ProblemKind::UnsupportedMediaType)
                .with_detail("The request must use Content-Type: application/json.")
        }
        JsonRejection::BytesRejection(error) if error.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            ApiError::new(ProblemKind::PayloadTooLarge)
                .with_detail("The request body exceeds the supported size.")
        }
        JsonRejection::BytesRejection(error) => {
            tracing::error!(%error, "request body extraction failed");
            ApiError::new(ProblemKind::InternalError)
        }
        _ => ApiError::new(ProblemKind::InternalError),
    }
}

fn path_rejection(rejection: PathRejection) -> ApiError {
    match rejection {
        PathRejection::FailedToDeserializePathParams(error) => {
            let pointer = match error.kind() {
                ErrorKind::ParseErrorAtKey { key, .. }
                | ErrorKind::DeserializeError { key, .. }
                | ErrorKind::InvalidUtf8InPathParam { key } => {
                    format!("#/{}", escape_pointer_segment(key))
                }
                ErrorKind::ParseErrorAtIndex { index, .. } => format!("#/{index}"),
                _ => "#/".to_owned(),
            };
            ApiError::new(ProblemKind::InvalidRequest)
                .with_detail("One or more path parameters are invalid.")
                .with_errors(vec![InvalidParameter::new(
                    InvalidParameterLocation::Path,
                    pointer,
                    "invalid_value",
                    "The path parameter has an invalid format.",
                )])
        }
        PathRejection::MissingPathParams(error) => {
            tracing::error!(%error, "path parameter extraction failed");
            ApiError::new(ProblemKind::InternalError)
        }
        _ => ApiError::new(ProblemKind::InternalError),
    }
}

fn query_rejection(rejection: QueryRejection) -> ApiError {
    match rejection {
        QueryRejection::FailedToDeserializeQueryString(error) => {
            let pointer = find_error_source::<
                serde_path_to_error::Error<serde_urlencoded::de::Error>,
            >(&error)
            .map(|error| json_pointer(error.path()))
            .unwrap_or_else(|| "#/".to_owned());
            ApiError::validation(InvalidParameter::new(
                InvalidParameterLocation::Query,
                pointer,
                "invalid_value",
                "The query parameter has an invalid format or value.",
            ))
        }
        _ => ApiError::new(ProblemKind::InternalError),
    }
}

fn find_error_source<'a, T>(error: &'a (dyn Error + 'static)) -> Option<&'a T>
where
    T: Error + 'static,
{
    if let Some(error) = error.downcast_ref::<T>() {
        Some(error)
    } else {
        error.source().and_then(find_error_source::<T>)
    }
}

fn json_pointer(path: &SerdePath) -> String {
    let mut pointer = String::from("#");
    for segment in path {
        pointer.push('/');
        match segment {
            Segment::Seq { index } => pointer.push_str(&index.to_string()),
            Segment::Map { key } => pointer.push_str(&escape_pointer_segment(key)),
            Segment::Enum { variant } => pointer.push_str(&escape_pointer_segment(variant)),
            Segment::Unknown => {}
        }
    }
    if pointer == "#" {
        pointer.push('/');
    }
    pointer
}

fn escape_pointer_segment(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
