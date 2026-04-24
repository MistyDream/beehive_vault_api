//! Repository for the `stocks` table.
//!
//! Provides CRUD operations for stock entities (equities tracked for scoring).

use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::enums::MarketRegion;
use crate::domain::market::stock::Stock;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::stock::{NewStockRow, StockRow};
use crate::schema::stocks;

#[derive(Clone)]
pub struct PgStockRepository {
    db: Db,
}

impl PgStockRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Insert a new stock and return the created entity.
    pub async fn insert(&self, new: NewStockRow<'static>) -> Result<Stock, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(stocks::table)
                    .values(&new)
                    .returning(StockRow::as_returning())
                    .get_result(conn)?;
                Stock::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }
}

impl StockRepository for PgStockRepository {
    fn find_by_id(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = stocks::table
                        .find(stock_id)
                        .select(StockRow::as_select())
                        .first(conn)?;
                    Stock::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_by_ids(
        &self,
        stock_ids: Vec<i32>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            if stock_ids.is_empty() {
                return Ok(Vec::new());
            }
            self.db
                .exec(move |conn| {
                    let rows = stocks::table
                        .filter(stocks::id.eq_any(&stock_ids))
                        .select(StockRow::as_select())
                        .load(conn)?;
                    rows.into_iter().map(Stock::try_from).collect()
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_by_symbol(
        &self,
        symbol: String,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = stocks::table
                        .filter(stocks::symbol.eq(&symbol))
                        .select(StockRow::as_select())
                        .first(conn)?;
                    Stock::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_by_isin(
        &self,
        isin: String,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = stocks::table
                        .filter(stocks::isin.eq(&isin))
                        .select(StockRow::as_select())
                        .first(conn)?;
                    Stock::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows = stocks::table
                        .select(StockRow::as_select())
                        .order(stocks::symbol.asc())
                        .load(conn)?;
                    rows.into_iter().map(Stock::try_from).collect()
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_by_region(
        &self,
        region: MarketRegion,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            let region_str = region.as_str();
            self.db
                .exec(move |conn| {
                    let rows = stocks::table
                        .filter(stocks::market_region.eq(region_str))
                        .select(StockRow::as_select())
                        .order(stocks::symbol.asc())
                        .load(conn)?;
                    rows.into_iter().map(Stock::try_from).collect()
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn delete(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count =
                        diesel::delete(stocks::table.find(stock_id)).execute(conn)?;
                    Ok(count > 0)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
