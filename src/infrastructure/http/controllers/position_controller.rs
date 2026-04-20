use actix_web::{HttpResponse, get, web};

use crate::application::services::position_service::PositionsQuery;
use crate::infrastructure::http::dto::request::position_request::PositionsQueryParams;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[get("/portfolios/{id}/positions")]
pub async fn get_positions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: web::Query<PositionsQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let result = state
        .position_service
        .get_positions_paginated(
            path.into_inner(),
            PositionsQuery {
                sort_by: q.sort_by,
                sort_dir: q.sort_dir,
                page: q.page,
                limit: q.limit,
            },
        )
        .await?;
    Ok(HttpResponse::Ok().json(result))
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
