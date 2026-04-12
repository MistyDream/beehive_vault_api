//! Repository for the `transactions` table.
//!
//! Provides CRUD operations for portfolio transactions
//! (buy, sell, dividend, fee, split, deposit, withdrawal).

use chrono::NaiveDate;
use diesel::prelude::*;

use crate::domain::wallet::transaction::Transaction;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::transaction::{NewTransactionRow, TransactionRow};
use crate::schema::transactions;

/// Fetch a single transaction by its ID, scoped to a portfolio.
pub async fn find_by_id(
    db: &Db,
    portfolio_id: i32,
    tx_id: i64,
) -> Result<Transaction, DbError> {
    db.exec(move |conn| {
        let row = transactions::table
            .find(tx_id)
            .filter(transactions::portfolio_id.eq(portfolio_id))
            .select(TransactionRow::as_select())
            .first(conn)?;
        Transaction::try_from(row)
    })
    .await
}

/// List all transactions for a portfolio, ordered by date descending.
pub async fn list_by_portfolio(
    db: &Db,
    portfolio_id: i32,
) -> Result<Vec<Transaction>, DbError> {
    db.exec(move |conn| {
        let rows = transactions::table
            .filter(transactions::portfolio_id.eq(portfolio_id))
            .select(TransactionRow::as_select())
            .order((transactions::executed_at.desc(), transactions::id.desc()))
            .load(conn)?;
        rows.into_iter().map(Transaction::try_from).collect()
    })
    .await
}

/// Optional filters for listing transactions.
pub struct TransactionFilter {
    pub transaction_type: Option<String>,
    pub stock_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

/// List transactions for a portfolio with optional filters.
pub async fn list_by_portfolio_filtered(
    db: &Db,
    portfolio_id: i32,
    filters: TransactionFilter,
) -> Result<Vec<Transaction>, DbError> {
    db.exec(move |conn| {
        let mut query = transactions::table
            .filter(transactions::portfolio_id.eq(portfolio_id))
            .into_boxed();

        if let Some(tx_type) = &filters.transaction_type {
            query = query.filter(transactions::transaction_type.eq(tx_type));
        }
        if let Some(sid) = filters.stock_id {
            query = query.filter(transactions::stock_id.eq(sid));
        }
        if let Some(from) = filters.from_date {
            query = query.filter(transactions::executed_at.ge(from));
        }
        if let Some(to) = filters.to_date {
            query = query.filter(transactions::executed_at.le(to));
        }

        let rows = query
            .select(TransactionRow::as_select())
            .order((transactions::executed_at.desc(), transactions::id.desc()))
            .load(conn)?;
        rows.into_iter().map(Transaction::try_from).collect()
    })
    .await
}

/// Insert a new transaction and return the created entity.
pub async fn insert(
    db: &Db,
    new: NewTransactionRow<'static>,
) -> Result<Transaction, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(transactions::table)
            .values(&new)
            .returning(TransactionRow::as_returning())
            .get_result(conn)?;
        Transaction::try_from(row)
    })
    .await
}

/// Replace all mutable fields of an existing transaction.
pub async fn update(
    db: &Db,
    portfolio_id: i32,
    tx_id: i64,
    new: NewTransactionRow<'static>,
) -> Result<Transaction, DbError> {
    db.exec(move |conn| {
        let row = diesel::update(
            transactions::table
                .find(tx_id)
                .filter(transactions::portfolio_id.eq(portfolio_id)),
        )
        .set((
            transactions::stock_id.eq(new.stock_id),
            transactions::transaction_type.eq(new.transaction_type),
            transactions::executed_at.eq(new.executed_at),
            transactions::quantity.eq(new.quantity),
            transactions::unit_price.eq(new.unit_price),
            transactions::amount.eq(new.amount),
            transactions::fees.eq(new.fees),
            transactions::tax.eq(new.tax),
            transactions::split_from.eq(new.split_from),
            transactions::split_to.eq(new.split_to),
            transactions::currency.eq(new.currency),
            transactions::exchange_rate.eq(new.exchange_rate),
            transactions::notes.eq(new.notes),
        ))
        .returning(TransactionRow::as_returning())
        .get_result(conn)?;
        Transaction::try_from(row)
    })
    .await
}

/// Delete a transaction by ID, scoped to a portfolio. Returns `true` if deleted.
pub async fn delete(
    db: &Db,
    portfolio_id: i32,
    tx_id: i64,
) -> Result<bool, DbError> {
    db.exec(move |conn| {
        let count = diesel::delete(
            transactions::table
                .find(tx_id)
                .filter(transactions::portfolio_id.eq(portfolio_id)),
        )
        .execute(conn)?;
        Ok(count > 0)
    })
    .await
}
