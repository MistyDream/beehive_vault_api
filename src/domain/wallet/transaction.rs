use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::wallet::enums::TransactionType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub portfolio_id: Uuid,
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
    pub portfolio_id: Uuid,
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

type FieldCheck = fn(&NewTransaction) -> Option<&'static str>;

const STOCK_ID: FieldCheck = |t| t.stock_id.is_none().then_some("stock_id");
const QUANTITY: FieldCheck = |t| t.quantity.is_none().then_some("quantity");
const UNIT_PRICE: FieldCheck = |t| t.unit_price.is_none().then_some("unit_price");
const AMOUNT: FieldCheck = |t| t.amount.is_none().then_some("amount");
const SPLIT_FROM_TO: FieldCheck = |t| {
    (t.split_from.is_none() || t.split_to.is_none()).then_some("split_from and split_to")
};

fn required_fields(tt: &TransactionType) -> &'static [FieldCheck] {
    match tt {
        TransactionType::Buy | TransactionType::Sell => &[STOCK_ID, QUANTITY, UNIT_PRICE],
        TransactionType::Dividend => &[STOCK_ID, AMOUNT],
        TransactionType::Fee => &[AMOUNT],
        TransactionType::Split => &[STOCK_ID, SPLIT_FROM_TO],
        TransactionType::Deposit | TransactionType::Withdrawal => &[AMOUNT],
    }
}

impl NewTransaction {
    pub fn check_invariants(&self) -> Result<(), String> {
        for check in required_fields(&self.transaction_type) {
            if let Some(field) = check(self) {
                return Err(format!("{} requires {}", self.transaction_type.as_str(), field));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct TransactionFilter {
    pub transaction_types: Vec<TransactionType>,
    pub stock_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}
