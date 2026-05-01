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
use uuid::Uuid;

use beehive_vault_api::application::error::AppError;
use beehive_vault_api::application::ports::health_checker::HealthChecker;
use beehive_vault_api::application::ports::portfolio_repository::PortfolioRepository;
use beehive_vault_api::application::ports::score_snapshot_repository::ScoreSnapshotRepository;
use beehive_vault_api::application::ports::stock_price_repository::StockPriceRepository;
use beehive_vault_api::application::ports::stock_repository::{
    StockRepository, StockSearchResult,
};
use beehive_vault_api::application::ports::transaction_repository::TransactionRepository;
use beehive_vault_api::domain::market::enums::MarketRegion;
use beehive_vault_api::domain::market::isin::Isin;
use beehive_vault_api::domain::market::price::{NewPrice, Price};
use beehive_vault_api::domain::market::stock::{NewStock, Stock, UpdateStock};
use beehive_vault_api::domain::scoring::score_snapshot::ScoreSnapshot;
use beehive_vault_api::domain::wallet::portfolio::{NewPortfolio, Portfolio, UpdatePortfolio};
use beehive_vault_api::domain::wallet::transaction::{
    NewTransaction, Transaction, TransactionFilter, UpdateTransaction,
};

// ============================== In-memory fakes ==============================

#[derive(Default)]
pub struct InMemoryStockRepo {
    by_id: Mutex<HashMap<i32, Stock>>,
    /// Stock ids whose `delete` will surface a `Conflict` instead of removing
    /// the row — mirrors what `PgStockRepository::delete` does when the
    /// `transactions_stock_id_fkey` ON DELETE RESTRICT FK fires.
    fk_protected: std::collections::HashSet<i32>,
}

impl InMemoryStockRepo {
    pub fn new(stocks: Vec<Stock>) -> Self {
        let map = stocks.into_iter().map(|s| (s.id, s)).collect();
        Self { by_id: Mutex::new(map), fk_protected: Default::default() }
    }

    pub fn with_fk_protected(stocks: Vec<Stock>, protected_ids: &[i32]) -> Self {
        let map = stocks.into_iter().map(|s| (s.id, s)).collect();
        Self {
            by_id: Mutex::new(map),
            fk_protected: protected_ids.iter().copied().collect(),
        }
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
        isin: Isin,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.by_id
                .lock()
                .unwrap()
                .values()
                .find(|s| s.isin == isin)
                .cloned()
                .ok_or(AppError::NotFound)
        })
    }

    fn search(
        &self,
        query: String,
    ) -> Pin<Box<dyn Future<Output = Result<StockSearchResult, AppError>> + Send + '_>> {
        Box::pin(async move {
            const FAKE_SEARCH_LIMIT: usize = 50;
            let needle = query.to_lowercase();
            let mut items: Vec<Stock> = self
                .by_id
                .lock()
                .unwrap()
                .values()
                .filter(|s| {
                    s.symbol.to_lowercase().contains(&needle)
                        || s.name.to_lowercase().contains(&needle)
                        || s.isin.as_str().to_lowercase().contains(&needle)
                })
                .cloned()
                .collect();
            // Mirror the prod adapter's `ORDER BY symbol ASC` so the cap evicts
            // the same items in tests as it would in production.
            items.sort_by(|a, b| a.symbol.cmp(&b.symbol));
            let truncated = items.len() > FAKE_SEARCH_LIMIT;
            if truncated {
                items.truncate(FAKE_SEARCH_LIMIT);
            }
            Ok(StockSearchResult { items, truncated })
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

    fn insert(
        &self,
        new: NewStock,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            let mut by_id = self.by_id.lock().unwrap();
            if by_id.values().any(|s| s.symbol == new.symbol) {
                return Err(AppError::Conflict("symbol already exists".to_string()));
            }
            if by_id.values().any(|s| s.isin == new.isin) {
                return Err(AppError::Conflict("isin already exists".to_string()));
            }
            let next_id = by_id.keys().max().copied().unwrap_or(0) + 1;
            let stock = Stock {
                id: next_id,
                symbol: new.symbol,
                name: new.name,
                isin: new.isin,
                currency: new.currency,
                market_region: new.market_region,
                market: new.market,
                sector: new.sector,
                industry: new.industry,
                country: new.country,
            };
            by_id.insert(next_id, stock.clone());
            Ok(stock)
        })
    }

    fn update(
        &self,
        stock_id: i32,
        data: UpdateStock,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            let mut by_id = self.by_id.lock().unwrap();
            if !by_id.contains_key(&stock_id) {
                return Err(AppError::NotFound);
            }
            if by_id
                .iter()
                .any(|(id, s)| *id != stock_id && s.symbol == data.symbol)
            {
                return Err(AppError::Conflict("symbol already exists".to_string()));
            }
            if by_id
                .iter()
                .any(|(id, s)| *id != stock_id && s.isin == data.isin)
            {
                return Err(AppError::Conflict("isin already exists".to_string()));
            }
            let stock = Stock {
                id: stock_id,
                symbol: data.symbol,
                name: data.name,
                isin: data.isin,
                currency: data.currency,
                market_region: data.market_region,
                market: data.market,
                sector: data.sector,
                industry: data.industry,
                country: data.country,
            };
            by_id.insert(stock_id, stock.clone());
            Ok(stock)
        })
    }

    fn delete(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        let blocked = self.fk_protected.contains(&stock_id);
        Box::pin(async move {
            if blocked {
                return Err(AppError::Conflict(
                    "stock is referenced by transactions and cannot be deleted".to_string(),
                ));
            }
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
        _id: Uuid,
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
        _id: Uuid,
        _data: UpdatePortfolio,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn delete(
        &self,
        _id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(false) })
    }
}

pub struct NoOpTransactionRepo;

impl TransactionRepository for NoOpTransactionRepo {
    fn find_by_id(
        &self,
        _portfolio_id: Uuid,
        _tx_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn list_by_portfolio(
        &self,
        _portfolio_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn list_by_portfolio_chronological(
        &self,
        _portfolio_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn list_by_portfolio_filtered(
        &self,
        _portfolio_id: Uuid,
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
        _portfolio_id: Uuid,
        _tx_id: Uuid,
        _data: UpdateTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move { Err(AppError::NotFound) })
    }

    fn delete(
        &self,
        _portfolio_id: Uuid,
        _tx_id: Uuid,
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

pub struct NotReadyHealthChecker;

impl HealthChecker for NotReadyHealthChecker {
    fn readiness(&self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move {
            Err(AppError::Internal(Box::new(std::io::Error::other(
                "simulated readiness failure",
            ))))
        })
    }
}

// ================================ Small builders =============================

/// Build a deterministic test stock. The generated ISIN is ISO 6166-formatted
/// (`US` + 10 zero-padded digits, last digit acts as the check digit) so it
/// passes path-level format validation without bespoke fixtures.
pub fn test_stock(id: i32, symbol: &str, currency: &str) -> Stock {
    Stock {
        id,
        symbol: symbol.to_string(),
        name: format!("Stock {id}"),
        isin: Isin::try_new(&format!("US{id:010}")).unwrap(),
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
