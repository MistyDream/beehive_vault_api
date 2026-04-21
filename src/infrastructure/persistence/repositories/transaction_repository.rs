//! Repository for the `transactions` table.
//!
//! Provides CRUD operations for portfolio transactions
//! (buy, sell, dividend, fee, split, deposit, withdrawal).

use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::transaction_repository::TransactionRepository;
use crate::domain::wallet::transaction::{
    NewTransaction, Transaction, TransactionFilter, UpdateTransaction,
};
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::transaction::{NewTransactionRow, TransactionRow};
use crate::schema::transactions;

#[derive(Clone)]
pub struct PgTransactionRepository {
    db: Db,
}

impl PgTransactionRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl TransactionRepository for PgTransactionRepository {
    fn find_by_id(
        &self,
        portfolio_id: i32,
        tx_id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = transactions::table
                        .find(tx_id)
                        .filter(transactions::portfolio_id.eq(portfolio_id))
                        .select(TransactionRow::as_select())
                        .first(conn)?;
                    Transaction::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_by_portfolio(
        &self,
        portfolio_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows = transactions::table
                        .filter(transactions::portfolio_id.eq(portfolio_id))
                        .select(TransactionRow::as_select())
                        .order((transactions::executed_at.desc(), transactions::id.desc()))
                        .load(conn)?;
                    rows.into_iter().map(Transaction::try_from).collect()
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_by_portfolio_chronological(
        &self,
        portfolio_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows = transactions::table
                        .filter(transactions::portfolio_id.eq(portfolio_id))
                        .select(TransactionRow::as_select())
                        .order((transactions::executed_at.asc(), transactions::id.asc()))
                        .load(conn)?;
                    rows.into_iter().map(Transaction::try_from).collect()
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn list_by_portfolio_filtered(
        &self,
        portfolio_id: i32,
        filters: TransactionFilter,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Transaction>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let mut query = transactions::table
                        .filter(transactions::portfolio_id.eq(portfolio_id))
                        .into_boxed();

                    if !filters.transaction_types.is_empty() {
                        query = query.filter(
                            transactions::transaction_type.eq_any(filters.transaction_types.clone()),
                        );
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
                .map_err(AppError::from)
        })
    }

    fn insert(
        &self,
        new: NewTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row_data = NewTransactionRow {
                        portfolio_id: new.portfolio_id,
                        stock_id: new.stock_id,
                        transaction_type: new.transaction_type.as_str(),
                        executed_at: new.executed_at,
                        quantity: new.quantity,
                        unit_price: new.unit_price,
                        amount: new.amount,
                        fees: new.fees,
                        tax: new.tax,
                        split_from: new.split_from,
                        split_to: new.split_to,
                        currency: &new.currency,
                        exchange_rate: new.exchange_rate,
                        notes: new.notes.as_deref(),
                    };
                    let row = diesel::insert_into(transactions::table)
                        .values(&row_data)
                        .returning(TransactionRow::as_returning())
                        .get_result(conn)?;
                    Transaction::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn update(
        &self,
        portfolio_id: i32,
        tx_id: i64,
        data: UpdateTransaction,
    ) -> Pin<Box<dyn Future<Output = Result<Transaction, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = diesel::update(
                        transactions::table
                            .find(tx_id)
                            .filter(transactions::portfolio_id.eq(portfolio_id)),
                    )
                    .set((
                        transactions::stock_id.eq(data.stock_id),
                        transactions::transaction_type.eq(data.transaction_type.as_str()),
                        transactions::executed_at.eq(data.executed_at),
                        transactions::quantity.eq(data.quantity),
                        transactions::unit_price.eq(data.unit_price),
                        transactions::amount.eq(data.amount),
                        transactions::fees.eq(data.fees),
                        transactions::tax.eq(data.tax),
                        transactions::split_from.eq(data.split_from),
                        transactions::split_to.eq(data.split_to),
                        transactions::currency.eq(&data.currency),
                        transactions::exchange_rate.eq(data.exchange_rate),
                        transactions::notes.eq(&data.notes),
                    ))
                    .returning(TransactionRow::as_returning())
                    .get_result(conn)?;
                    Transaction::try_from(row)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn delete(
        &self,
        portfolio_id: i32,
        tx_id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count = diesel::delete(
                        transactions::table
                            .find(tx_id)
                            .filter(transactions::portfolio_id.eq(portfolio_id)),
                    )
                    .execute(conn)?;
                    Ok(count > 0)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
