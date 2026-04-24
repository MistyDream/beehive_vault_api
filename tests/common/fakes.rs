//! Port implementations used by HTTP integration tests.
//!
//! - In-memory fakes (`InMemoryStockRepo`, `InMemoryStockPriceRepo`) for the
//!   ports actually exercised by the routes under test.
//! - No-op placeholders (`NoOp*`) for the remaining ports — they return
//!   `AppError::NotFound` or empty collections, never panic, and make it
//!   obvious which repo was unexpectedly hit if a test does start routing
//!   through the wrong code path.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use chrono::NaiveDate;

use beehive_vault_api::application::error::AppError;
use beehive_vault_api::application::ports::health_checker::HealthChecker;
use beehive_vault_api::application::ports::portfolio_repository::PortfolioRepository;
use beehive_vault_api::application::ports::score_snapshot_repository::ScoreSnapshotRepository;
use beehive_vault_api::application::ports::stock_price_repository::StockPriceRepository;
use beehive_vault_api::application::ports::stock_repository::StockRepository;
use beehive_vault_api::application::ports::transaction_repository::TransactionRepository;
use beehive_vault_api::domain::market::enums::MarketRegion;
use beehive_vault_api::domain::market::price::{NewPrice, Price};
use beehive_vault_api::domain::market::stock::Stock;
use beehive_vault_api::domain::scoring::score_snapshot::ScoreSnapshot;
use beehive_vault_api::domain::wallet::portfolio::{NewPortfolio, Portfolio, UpdatePortfolio};
use beehive_vault_api::domain::wallet::transaction::{
    NewTransaction, Transaction, TransactionFilter, UpdateTransaction,
};

// ============================== In-memory fakes ==============================

#[derive(Default)]
pub struct InMemoryStockRepo {
    by_id: Mutex<HashMap<i32, Stock>>,
}

impl InMemoryStockRepo {
    pub fn new(stocks: Vec<Stock>) -> Self {
        let map = stocks.into_iter().map(|s| (s.id, s)).collect();
        Self { by_id: Mutex::new(map) }
    }
}

impl StockRepository for InMemoryStockRepo {
    fn find_by_id(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.by_id
                .lock()
                .unwrap()
                .get(&stock_id)
                .cloned()
                .ok_or(AppError::NotFound)
        })
    }

    fn find_by_ids(
        &self,
        stock_ids: Vec<i32>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            let by_id = self.by_id.lock().unwrap();
            Ok(stock_ids
                .into_iter()
                .filter_map(|id| by_id.get(&id).cloned())
                .collect())
        })
    }

    fn find_by_symbol(
        &self,
        _symbol: String,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn find_by_isin(
        &self,
        _isin: String,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            Ok(self.by_id.lock().unwrap().values().cloned().collect())
        })
    }

    fn list_by_region(
        &self,
        region: MarketRegion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            Ok(self
                .by_id
                .lock()
                .unwrap()
                .values()
                .filter(|s| s.market_region == region)
                .cloned()
                .collect())
        })
    }

    fn delete(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            Ok(self.by_id.lock().unwrap().remove(&stock_id).is_some())
        })
    }
}

#[derive(Default)]
pub struct InMemoryStockPriceRepo {
    by_stock: Mutex<HashMap<i32, Vec<Price>>>,
}

impl InMemoryStockPriceRepo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_prices(prices: Vec<Price>) -> Self {
        let mut by_stock: HashMap<i32, Vec<Price>> = HashMap::new();
        for price in prices {
            by_stock.entry(price.stock_id).or_default().push(price);
        }
        for v in by_stock.values_mut() {
            v.sort_by_key(|p| p.price_date);
        }
        Self { by_stock: Mutex::new(by_stock) }
    }
}

impl StockPriceRepository for InMemoryStockPriceRepo {
    fn upsert_many(
        &self,
        _prices: Vec<NewPrice>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(0) })
    }

    fn find_latest(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Price>, AppError>> + Send + '_>> {
        Box::pin(async move {
            Ok(self
                .by_stock
                .lock()
                .unwrap()
                .get(&stock_id)
                .and_then(|v| v.last().cloned()))
        })
    }

    fn find_latest_batch(
        &self,
        stock_ids: Vec<i32>,
    ) -> Pin<Box<dyn Future<Output = Result<HashMap<i32, Price>, AppError>> + Send + '_>>
    {
        Box::pin(async move {
            let by_stock = self.by_stock.lock().unwrap();
            Ok(stock_ids
                .into_iter()
                .filter_map(|id| by_stock.get(&id).and_then(|v| v.last()).map(|p| (id, p.clone())))
                .collect())
        })
    }

    fn find_history(
        &self,
        stock_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Price>, AppError>> + Send + '_>> {
        Box::pin(async move {
            Ok(self
                .by_stock
                .lock()
                .unwrap()
                .get(&stock_id)
                .map(|v| {
                    v.iter()
                        .filter(|p| p.price_date >= from && p.price_date <= to)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default())
        })
    }
}

// ================================ No-op fakes ================================

pub struct NoOpPortfolioRepo;

impl PortfolioRepository for NoOpPortfolioRepo {
    fn find_by_id(
        &self,
        _id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Portfolio>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn insert(
        &self,
        _new: NewPortfolio,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn update(
        &self,
        _id: i32,
        _data: UpdatePortfolio,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn delete(
        &self,
        _id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(false) })
    }
}

pub struct NoOpTransactionRepo;

impl TransactionRepository for NoOpTransactionRepo {
    fn find_by_id(
        &self,
        _portfolio_id: i32,
        _tx_id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn list_by_portfolio(
        &self,
        _portfolio_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn list_by_portfolio_chronological(
        &self,
        _portfolio_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn list_by_portfolio_filtered(
        &self,
        _portfolio_id: i32,
        _filters: TransactionFilter,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn insert(
        &self,
        _new: NewTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn update(
        &self,
        _portfolio_id: i32,
        _tx_id: i64,
        _data: UpdateTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn delete(
        &self,
        _portfolio_id: i32,
        _tx_id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(false) })
    }
}

pub struct NoOpScoreSnapshotRepo;

impl ScoreSnapshotRepository for NoOpScoreSnapshotRepo {
    fn find_by_id(
        &self,
        _snapshot_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<ScoreSnapshot, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn find_by_stock(
        &self,
        _stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ScoreSnapshot>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn find_latest_by_stock(
        &self,
        _stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<ScoreSnapshot, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn delete(
        &self,
        _snapshot_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(false) })
    }

    fn delete_by_stock(
        &self,
        _stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(0) })
    }
}

pub struct AlwaysReadyHealthChecker;

impl HealthChecker for AlwaysReadyHealthChecker {
    fn readiness(&self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }
}

// ================================ Small builders =============================

pub fn test_stock(id: i32, symbol: &str, currency: &str) -> Stock {
    Stock {
        id,
        symbol: symbol.to_string(),
        name: format!("Stock {id}"),
        isin: format!("ISIN{id:04}"),
        currency: currency.to_string(),
        market_region: MarketRegion::Europe,
        market: None,
        sector: None,
        industry: None,
        country: None,
    }
}

pub fn test_price(stock_id: i32, date: NaiveDate, close: &str) -> Price {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    Price {
        id: stock_id as i64,
        stock_id,
        price_date: date,
        close: Decimal::from_str(close).unwrap(),
        source: "yahoo".to_string(),
        fetched_at: date.and_hms_opt(12, 0, 0).unwrap(),
    }
}
