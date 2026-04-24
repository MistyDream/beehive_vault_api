use actix_web::{HttpRequest, HttpResponse, get, web};
use garde_actix_web::web::Query;

use crate::application::error::AppError;
use crate::infrastructure::http::PRIVATE_SHORT_CACHE;
use crate::infrastructure::http::dto::request::stock_price_request::PriceHistoryQuery;
use crate::infrastructure::http::dto::response::stock_price_response::{
    PriceHistoryResponse, PriceResponse,
};
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::etag::respond_with_etag;
use crate::infrastructure::http::state::AppState;

#[get("/stocks/{stock_id}/price")]
pub async fn get_stock_latest_price(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let stock_id = path.into_inner();
    let (price, stock) = state.price_service.get_latest(stock_id).await?;
    let response = PriceResponse::from_price(price, stock.currency);
    respond_with_etag(&request, &response, PRIVATE_SHORT_CACHE)
}

#[get("/stocks/{stock_id}/prices")]
pub async fn get_stock_price_history(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: Query<PriceHistoryQuery>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let stock_id = path.into_inner();
    let q = query.into_inner();
    let from = q
        .from
        .ok_or_else(|| AppError::BadRequest("query parameter 'from' is required".to_string()))?;
    let to = q
        .to
        .ok_or_else(|| AppError::BadRequest("query parameter 'to' is required".to_string()))?;
    let (prices, stock) = state.price_service.get_history(stock_id, from, to).await?;
    let response = PriceHistoryResponse::from_prices(prices, stock.currency);
    respond_with_etag(&request, &response, PRIVATE_SHORT_CACHE)
}
