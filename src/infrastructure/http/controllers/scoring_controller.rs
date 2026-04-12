use actix_web::{HttpResponse, get, web};

use crate::application::error::AppError;
use crate::domain::wallet::portfolio_scoring::{PortfolioScoring, StockScore};
use crate::domain::wallet::position::compute_positions;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::repositories::{
    portfolio_repository, score_snapshot_repository, stock_repository, transaction_repository,
};

#[get("/portfolios/{id}/scoring")]
pub async fn get_portfolio_scoring(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
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

    if positions.is_empty() {
        return Ok(HttpResponse::Ok().json(PortfolioScoring {
            portfolio_id: portfolio.id,
            stock_scores: vec![],
            weighted_score: None,
        }));
    }

    // Compute total portfolio value (by cost) for weighting
    let total_cost: f64 = positions.iter().map(|p| p.total_cost).sum();

    let mut stock_scores = Vec::with_capacity(positions.len());
    let mut weighted_sum = 0.0;
    let mut weighted_total = 0.0;

    for pos in &positions {
        let weight = if total_cost > 0.0 {
            pos.total_cost / total_cost
        } else {
            0.0
        };

        let stock = stock_repository::find_by_id(&state.db, pos.stock_id)
            .await
            .map_err(AppError::from)?;

        let snapshot = score_snapshot_repository::find_latest_by_stock(&state.db, pos.stock_id)
            .await
            .ok();

        if let Some(ref snap) = snapshot {
            weighted_sum += weight * snap.global_score;
            weighted_total += weight;
        }

        stock_scores.push(StockScore {
            stock_id: pos.stock_id,
            symbol: stock.symbol,
            name: stock.name,
            weight,
            global_score: snapshot.as_ref().map(|s| s.global_score),
            scored_at: snapshot.as_ref().map(|s| s.scored_at),
        });
    }

    let weighted_score = if weighted_total > 0.0 {
        Some(weighted_sum / weighted_total)
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(PortfolioScoring {
        portfolio_id: portfolio.id,
        stock_scores,
        weighted_score,
    }))
}
