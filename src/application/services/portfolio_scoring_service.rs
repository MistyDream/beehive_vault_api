use std::collections::HashMap;
use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::application::ports::score_snapshot_repository::ScoreSnapshotRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::domain::wallet::portfolio_scoring::{PortfolioScoring, StockScore};
use crate::domain::wallet::position::compute_positions;

pub struct PortfolioScoringService {
    portfolio_repo: Arc<dyn PortfolioRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
    stock_repo: Arc<dyn StockRepository>,
    score_repo: Arc<dyn ScoreSnapshotRepository>,
}

impl PortfolioScoringService {
    pub fn new(
        portfolio_repo: Arc<dyn PortfolioRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
        stock_repo: Arc<dyn StockRepository>,
        score_repo: Arc<dyn ScoreSnapshotRepository>,
    ) -> Self {
        Self { portfolio_repo, transaction_repo, stock_repo, score_repo }
    }

    pub async fn get_scoring(&self, portfolio_id: i32) -> Result<PortfolioScoring, AppError> {
        let portfolio = self.portfolio_repo.find_by_id(portfolio_id).await?;
        let transactions = self.transaction_repo
            .list_by_portfolio_chronological(portfolio_id)
            .await?;

        let stock_ids: Vec<i32> = transactions.iter().filter_map(|t| t.stock_id).collect::<std::collections::HashSet<_>>().into_iter().collect();
        let stocks = self.stock_repo.find_by_ids(stock_ids).await?;
        let stocks_by_id: HashMap<i32, _> = stocks.into_iter().map(|s| (s.id, s)).collect();

        let positions = compute_positions(&transactions, &stocks_by_id);

        if positions.is_empty() {
            return Ok(PortfolioScoring {
                portfolio_id: portfolio.id,
                stock_scores: vec![],
                weighted_score: None,
            });
        }

        let mut stock_scores = Vec::with_capacity(positions.len());
        let mut weighted_sum = 0.0;
        let mut weighted_total = 0.0;

        for pos in &positions {
            let weight_fraction = pos.weight / 100.0;
            let snapshot = self.score_repo.find_latest_by_stock(pos.stock.id).await.ok();

            if let Some(ref snap) = snapshot {
                weighted_sum += weight_fraction * snap.global_score;
                weighted_total += weight_fraction;
            }

            stock_scores.push(StockScore {
                stock_id: pos.stock.id,
                symbol: pos.stock.symbol.clone(),
                name: pos.stock.name.clone(),
                weight: weight_fraction,
                global_score: snapshot.as_ref().map(|s| s.global_score),
                scored_at: snapshot.as_ref().map(|s| s.scored_at),
            });
        }

        let weighted_score = if weighted_total > 0.0 { Some(weighted_sum / weighted_total) } else { None };

        Ok(PortfolioScoring {
            portfolio_id: portfolio.id,
            stock_scores,
            weighted_score,
        })
    }
}
