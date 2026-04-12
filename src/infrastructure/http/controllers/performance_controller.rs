use actix_web::{HttpResponse, get, web};

use crate::application::error::AppError;
use crate::domain::wallet::performance::compute_performance;
use crate::infrastructure::http::dto::request::transaction_request::PerformanceQueryParams;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::repositories::{
    portfolio_repository, transaction_repository,
};
use crate::infrastructure::persistence::repositories::transaction_repository::TransactionFilter;

#[get("/portfolios/{id}/performance")]
pub async fn get_performance(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: web::Query<PerformanceQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();

    let portfolio = portfolio_repository::find_by_id(&state.db, portfolio_id)
        .await
        .map_err(AppError::from)?;

    let transactions = if query.from_date.is_some() || query.to_date.is_some() {
        let filters = TransactionFilter {
            transaction_type: None,
            stock_id: None,
            from_date: query.from_date,
            to_date: query.to_date,
        };
        transaction_repository::list_by_portfolio_filtered(&state.db, portfolio_id, filters)
            .await
            .map_err(AppError::from)?
    } else {
        transaction_repository::list_by_portfolio_chronological(&state.db, portfolio_id)
            .await
            .map_err(AppError::from)?
    };

    let report = compute_performance(portfolio_id, &portfolio.currency, &transactions);
    Ok(HttpResponse::Ok().json(report))
}
