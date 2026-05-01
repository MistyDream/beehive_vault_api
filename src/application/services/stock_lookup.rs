use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::stock::Stock;
use crate::domain::wallet::transaction::Transaction;

pub async fn fetch_stocks_for_transactions(
    stock_repo: &Arc<dyn StockRepository>,
    transactions: &[Transaction],
) -> Result<HashMap<i32, Stock>, AppError> {
    let stock_ids: Vec<i32> = transactions
        .iter()
        .filter_map(|t| t.stock_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let stocks = stock_repo.find_by_ids(stock_ids).await?;
    Ok(stocks.into_iter().map(|s| (s.id, s)).collect())
}

pub async fn fetch_stock_by_id_optional(
    stock_repo: &Arc<dyn StockRepository>,
    stock_id: Option<i32>,
) -> Result<HashMap<i32, Stock>, AppError> {
    let Some(id) = stock_id else {
        return Ok(HashMap::new());
    };
    let stocks = stock_repo.find_by_ids(vec![id]).await?;
    Ok(stocks.into_iter().map(|s| (s.id, s)).collect())
}
