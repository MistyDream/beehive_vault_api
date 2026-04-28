use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, get, web};
use garde_actix_web::web::Query;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockSearchResult;
use crate::infrastructure::http::PRIVATE_SHORT_CACHE;
use crate::infrastructure::http::dto::request::stock_request::StockSearchQuery;
use crate::infrastructure::http::dto::response::stock_response::StockResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::etag::respond_with_etag;
use crate::infrastructure::http::state::AppState;

const X_RESULT_TRUNCATED: HeaderName = HeaderName::from_static("x-result-truncated");

#[get("/stocks")]
pub async fn search_stocks(
    state: web::Data<AppState>,
    query: Query<StockSearchQuery>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let raw = query
        .into_inner()
        .q
        .ok_or_else(|| AppError::BadRequest("query parameter 'q' is required".to_string()))?;
    let q = raw.trim();
    if q.len() < 2 {
        return Err(AppError::BadRequest(
            "query parameter 'q' must contain at least 2 non-whitespace characters".to_string(),
        )
        .into());
    }
    let StockSearchResult { items, truncated } =
        state.stock_service.search(q.to_string()).await?;
    let response: Vec<StockResponse> = items.into_iter().map(StockResponse::from).collect();
    let mut http_response = respond_with_etag(&request, &response, PRIVATE_SHORT_CACHE)?;
    if truncated {
        http_response
            .headers_mut()
            .insert(X_RESULT_TRUNCATED, HeaderValue::from_static("true"));
    }
    Ok(http_response)
}
