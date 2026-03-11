//! Repository for the `stocks` table.
//!
//! Provides CRUD operations for stock entities (equities tracked for scoring).

use diesel::prelude::*;

use crate::domain::market::stock::Stock;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::stock::{NewStockRow, StockRow};
use crate::schema::stocks;

/// Fetch a single stock by its primary key.
pub async fn find_by_id(db: &Db, stock_id: i32) -> Result<Stock, DbError> {
    db.exec(move |conn| {
        let row = stocks::table
            .find(stock_id)
            .select(StockRow::as_select())
            .first(conn)?;
        Ok(Stock::from(row))
    })
    .await
}

/// Fetch a single stock by its ticker symbol (e.g. "AAPL").
pub async fn find_by_symbol(db: &Db, symbol: String) -> Result<Stock, DbError> {
    db.exec(move |conn| {
        let row = stocks::table
            .filter(stocks::symbol.eq(&symbol))
            .select(StockRow::as_select())
            .first(conn)?;
        Ok(Stock::from(row))
    })
    .await
}

/// Fetch a single stock by its ISIN code.
pub async fn find_by_isin(db: &Db, isin: String) -> Result<Stock, DbError> {
    db.exec(move |conn| {
        let row = stocks::table
            .filter(stocks::isin.eq(&isin))
            .select(StockRow::as_select())
            .first(conn)?;
        Ok(Stock::from(row))
    })
    .await
}

/// List all stocks, ordered alphabetically by symbol.
pub async fn list_all(db: &Db) -> Result<Vec<Stock>, DbError> {
    db.exec(move |conn| {
        let rows = stocks::table
            .select(StockRow::as_select())
            .order(stocks::symbol.asc())
            .load(conn)?;
        Ok(rows.into_iter().map(Stock::from).collect())
    })
    .await
}

/// Insert a new stock and return the created entity.
pub async fn insert(db: &Db, new: NewStockRow<'static>) -> Result<Stock, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(stocks::table)
            .values(&new)
            .returning(StockRow::as_returning())
            .get_result(conn)?;
        Ok(Stock::from(row))
    })
    .await
}

/// Delete a stock by ID. Returns `true` if a row was actually deleted.
pub async fn delete(db: &Db, stock_id: i32) -> Result<bool, DbError> {
    db.exec(move |conn| {
        let count = diesel::delete(stocks::table.find(stock_id)).execute(conn)?;
        Ok(count > 0)
    })
    .await
}
