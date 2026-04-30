//! Repository for the `stocks` table.
//!
//! Provides CRUD operations for stock entities (equities tracked for scoring).

use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

use crate::application::error::AppError;
use crate::application::ports::stock_repository::{StockRepository, StockSearchResult};
use crate::domain::market::enums::MarketRegion;
use crate::domain::market::stock::{NewStock, Stock, UpdateStock};
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::stock::{NewStockRow, StockRow};
use crate::schema::stocks;

/// Names of the Postgres constraints on `stocks` and the FKs targeting it.
/// Used to map DB-level integrity violations to friendly 409 messages.
const SYMBOL_UNIQUE: &str = "stocks_symbol_unique";
const ISIN_UNIQUE: &str = "stocks_isin_unique";
const TRANSACTIONS_STOCK_FK: &str = "transactions_stock_id_fkey";

fn map_constraint_violation(err: DbError) -> AppError {
    if let DbError::Diesel(DieselError::DatabaseError(kind, info)) = &err {
        match (kind, info.constraint_name()) {
            (DatabaseErrorKind::UniqueViolation, Some(SYMBOL_UNIQUE)) => {
                return AppError::Conflict("symbol already exists".to_string());
            }
            (DatabaseErrorKind::UniqueViolation, Some(ISIN_UNIQUE)) => {
                return AppError::Conflict("isin already exists".to_string());
            }
            (DatabaseErrorKind::ForeignKeyViolation, Some(TRANSACTIONS_STOCK_FK)) => {
                return AppError::Conflict(
                    "stock is referenced by transactions and cannot be deleted".to_string(),
                );
            }
            // Surface unmapped integrity violations in logs so a future renamed
            // or newly-added constraint doesn't degrade silently to a 500.
            (DatabaseErrorKind::UniqueViolation | DatabaseErrorKind::ForeignKeyViolation, name) => {
                tracing::warn!(
                    constraint = ?name,
                    kind = ?kind,
                    "unmapped DB integrity violation on stocks — surfacing as 500"
                );
            }
            _ => {}
        }
    }
    AppError::from(err)
}

/// Defence-in-depth cap so a single unselective query cannot scan the table.
const SEARCH_LIMIT: i64 = 50;

/// Escape Postgres LIKE wildcards so `?q=%` or `?q=_` cannot widen the pattern.
/// Backslash is escaped first because it is itself the LIKE escape character.
fn escape_like_pattern(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

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

    fn search(
        &self,
        query: String,
    ) -> Pin<Box<dyn Future<Output = Result<StockSearchResult, AppError>> + Send + '_>> {
        Box::pin(async move {
            let pattern = format!("%{}%", escape_like_pattern(&query));
            self.db
                .exec(move |conn| {
                    // LIMIT+1 probe lets us detect truncation without a COUNT(*).
                    let mut rows = stocks::table
                        .filter(
                            stocks::symbol
                                .ilike(&pattern)
                                .or(stocks::name.ilike(&pattern))
                                .or(stocks::isin.ilike(&pattern)),
                        )
                        .select(StockRow::as_select())
                        .order(stocks::symbol.asc())
                        .limit(SEARCH_LIMIT + 1)
                        .load::<StockRow>(conn)?;
                    let truncated = rows.len() as i64 > SEARCH_LIMIT;
                    if truncated {
                        rows.truncate(SEARCH_LIMIT as usize);
                    }
                    let items: Vec<Stock> = rows
                        .into_iter()
                        .map(Stock::try_from)
                        .collect::<Result<_, _>>()?;
                    Ok(StockSearchResult { items, truncated })
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

    fn insert(
        &self,
        new: NewStock,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row_data = NewStockRow {
                        symbol: &new.symbol,
                        name: &new.name,
                        isin: &new.isin,
                        currency: &new.currency,
                        market_region: new.market_region.as_str(),
                        market: new.market.as_deref(),
                        sector: new.sector.as_deref(),
                        industry: new.industry.as_deref(),
                        country: new.country.as_deref(),
                    };
                    let row = diesel::insert_into(stocks::table)
                        .values(&row_data)
                        .returning(StockRow::as_returning())
                        .get_result(conn)?;
                    Stock::try_from(row).map_err(DbError::from)
                })
                .await
                .map_err(map_constraint_violation)
        })
    }

    fn update(
        &self,
        stock_id: i32,
        data: UpdateStock,
    ) -> Pin<Box<dyn Future<Output = Result<Stock, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = diesel::update(stocks::table.find(stock_id))
                        .set((
                            stocks::symbol.eq(&data.symbol),
                            stocks::name.eq(&data.name),
                            stocks::isin.eq(&data.isin),
                            stocks::currency.eq(&data.currency),
                            stocks::market_region.eq(data.market_region.as_str()),
                            stocks::market.eq(&data.market),
                            stocks::sector.eq(&data.sector),
                            stocks::industry.eq(&data.industry),
                            stocks::country.eq(&data.country),
                            stocks::updated_at.eq(diesel::dsl::now),
                        ))
                        .returning(StockRow::as_returning())
                        .get_result(conn)?;
                    Stock::try_from(row).map_err(DbError::from)
                })
                .await
                .map_err(map_constraint_violation)
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
                .map_err(map_constraint_violation)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::escape_like_pattern;

    #[test]
    fn escape_doubles_backslash_first_so_added_backslashes_are_not_re_escaped() {
        assert_eq!(escape_like_pattern("a\\b"), "a\\\\b");
    }

    #[test]
    fn escape_neutralises_percent_and_underscore_wildcards() {
        assert_eq!(escape_like_pattern("50%"), "50\\%");
        assert_eq!(escape_like_pattern("a_b"), "a\\_b");
    }

    #[test]
    fn escape_handles_combined_metacharacters() {
        assert_eq!(escape_like_pattern("100%_off\\"), "100\\%\\_off\\\\");
    }

    #[test]
    fn escape_passes_through_normal_characters() {
        assert_eq!(escape_like_pattern("AAPL"), "AAPL");
    }
}
