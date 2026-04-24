use actix_web::http::header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH};
use actix_web::{HttpRequest, HttpResponse};
use serde::Serialize;

use crate::application::error::AppError;
use crate::infrastructure::http::error::ApiError;

/// FNV-1a 64-bit over the serialized body. Deliberately not `DefaultHasher`
/// (SipHash with a process-global random seed): its output is not stable
/// across process restarts, which would silently defeat `If-None-Match`
/// during the Cache-Control window after any deploy.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce4_84222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub fn respond_with_etag<T: Serialize>(
    request: &HttpRequest,
    value: &T,
    cache_control: &str,
) -> Result<HttpResponse, ApiError> {
    let body = serde_json::to_vec(value)
        .map_err(|e| AppError::Internal(Box::new(e)))?;

    let etag = format!("\"{:016x}\"", fnv1a_64(&body));

    // RFC 9110 §13.1.2: `If-None-Match: *` matches any current representation,
    // so when we are about to return a representation (the 200 path below), any
    // `*` in the list must short-circuit to 304.
    let matches = request
        .headers()
        .get(IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split(',').any(|tag| {
                let trimmed = tag.trim();
                trimmed == "*" || trimmed == etag
            })
        })
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
