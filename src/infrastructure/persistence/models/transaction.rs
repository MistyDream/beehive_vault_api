use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;
use crate::infrastructure::persistence::error::DbError;
use crate::schema::transactions;

#[derive(Queryable, Selectable)]
#[diesel(table_name = transactions)]
pub struct TransactionRow {
    pub id: i64,
    pub portfolio_id: i32,
    pub stock_id: Option<i32>,
    pub transaction_type: String,
    pub executed_at: NaiveDate,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub amount: Option<f64>,
    pub fees: f64,
    pub tax: f64,
    pub split_from: Option<i32>,
    pub split_to: Option<i32>,
    pub currency: String,
    pub exchange_rate: f64,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransactionRow<'a> {
    pub portfolio_id: i32,
    pub stock_id: Option<i32>,
    pub transaction_type: &'a str,
    pub executed_at: NaiveDate,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub amount: Option<f64>,
    pub fees: f64,
    pub tax: f64,
    pub split_from: Option<i32>,
    pub split_to: Option<i32>,
    pub currency: &'a str,
    pub exchange_rate: f64,
    pub notes: Option<&'a str>,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = DbError;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        Ok(Transaction {
            id: row.id,
            portfolio_id: row.portfolio_id,
            stock_id: row.stock_id,
            transaction_type: TransactionType::try_from(row.transaction_type.as_str())
                .map_err(DbError::Conversion)?,
            executed_at: row.executed_at,
            quantity: row.quantity,
            unit_price: row.unit_price,
            amount: row.amount,
            fees: row.fees,
            tax: row.tax,
            split_from: row.split_from,
            split_to: row.split_to,
            currency: row.currency,
            exchange_rate: row.exchange_rate,
            notes: row.notes,
        })
    }
}
