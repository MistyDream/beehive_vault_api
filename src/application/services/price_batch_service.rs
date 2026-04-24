use std::sync::Arc;

use chrono::{Days, Utc};
use tracing::{info, warn};

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
                    to_persist.extend(fetched.into_iter().map(|fp| NewPrice {
                        stock_id: stock.id,
                        price_date: fp.price_date,
                        close: fp.close,
                        source: fp.source,
                    }));
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

        if !to_persist.is_empty() {
            report.prices_persisted = self.price_repo.upsert_many(to_persist).await?;
        }

        info!(
            region = region.as_str(),
            stocks_total = report.stocks_total,
            stocks_ok = report.stocks_ok,
            stocks_failed = report.stocks_failed,
            prices_persisted = report.prices_persisted,
            "price batch completed"
        );
        Ok(report)
    }

    /// Backfill the last 5 years of closes for a single stock. Intended to be
    /// called right after a stock is created so the wallet can value it
    /// immediately. Failures propagate to the caller (creation path decides
    /// whether to treat it as fatal or best-effort).
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
            .map(|fp| NewPrice {
                stock_id: stock.id,
                price_date: fp.price_date,
                close: fp.close,
                source: fp.source,
            })
            .collect();
        self.price_repo.upsert_many(rows).await
    }
}
