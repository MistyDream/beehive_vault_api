use actix_web::{HttpResponse, post, web};

use crate::domain::market::stock::NewStock;
use crate::infrastructure::http::dto::request::stock_request::CreateStockRequest;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[post("/stocks")]
pub async fn create_stock(
    state: web::Data<AppState>,
    body: web::Json<CreateStockRequest>,
) -> Result<HttpResponse, ApiError> {
    let new_stock = NewStock::from(body.into_inner());

    let stock = state.stock_service.create_stock(new_stock).await?;

    Ok(HttpResponse::Created().json(stock))
}
