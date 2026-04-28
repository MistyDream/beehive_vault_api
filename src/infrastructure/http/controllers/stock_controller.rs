use actix_web::{HttpResponse, get, web};
use garde_actix_web::web::Query;

use crate::application::error::AppError;
use crate::infrastructure::http::dto::request::stock_request::StockSearchQuery;
use crate::infrastructure::http::dto::response::stock_response::StockResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[get("/stocks")]
pub async fn search_stocks(
    state: web::Data<AppState>,
    query: Query<StockSearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query
        .into_inner()
        .q
        .ok_or_else(|| AppError::BadRequest("query parameter 'q' is required".to_string()))?;
    let stocks = state.stock_service.search(q).await?;
    let response: Vec<StockResponse> = stocks.into_iter().map(StockResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}
