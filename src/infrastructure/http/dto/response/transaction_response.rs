use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;

#[derive(Serialize)]
pub struct TransactionResponse {
    pub id: i64,
    pub portfolio_id: i32,
    pub stock_id: Option<i32>,
    pub transaction_type: TransactionType,
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
}

impl From<Transaction> for TransactionResponse {
    fn from(t: Transaction) -> Self {
        TransactionResponse {
            id: t.id,
            portfolio_id: t.portfolio_id,
            stock_id: t.stock_id,
            transaction_type: t.transaction_type,
            executed_at: t.executed_at,
            quantity: t.quantity,
            unit_price: t.unit_price,
            amount: t.amount,
            fees: t.fees,
            tax: t.tax,
            split_from: t.split_from,
            split_to: t.split_to,
            currency: t.currency,
            exchange_rate: t.exchange_rate,
            notes: t.notes,
        }
    }
}
