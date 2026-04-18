use actix_web::{HttpResponse, get, web};

use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[get("/portfolios/{id}/positions")]
pub async fn get_positions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let positions = state.position_service.get_positions(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(positions))
}

#[get("/portfolios/{id}/cash")]
pub async fn get_cash_balance(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let cash = state.position_service.get_cash_balance(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(cash))
}

#[get("/portfolios/{id}/summary")]
pub async fn get_portfolio_summary(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let summary = state.position_service.get_summary(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(summary))
}
