use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::stock::{NewStock, Stock};
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
}

impl StockRepository for PgStockRepository {
    fn find_by_id(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = stocks::table
                        .find(stock_id)
                        .select(StockRow::as_select())
                        .first(conn)?;
                    Ok(Stock::from(row))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_by_symbol(&self, symbol: String) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = stocks::table
                        .filter(stocks::symbol.eq(&symbol))
                        .select(StockRow::as_select())
                        .first(conn)?;
                    Ok(Stock::from(row))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_by_isin(&self, isin: String) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = stocks::table
                        .filter(stocks::isin.eq(&isin))
                        .select(StockRow::as_select())
                        .first(conn)?;
                    Ok(Stock::from(row))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows = stocks::table
                        .select(StockRow::as_select())
                        .order(stocks::symbol.asc())
                        .load(conn)?;
                    Ok(rows.into_iter().map(Stock::from).collect())
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn insert(&self, new: NewStock) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        let row_data = NewStockRow::from(new);
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = diesel::insert_into(stocks::table)
                        .values(&row_data)
                        .returning(StockRow::as_returning())
                        .get_result(conn)?;
                    Ok(Stock::from(row))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn delete(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count = diesel::delete(stocks::table.find(stock_id)).execute(conn)?;
                    Ok(count > 0)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
