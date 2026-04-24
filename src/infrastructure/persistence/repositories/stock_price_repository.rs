//! Repository for the `stock_prices` table.
//!
//! Stores daily close prices per stock. Upsert-based ingestion on
//! `(stock_id, price_date)` for idempotent scheduler runs.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::upsert::excluded;

use crate::application::error::AppError;
use crate::application::ports::stock_price_repository::StockPriceRepository;
use crate::domain::market::price::{NewPrice, Price};
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::stock_price::{
    NewStockPriceRow, StockPriceRow,
};
use crate::schema::stock_prices;

#[derive(Clone)]
pub struct PgStockPriceRepository {
    db: Db,
}

impl PgStockPriceRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl StockPriceRepository for PgStockPriceRepository {
    fn upsert_many(
        &self,
        prices: Vec<NewPrice>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
        Box::pin(async move {
            if prices.is_empty() {
                return Ok(0);
            }
            self.db
                .exec(move |conn| {
                    let rows: Vec<NewStockPriceRow<'_>> = prices
                        .iter()
                        .map(|p| NewStockPriceRow {
                            stock_id: p.stock_id,
                            price_date: p.price_date,
                            close: p.close,
                            source: p.source.as_str(),
                        })
                        .collect();
                    let count = diesel::insert_into(stock_prices::table)
                        .values(&rows)
                        .on_conflict((stock_prices::stock_id, stock_prices::price_date))
                        .do_update()
                        .set((
                            stock_prices::close.eq(excluded(stock_prices::close)),
                            stock_prices::source.eq(excluded(stock_prices::source)),
                            stock_prices::fetched_at.eq(diesel::dsl::now),
                        ))
                        .execute(conn)?;
                    Ok(count)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_latest(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Price>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row: Option<StockPriceRow> = stock_prices::table
                        .filter(stock_prices::stock_id.eq(stock_id))
                        .order(stock_prices::price_date.desc())
                        .select(StockPriceRow::as_select())
                        .first(conn)
                        .optional()?;
                    Ok(row.map(Price::from))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_latest_batch(
        &self,
        stock_ids: Vec<i32>,
    ) -> Pin<Box<dyn Future<Output = Result<HashMap<i32, Price>, AppError>> + Send + '_>> {
        Box::pin(async move {
            if stock_ids.is_empty() {
                return Ok(HashMap::new());
            }
            self.db
                .exec(move |conn| {
                    let rows: Vec<StockPriceRow> = stock_prices::table
                        .filter(stock_prices::stock_id.eq_any(&stock_ids))
                        .distinct_on(stock_prices::stock_id)
                        .order((
                            stock_prices::stock_id.asc(),
                            stock_prices::price_date.desc(),
                        ))
                        .select(StockPriceRow::as_select())
                        .load(conn)?;
                    Ok(rows
                        .into_iter()
                        .map(Price::from)
                        .map(|p| (p.stock_id, p))
                        .collect())
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_history(
        &self,
        stock_id: i32,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Price>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows: Vec<StockPriceRow> = stock_prices::table
                        .filter(
                            stock_prices::stock_id
                                .eq(stock_id)
                                .and(stock_prices::price_date.between(from, to)),
                        )
                        .order(stock_prices::price_date.asc())
                        .select(StockPriceRow::as_select())
                        .load(conn)?;
                    Ok(rows.into_iter().map(Price::from).collect())
                })
                .await
                .map_err(AppError::from)
        })
    }
}
