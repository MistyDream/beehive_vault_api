use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use actix_web::http::header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH};
use actix_web::{HttpRequest, HttpResponse};
use serde::Serialize;

use crate::application::error::AppError;
use crate::infrastructure::http::error::ApiError;

pub fn respond_with_etag<T: Serialize>(
    request: &HttpRequest,
    value: &T,
    cache_control: &str,
) -> Result<HttpResponse, ApiError> {
    let body = serde_json::to_vec(value)
        .map_err(|e| AppError::Internal(Box::new(e)))?;

    let mut hasher = DefaultHasher::new();
    hasher.write(&body);
    let etag = format!("\"{:016x}\"", hasher.finish());

    let matches = request
        .headers()
        .get(IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').any(|tag| tag.trim() == etag))
        .unwrap_or(false);

    if matches {
        return Ok(HttpResponse::NotModified()
            .insert_header((ETAG, etag))
            .insert_header((CACHE_CONTROL, cache_control))
            .finish());
    }

    Ok(HttpResponse::Ok()
        .insert_header((ETAG, etag))
        .insert_header((CACHE_CONTROL, cache_control))
        .content_type("application/json")
        .body(body))
}
