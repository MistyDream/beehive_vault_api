use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpRequest, HttpResponse, delete, get, patch, post, web};
use garde_actix_web::web::{Json, Query};

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockSearchResult;
use crate::domain::market::stock::NewStock;
use crate::infrastructure::http::PRIVATE_SHORT_CACHE;
use crate::infrastructure::http::dto::request::stock_request::{
    CreateStockRequest, StockSearchQuery, UpdateStockRequest,
};
use crate::infrastructure::http::dto::response::stock_response::{
    StockDetailResponse, StockResponse,
};
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::etag::{respond_with_etag, respond_with_etag_qualified};
use crate::infrastructure::http::state::AppState;

const X_RESULT_TRUNCATED: HeaderName = HeaderName::from_static("x-result-truncated");

#[get("/stocks")]
pub async fn search_stocks(
    state: web::Data<AppState>,
    query: Query<StockSearchQuery>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let q = query
        .into_inner()
        .q
        .ok_or_else(|| AppError::BadRequest("query parameter 'q' is required".to_string()))?;
    let StockSearchResult { items, truncated } =
        state.stock_service.search(q.trim().to_string()).await?;
    let response: Vec<StockResponse> = items.into_iter().map(StockResponse::from).collect();
    // Fold `truncated` into the ETag so a representation that shares its body
    // bytes with a previous response but differs in truncation status does not
    // get a 304 that drops the truncation signal.
    let discriminator: &[u8] = if truncated { b"t" } else { b"f" };
    let mut http_response =
        respond_with_etag_qualified(&request, &response, PRIVATE_SHORT_CACHE, discriminator)?;
    if truncated {
        http_response
            .headers_mut()
            .insert(X_RESULT_TRUNCATED, HeaderValue::from_static("true"));
    }
    Ok(http_response)
}

#[get("/stocks/{id}")]
pub async fn get_stock(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let stock = state.stock_service.get_by_id(path.into_inner()).await?;
    let response = StockDetailResponse::from(stock);
    Ok(respond_with_etag(&request, &response, PRIVATE_SHORT_CACHE)?)
}

#[post("/stocks")]
pub async fn create_stock(
    state: web::Data<AppState>,
    body: Json<CreateStockRequest>,
) -> Result<HttpResponse, ApiError> {
    let new = NewStock::from(body.into_inner());
    let stock = state.stock_service.create(new).await?;
    let location = format!("/v1/stocks/{}", stock.id);
    Ok(HttpResponse::Created()
        .insert_header(("Location", location))
        .json(StockDetailResponse::from(stock)))
}

#[patch("/stocks/{id}")]
pub async fn update_stock(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: Json<UpdateStockRequest>,
) -> Result<HttpResponse, ApiError> {
    let stock = state
        .stock_service
        .update(path.into_inner(), body.into_inner().into())
        .await?;
    Ok(HttpResponse::Ok().json(StockDetailResponse::from(stock)))
}

#[delete("/stocks/{id}")]
pub async fn delete_stock(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    state.stock_service.delete(path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
