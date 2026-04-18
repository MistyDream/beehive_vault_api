use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::wallet::enums::TransactionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
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

pub struct NewTransaction {
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

pub type UpdateTransaction = NewTransaction;

pub struct TransactionFilter {
    pub transaction_type: Option<String>,
    pub stock_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}
