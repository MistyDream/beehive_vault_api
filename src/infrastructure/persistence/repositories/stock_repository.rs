use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::stock_repository::StockRepository;
use crate::domain::market::stock::{NewStock, Paginated, Stock, StockFilter, UpdateStock};
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::stock::{NewStockRow, StockRow, UpdateStockRow};
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

    fn search(&self, filter: StockFilter) -> Pin<Box<dyn Future<Output = Result<Paginated<Stock>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    macro_rules! apply_filters {
                        ($query:expr) => {{
                            let mut q = $query;
                            if let Some(ref v) = filter.symbol { q = q.filter(stocks::symbol.ilike(format!("%{v}%"))); }
                            if let Some(ref v) = filter.name { q = q.filter(stocks::name.ilike(format!("%{v}%"))); }
                            if let Some(ref v) = filter.isin { q = q.filter(stocks::isin.eq(v)); }
                            if let Some(ref v) = filter.currency { q = q.filter(stocks::currency.eq(v)); }
                            if let Some(ref v) = filter.market { q = q.filter(stocks::market.eq(v)); }
                            if let Some(ref v) = filter.sector { q = q.filter(stocks::sector.eq(v)); }
                            if let Some(ref v) = filter.industry { q = q.filter(stocks::industry.eq(v)); }
                            if let Some(ref v) = filter.country { q = q.filter(stocks::country.eq(v)); }
                            q
                        }};
                    }

                    let total: i64 = apply_filters!(stocks::table.into_boxed())
                        .count()
                        .get_result(conn)?;

                    let offset = (filter.page - 1) * filter.per_page;
                    let rows = apply_filters!(stocks::table.into_boxed())
                        .select(StockRow::as_select())
                        .order(stocks::symbol.asc())
                        .limit(filter.per_page)
                        .offset(offset)
                        .load(conn)?;

                    Ok(Paginated {
                        data: rows.into_iter().map(Stock::from).collect(),
                        page: filter.page,
                        per_page: filter.per_page,
                        total,
                    })
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn update(&self, isin: String, data: UpdateStock) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        let changeset = UpdateStockRow::from(data);
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = diesel::update(stocks::table.filter(stocks::isin.eq(&isin)))
                        .set(&changeset)
                        .returning(StockRow::as_returning())
                        .get_result(conn)?;
                    Ok(Stock::from(row))
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

    fn delete_by_isin(&self, isin: String) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count = diesel::delete(stocks::table.filter(stocks::isin.eq(&isin)))
                        .execute(conn)?;
                    Ok(count > 0)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
