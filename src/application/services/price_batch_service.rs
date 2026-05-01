use std::sync::Arc;
use std::time::Instant;

use chrono::{Days, Utc};
use tracing::{info, warn, Instrument};

use crate::application::error::AppError;
use crate::application::ports::price_fetcher::PriceFetcher;
use crate::application::ports::stock_price_repository::StockPriceRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::enums::MarketRegion;
use crate::domain::market::price::NewPrice;

/// Days of look-back per daily batch. Generous enough to bridge weekends,
/// holidays and multi-day outages since we upsert on `(stock_id, price_date)`.
const DAILY_LOOKBACK_DAYS: u64 = 7;

/// Look-back window applied when backfilling a freshly-created stock. Covers
/// the horizon of a typical retail investor (Q D decision).
const BACKFILL_LOOKBACK_DAYS: u64 = 365 * 5;

/// Summary of a single `fetch_region` invocation. Returned to the caller and
/// emitted as an `info!` tracing event so the operator can audit batch runs.
#[derive(Debug, Default, Clone)]
pub struct FetchReport {
    pub region: Option<MarketRegion>,
    pub stocks_total: usize,
    pub stocks_ok: usize,
    pub stocks_failed: usize,
    pub prices_persisted: usize,
}

pub struct PriceBatchService {
    stock_repo: Arc<dyn StockRepository>,
    price_repo: Arc<dyn StockPriceRepository>,
    fetcher: Arc<dyn PriceFetcher>,
}

impl PriceBatchService {
    pub fn new(
        stock_repo: Arc<dyn StockRepository>,
        price_repo: Arc<dyn StockPriceRepository>,
        fetcher: Arc<dyn PriceFetcher>,
    ) -> Self {
        Self { stock_repo, price_repo, fetcher }
    }

    /// Fetch the last `DAILY_LOOKBACK_DAYS` of closes for every stock in the
    /// region and upsert them. Individual stock failures are logged and counted
    /// in the report but never abort the batch.
    pub async fn fetch_region(&self, region: MarketRegion) -> Result<FetchReport, AppError> {
        let span = tracing::info_span!("price_batch.fetch_region", region = region.as_str());
        self.fetch_region_inner(region).instrument(span).await
    }

    async fn fetch_region_inner(&self, region: MarketRegion) -> Result<FetchReport, AppError> {
        let started_at = Instant::now();
        let today = Utc::now().date_naive();
        let from = today
            .checked_sub_days(Days::new(DAILY_LOOKBACK_DAYS))
            .expect("today - 7 days is a valid date");

        let stocks = self.stock_repo.list_by_region(region).await?;
        let mut report = FetchReport {
            region: Some(region),
            stocks_total: stocks.len(),
            ..FetchReport::default()
        };

        let mut to_persist: Vec<NewPrice> = Vec::new();
        for stock in stocks {
            match self
                .fetcher
                .fetch_history(stock.symbol.clone(), from, today)
                .await
            {
                Ok(fetched) => {
                    report.stocks_ok += 1;
                    to_persist.extend(fetched.into_iter().map(|fp| fp.into_new_price(stock.id)));
                }
                Err(err) => {
                    report.stocks_failed += 1;
                    warn!(
                        stock_id = stock.id,
                        symbol = %stock.symbol,
                        error = %err,
                        "price fetch failed, skipping stock"
                    );
                }
            }
        }

        // `upsert_many` short-circuits on empty input, so no caller-side guard needed.
        report.prices_persisted = self.price_repo.upsert_many(to_persist).await?;

        info!(
            stocks_total = report.stocks_total,
            stocks_ok = report.stocks_ok,
            stocks_failed = report.stocks_failed,
            prices_persisted = report.prices_persisted,
            duration_ms = started_at.elapsed().as_millis() as u64,
            "price batch completed"
        );
        Ok(report)
    }

    /// Backfill the last 5 years of closes for a single stock. Intended to be
    /// called right after a stock is created so the wallet can value it
    /// immediately. Failures propagate to the caller (creation path decides
    /// whether to treat it as fatal or best-effort).
    #[tracing::instrument(skip(self), fields(stock_id = %stock_id))]
    pub async fn backfill_stock(&self, stock_id: i32) -> Result<usize, AppError> {
        let stock = self.stock_repo.find_by_id(stock_id).await?;
        let today = Utc::now().date_naive();
        let from = today
            .checked_sub_days(Days::new(BACKFILL_LOOKBACK_DAYS))
            .expect("today - 5 years is a valid date");

        let fetched = self.fetcher.fetch_history(stock.symbol, from, today).await?;
        if fetched.is_empty() {
            return Ok(0);
        }

        let rows: Vec<NewPrice> = fetched
            .into_iter()
            .map(|fp| fp.into_new_price(stock.id))
            .collect();
        self.price_repo.upsert_many(rows).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::str::FromStr;
    use std::sync::Mutex;

    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    use crate::application::ports::price_fetcher::{FetchedPrice, PriceFetcher};
    use crate::application::ports::stock_price_repository::StockPriceRepository;
    use crate::application::ports::stock_repository::{StockRepository, StockSearchResult};
    use crate::domain::market::price::Price;
    use crate::domain::market::isin::Isin;
    use crate::domain::market::stock::{NewStock, Stock, UpdateStock};

    // ---------- fakes ----------

    struct FakeStockRepo {
        by_region: HashMap<MarketRegion, Vec<Stock>>,
    }

    impl FakeStockRepo {
        fn with_region(region: MarketRegion, stocks: Vec<Stock>) -> Self {
            let mut by_region = HashMap::new();
            by_region.insert(region, stocks);
            Self { by_region }
        }
    }

    impl StockRepository for FakeStockRepo {
        fn find_by_id(
            &self,
            _stock_id: i32,
        ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
            Box::pin(async move { Err(AppError::NotFound) })
        }

        fn find_by_ids(
            &self,
            _stock_ids: Vec<i32>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn find_by_symbol(
            &self,
            _symbol: String,
        ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
            Box::pin(async move { Err(AppError::NotFound) })
        }

        fn find_by_isin(
            &self,
            _isin: Isin,
        ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
            Box::pin(async move { Err(AppError::NotFound) })
        }

        fn search(
            &self,
            _query: String,
        ) -> Pin<Box<dyn Future<Output = Result<StockSearchResult, AppError>> + Send + '_>>
        {
            Box::pin(async move {
                Ok(StockSearchResult {
                    items: Vec::new(),
                    truncated: false,
                })
            })
        }

        fn list_by_region(
            &self,
            region: MarketRegion,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
            let stocks = self
                .by_region
                .get(&region)
                .cloned()
                .unwrap_or_default();
            Box::pin(async move { Ok(stocks) })
        }

        fn insert(
            &self,
            _new: NewStock,
        ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
            Box::pin(async move { Err(AppError::NotFound) })
        }

        fn update(
            &self,
            _stock_id: i32,
            _data: UpdateStock,
        ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
            Box::pin(async move { Err(AppError::NotFound) })
        }

        fn delete(
            &self,
            _stock_id: i32,
        ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(false) })
        }
    }

    #[derive(Default)]
    struct SpyingPriceRepo {
        /// Count of rows handed to `upsert_many`, per invocation.
        upserts: Mutex<Vec<Vec<NewPrice>>>,
    }

    impl SpyingPriceRepo {
        fn upserted_rows(&self) -> Vec<NewPrice> {
            self.upserts.lock().unwrap().iter().flatten().cloned().collect()
        }
    }

    // Clone needed because NewPrice isn't Clone by default — derive it in the fake
    // by collecting manually below. For simpler assertions we just count and sample.
    impl StockPriceRepository for SpyingPriceRepo {
        fn upsert_many(
            &self,
            prices: Vec<NewPrice>,
        ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
            let count = prices.len();
            self.upserts.lock().unwrap().push(prices);
            Box::pin(async move { Ok(count) })
        }

        fn find_latest(
            &self,
            _stock_id: i32,
        ) -> Pin<Box<dyn Future<Output = Result<Option<Price>, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(None) })
        }

        fn find_latest_batch(
            &self,
            _stock_ids: Vec<i32>,
        ) -> Pin<Box<dyn Future<Output = Result<HashMap<i32, Price>, AppError>> + Send + '_>>
        {
            Box::pin(async move { Ok(HashMap::new()) })
        }

        fn find_history(
            &self,
            _stock_id: i32,
            _from: NaiveDate,
            _to: NaiveDate,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<Price>, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
    }

    /// Deterministic fetcher: per symbol, returns the prices configured or an error.
    struct ProgrammableFetcher {
        by_symbol: HashMap<String, Result<Vec<FetchedPrice>, String>>,
    }

    impl PriceFetcher for ProgrammableFetcher {
        fn fetch_history(
            &self,
            symbol: String,
            _from: NaiveDate,
            _to: NaiveDate,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<FetchedPrice>, AppError>> + Send + '_>>
        {
            let outcome = self.by_symbol.get(&symbol).cloned();
            Box::pin(async move {
                match outcome {
                    Some(Ok(prices)) => Ok(prices),
                    Some(Err(msg)) => {
                        Err(AppError::Internal(Box::<dyn std::error::Error + Send + Sync>::from(msg)))
                    }
                    None => Ok(Vec::new()),
                }
            })
        }
    }

    // ---------- helpers ----------

    fn stock(id: i32, symbol: &str) -> Stock {
        Stock {
            id,
            symbol: symbol.to_string(),
            name: format!("Stock {id}"),
            isin: Isin::try_new(&format!("US{id:010}")).unwrap(),
            currency: "EUR".to_string(),
            market_region: MarketRegion::Europe,
            market: None,
            sector: None,
            industry: None,
            country: None,
        }
    }

    fn fetched(date_str: (i32, u32, u32), close: &str) -> FetchedPrice {
        FetchedPrice {
            price_date: NaiveDate::from_ymd_opt(date_str.0, date_str.1, date_str.2).unwrap(),
            close: Decimal::from_str(close).unwrap(),
            source: "yahoo".to_string(),
        }
    }

    // ---------- tests ----------

    #[tokio::test]
    async fn fetch_region_reports_empty_when_no_stocks_in_region() {
        let stock_repo = Arc::new(FakeStockRepo::with_region(MarketRegion::Europe, vec![]));
        let price_repo = Arc::new(SpyingPriceRepo::default());
        let fetcher = Arc::new(ProgrammableFetcher {
            by_symbol: HashMap::new(),
        });
        let service = PriceBatchService::new(stock_repo, price_repo.clone(), fetcher);

        let report = service.fetch_region(MarketRegion::Europe).await.unwrap();

        assert_eq!(report.stocks_total, 0);
        assert_eq!(report.stocks_ok, 0);
        assert_eq!(report.stocks_failed, 0);
        assert_eq!(report.prices_persisted, 0);
        assert!(price_repo.upserted_rows().is_empty());
    }

    #[tokio::test]
    async fn fetch_region_continues_when_one_stock_fetch_fails() {
        // Two stocks, one fetch succeeds with two prices, the other errors.
        // Batch must NOT abort: one upserted, the other counted as failed.
        let stocks = vec![stock(1, "OK.PA"), stock(2, "KO.PA")];
        let stock_repo = Arc::new(FakeStockRepo::with_region(MarketRegion::Europe, stocks));

        let mut by_symbol = HashMap::new();
        by_symbol.insert(
            "OK.PA".to_string(),
            Ok(vec![fetched((2026, 4, 23), "100.00"), fetched((2026, 4, 24), "101.00")]),
        );
        by_symbol.insert("KO.PA".to_string(), Err("rate limited".to_string()));
        let fetcher = Arc::new(ProgrammableFetcher { by_symbol });

        let price_repo = Arc::new(SpyingPriceRepo::default());
        let service = PriceBatchService::new(stock_repo, price_repo.clone(), fetcher);

        let report = service.fetch_region(MarketRegion::Europe).await.unwrap();

        assert_eq!(report.stocks_total, 2);
        assert_eq!(report.stocks_ok, 1);
        assert_eq!(report.stocks_failed, 1);
        assert_eq!(report.prices_persisted, 2);

        let rows = price_repo.upserted_rows();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.stock_id == 1));
        assert!(rows.iter().all(|r| r.source == "yahoo"));
    }

    #[tokio::test]
    async fn fetch_region_counts_all_failures_when_every_fetch_errors() {
        // Full-region outage (Yahoo down, auth expired, ...): every fetch fails.
        // The batch must still return Ok with stocks_failed == stocks_total and
        // must NOT call upsert_many since there is nothing to persist.
        let stocks = vec![stock(1, "KO1.PA"), stock(2, "KO2.PA")];
        let stock_repo = Arc::new(FakeStockRepo::with_region(MarketRegion::Europe, stocks));

        let mut by_symbol = HashMap::new();
        by_symbol.insert("KO1.PA".to_string(), Err("rate limited".to_string()));
        by_symbol.insert("KO2.PA".to_string(), Err("upstream 500".to_string()));
        let fetcher = Arc::new(ProgrammableFetcher { by_symbol });

        let price_repo = Arc::new(SpyingPriceRepo::default());
        let service = PriceBatchService::new(stock_repo, price_repo.clone(), fetcher);

        let report = service.fetch_region(MarketRegion::Europe).await.unwrap();

        assert_eq!(report.stocks_total, 2);
        assert_eq!(report.stocks_ok, 0);
        assert_eq!(report.stocks_failed, 2);
        assert_eq!(report.prices_persisted, 0);
        assert!(
            price_repo.upserted_rows().is_empty(),
            "no rows must be upserted when every fetch fails"
        );
    }

    #[tokio::test]
    async fn fetch_region_persists_nothing_when_every_fetch_is_empty() {
        let stocks = vec![stock(1, "EMPTY.PA")];
        let stock_repo = Arc::new(FakeStockRepo::with_region(MarketRegion::Europe, stocks));
        // Fetcher returns an empty Vec (e.g. market closed / no trades in window).
        let mut by_symbol = HashMap::new();
        by_symbol.insert("EMPTY.PA".to_string(), Ok(vec![]));
        let fetcher = Arc::new(ProgrammableFetcher { by_symbol });

        let price_repo = Arc::new(SpyingPriceRepo::default());
        let service = PriceBatchService::new(stock_repo, price_repo.clone(), fetcher);

        let report = service.fetch_region(MarketRegion::Europe).await.unwrap();

        assert_eq!(report.stocks_ok, 1);
        assert_eq!(report.prices_persisted, 0);
        assert!(price_repo.upserted_rows().is_empty());
    }
}
