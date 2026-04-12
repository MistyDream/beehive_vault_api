use actix_web::{HttpResponse, Responder, get, web};

use crate::application::error::AppError;
use crate::domain::wallet::cash_balance::compute_cash_balance;
use crate::domain::wallet::portfolio_summary::PortfolioSummary;
use crate::domain::wallet::position::compute_positions;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::repositories::{
    portfolio_repository, transaction_repository,
};

#[get("/portfolios/{id}/positions")]
pub async fn get_positions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();

    let transactions = transaction_repository::list_by_portfolio_chronological(
        &state.db,
        portfolio_id,
    )
    .await
    .map_err(AppError::from)?;

    let positions = compute_positions(&transactions);
    Ok(HttpResponse::Ok().json(positions))
}

#[get("/portfolios/{id}/cash")]
pub async fn get_cash_balance(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();

    let portfolio = portfolio_repository::find_by_id(&state.db, portfolio_id)
        .await
        .map_err(AppError::from)?;

    let transactions = transaction_repository::list_by_portfolio_chronological(
        &state.db,
        portfolio_id,
    )
    .await
    .map_err(AppError::from)?;

    let cash = compute_cash_balance(&transactions, &portfolio.currency);
    Ok(HttpResponse::Ok().json(cash))
}

#[get("/portfolios/{id}/summary")]
pub async fn get_portfolio_summary(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();

    let portfolio = portfolio_repository::find_by_id(&state.db, portfolio_id)
        .await
        .map_err(AppError::from)?;

    let transactions = transaction_repository::list_by_portfolio_chronological(
        &state.db,
        portfolio_id,
    )
    .await
    .map_err(AppError::from)?;

    let positions = compute_positions(&transactions);
    let cash = compute_cash_balance(&transactions, &portfolio.currency);
    let total_invested: f64 = positions.iter().map(|p| p.total_cost).sum();

    let summary = PortfolioSummary {
        portfolio,
        positions,
        cash,
        total_invested,
    };

    Ok(HttpResponse::Ok().json(summary))
}
