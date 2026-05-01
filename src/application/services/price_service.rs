use std::sync::Arc;

use chrono::{Days, NaiveDate};

use crate::application::error::AppError;
use crate::application::ports::stock_price_repository::StockPriceRepository;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::price::Price;
use crate::domain::market::stock::Stock;

/// Defence-in-depth cap on the price history window. The storage itself is
/// already bounded by the 5-year backfill policy, but this guard survives any
/// future extension of that policy and keeps a single unbounded client query
/// from scanning the whole table.
const MAX_HISTORY_WINDOW_DAYS: u64 = 365 * 10;

pub struct PriceService {
    stock_repo: Arc<dyn StockRepository>,
    price_repo: Arc<dyn StockPriceRepository>,
}

impl PriceService {
    pub fn new(
        stock_repo: Arc<dyn StockRepository>,
        price_repo: Arc<dyn StockPriceRepository>,
    ) -> Self {
        Self { stock_repo, price_repo }
    }

    /// Return the most recent close for `stock_id` along with the stock's currency.
    /// Errors: `NotFound` if the stock does not exist, `NotFound` if no price exists.
    pub async fn get_latest(&self, stock_id: i32) -> Result<(Price, Stock), AppError> {
        let stock = self.stock_repo.find_by_id(stock_id).await?;
        let price = self
            .price_repo
            .find_latest(stock_id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok((price, stock))
    }

    /// Return all persisted closes for `stock_id` between `from` and `to` (inclusive)
    /// along with the stock (used by callers to expose the currency).
    /// Errors: `NotFound` if the stock does not exist,
    /// `BadRequest` if `from > to`.
    pub async fn get_history(
        &self,
        stock_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<(Vec<Price>, Stock), AppError> {
        if from > to {
            return Err(AppError::BadRequest(
                "query parameter 'from' must be on or before 'to'".to_string(),
            ));
        }
        if from
            .checked_add_days(Days::new(MAX_HISTORY_WINDOW_DAYS))
            .is_none_or(|cap| to > cap)
        {
            return Err(AppError::BadRequest(format!(
                "price history window must not exceed {} days",
                MAX_HISTORY_WINDOW_DAYS
            )));
        }
        let stock = self.stock_repo.find_by_id(stock_id).await?;
        let prices = self.price_repo.find_history(stock_id, from, to).await?;
        Ok((prices, stock))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Mutex;

    use crate::application::ports::stock_repository::StockSearchResult;
    use crate::domain::market::enums::MarketRegion;
    use crate::domain::market::isin::Isin;
    use crate::domain::market::stock::{NewStock, UpdateStock};

    // ---------- minimal fakes for the two ports the service depends on ----------

    #[derive(Default)]
    struct FakeStockRepo {
        by_id: Mutex<HashMap<i32, Stock>>,
    }

    impl FakeStockRepo {
        fn with_stock(id: i32) -> Self {
            let mut map = HashMap::new();
            map.insert(
                id,
                Stock {
                    id,
                    symbol: format!("S{id}"),
                    name: format!("Stock {id}"),
                    isin: Isin::try_new(&format!("US{id:010}")).unwrap(),
                    currency: "EUR".to_string(),
                    market_region: MarketRegion::Europe,
                    market: None,
                    sector: None,
                    industry: None,
                    country: None,
                },
            );
            Self { by_id: Mutex::new(map) }
        }
    }

    impl crate::application::ports::stock_repository::StockRepository for FakeStockRepo {
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
            _region: MarketRegion,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(Vec::new()) })
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
    struct FakePriceRepo {
        latest: Mutex<HashMap<i32, Price>>,
    }

    impl FakePriceRepo {
        fn empty() -> Self {
            Self::default()
        }

        fn with_latest(stock_id: i32, price: Price) -> Self {
            let mut map = HashMap::new();
            map.insert(stock_id, price);
            Self { latest: Mutex::new(map) }
        }
    }

    impl crate::application::ports::stock_price_repository::StockPriceRepository for FakePriceRepo {
        fn upsert_many(
            &self,
            _prices: Vec<crate::domain::market::price::NewPrice>,
        ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(0) })
        }

        fn find_latest(
            &self,
            stock_id: i32,
        ) -> Pin<Box<dyn Future<Output = Result<Option<Price>, AppError>> + Send + '_>> {
            Box::pin(async move { Ok(self.latest.lock().unwrap().get(&stock_id).cloned()) })
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

    // ---------- helpers ----------

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_service(stock_repo: FakeStockRepo, price_repo: FakePriceRepo) -> PriceService {
        PriceService::new(Arc::new(stock_repo), Arc::new(price_repo))
    }

    // ---------- tests ----------

    #[tokio::test]
    async fn get_latest_returns_not_found_when_stock_is_unknown() {
        let service = make_service(FakeStockRepo::default(), FakePriceRepo::empty());
        let err = service.get_latest(42).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[tokio::test]
    async fn get_latest_returns_not_found_when_stock_has_no_price_yet() {
        let service = make_service(FakeStockRepo::with_stock(1), FakePriceRepo::empty());
        let err = service.get_latest(1).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[tokio::test]
    async fn get_history_rejects_inverted_range() {
        let service = make_service(FakeStockRepo::with_stock(1), FakePriceRepo::empty());
        let err = service
            .get_history(1, date(2026, 4, 24), date(2026, 4, 23))
            .await
            .unwrap_err();
        match err {
            AppError::BadRequest(msg) => assert!(msg.contains("on or before")),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_history_rejects_windows_longer_than_the_cap() {
        // 10 years + 1 day exceeds MAX_HISTORY_WINDOW_DAYS.
        let service = make_service(FakeStockRepo::with_stock(1), FakePriceRepo::empty());
        let from = date(2016, 1, 1);
        let to = from
            .checked_add_days(chrono::Days::new(MAX_HISTORY_WINDOW_DAYS + 1))
            .unwrap();
        let err = service.get_history(1, from, to).await.unwrap_err();
        match err {
            AppError::BadRequest(msg) => assert!(msg.contains("must not exceed")),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_history_returns_not_found_when_stock_is_unknown() {
        let service = make_service(FakeStockRepo::default(), FakePriceRepo::empty());
        let err = service
            .get_history(999, date(2026, 4, 1), date(2026, 4, 24))
            .await
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[tokio::test]
    async fn get_history_returns_empty_when_stock_exists_but_has_no_prices() {
        // Existing stock, no prices → empty Vec, not NotFound (Q: "404 ou empty si aucun prix").
        let service = make_service(FakeStockRepo::with_stock(1), FakePriceRepo::empty());
        let (prices, stock) = service
            .get_history(1, date(2026, 4, 1), date(2026, 4, 24))
            .await
            .expect("valid range on existing stock should not error");
        assert!(prices.is_empty());
        assert_eq!(stock.id, 1);
    }

    #[tokio::test]
    async fn get_latest_returns_the_price_and_stock_together() {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let price = Price {
            id: 1,
            stock_id: 1,
            price_date: date(2026, 4, 24),
            close: Decimal::from_str("130.00").unwrap(),
            source: "yahoo".to_string(),
            fetched_at: date(2026, 4, 24).and_hms_opt(12, 0, 0).unwrap(),
        };
        let service = make_service(
            FakeStockRepo::with_stock(1),
            FakePriceRepo::with_latest(1, price),
        );
        let (returned_price, returned_stock) = service.get_latest(1).await.unwrap();
        assert_eq!(returned_stock.id, 1);
        assert_eq!(returned_stock.currency, "EUR");
        assert_eq!(returned_price.close, Decimal::from_str("130.00").unwrap());
    }
}
