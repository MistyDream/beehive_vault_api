//! Repository for the `portfolios` table.
//!
//! Provides CRUD operations for investment portfolios.

use diesel::prelude::*;

use crate::domain::wallet::portfolio::Portfolio;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::portfolio::{NewPortfolioRow, PortfolioRow};
use crate::schema::portfolios;

/// Fetch a single portfolio by its primary key.
pub async fn find_by_id(db: &Db, portfolio_id: i32) -> Result<Portfolio, DbError> {
    db.exec(move |conn| {
        let row = portfolios::table
            .find(portfolio_id)
            .select(PortfolioRow::as_select())
            .first(conn)?;
        Portfolio::try_from(row)
    })
    .await
}

/// List all portfolios, ordered alphabetically by name.
pub async fn list_all(db: &Db) -> Result<Vec<Portfolio>, DbError> {
    db.exec(move |conn| {
        let rows = portfolios::table
            .select(PortfolioRow::as_select())
            .order(portfolios::name.asc())
            .load(conn)?;
        rows.into_iter().map(Portfolio::try_from).collect()
    })
    .await
}

/// Insert a new portfolio and return the created entity.
pub async fn insert(db: &Db, new: NewPortfolioRow<'static>) -> Result<Portfolio, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(portfolios::table)
            .values(&new)
            .returning(PortfolioRow::as_returning())
            .get_result(conn)?;
        Portfolio::try_from(row)
    })
    .await
}

/// Replace all mutable fields of an existing portfolio.
/// The `updated_at` timestamp is managed automatically by the DB trigger.
pub async fn update(
    db: &Db,
    portfolio_id: i32,
    new: NewPortfolioRow<'static>,
) -> Result<Portfolio, DbError> {
    db.exec(move |conn| {
        let row = diesel::update(portfolios::table.find(portfolio_id))
            .set((
                portfolios::name.eq(new.name),
                portfolios::kind.eq(new.kind),
                portfolios::currency.eq(new.currency),
                portfolios::description.eq(new.description),
            ))
            .returning(PortfolioRow::as_returning())
            .get_result(conn)?;
        Portfolio::try_from(row)
    })
    .await
}

/// Delete a portfolio by ID. Returns `true` if a row was actually deleted.
pub async fn delete(db: &Db, portfolio_id: i32) -> Result<bool, DbError> {
    db.exec(move |conn| {
        let count = diesel::delete(portfolios::table.find(portfolio_id)).execute(conn)?;
        Ok(count > 0)
    })
    .await
}
