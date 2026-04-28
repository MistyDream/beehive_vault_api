use actix_web::{HttpResponse, get, web};

use crate::infrastructure::http::dto::response::stock_response::StockResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[get("/stocks")]
pub async fn list_stocks(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let stocks = state.stock_service.list().await?;
    let response: Vec<StockResponse> = stocks.into_iter().map(StockResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}
