//! Repository for the `portfolios` table.
//!
//! Provides CRUD operations for investment portfolios.

use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::domain::wallet::portfolio::{NewPortfolio, Portfolio, UpdatePortfolio};
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::portfolio::{NewPortfolioRow, PortfolioRow};
use crate::schema::portfolios;

#[derive(Clone)]
pub struct PgPortfolioRepository {
    db: Db,
}

impl PgPortfolioRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl PortfolioRepository for PgPortfolioRepository {
    fn find_by_id(
        &self,
        id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = portfolios::table
                        .find(id)
                        .select(PortfolioRow::as_select())
                        .first(conn)?;
                    Portfolio::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_all(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Portfolio>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows = portfolios::table
                        .select(PortfolioRow::as_select())
                        .order(portfolios::name.asc())
                        .load(conn)?;
                    rows.into_iter().map(Portfolio::try_from).collect()
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn insert(
        &self,
        new: NewPortfolio,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row_data = NewPortfolioRow {
                        name: &new.name,
                        kind: new.kind.as_str(),
                        currency: &new.currency,
                        description: new.description.as_deref(),
                    };
                    let row = diesel::insert_into(portfolios::table)
                        .values(&row_data)
                        .returning(PortfolioRow::as_returning())
                        .get_result(conn)?;
                    Portfolio::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn update(
        &self,
        id: i32,
        data: UpdatePortfolio,
    ) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = diesel::update(portfolios::table.find(id))
                        .set((
                            portfolios::name.eq(&data.name),
                            portfolios::kind.eq(data.kind.as_str()),
                            portfolios::currency.eq(&data.currency),
                            portfolios::description.eq(&data.description),
                        ))
                        .returning(PortfolioRow::as_returning())
                        .get_result(conn)?;
                    Portfolio::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn delete(
        &self,
        id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count =
                        diesel::delete(portfolios::table.find(id)).execute(conn)?;
                    Ok(count > 0)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
