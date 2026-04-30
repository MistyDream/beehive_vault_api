use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Serialize;
use uuid::Uuid;

use crate::application::services::transaction_service::TransactionStats;
use crate::domain::market::stock::Stock;
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::Transaction;

#[derive(Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub portfolio_id: Uuid,
    pub stock: Option<Stock>,
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

impl TransactionResponse {
    pub fn from_transaction(t: Transaction, stocks_by_id: &HashMap<i32, Stock>) -> Self {
        let stock = t.stock_id.and_then(|id| stocks_by_id.get(&id).cloned());
        TransactionResponse {
            id: t.id,
            portfolio_id: t.portfolio_id,
            stock,
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

#[derive(Serialize)]
pub struct TransactionStatsByType {
    pub buy: u64,
    pub sell: u64,
    pub dividend: u64,
    pub fee: u64,
    pub split: u64,
    pub deposit: u64,
    pub withdrawal: u64,
}

#[derive(Serialize)]
pub struct TransactionStatsResponse {
    pub total: u64,
    pub by_type: TransactionStatsByType,
}

impl From<TransactionStats> for TransactionStatsResponse {
    fn from(s: TransactionStats) -> Self {
        TransactionStatsResponse {
            total: s.total,
            by_type: TransactionStatsByType {
                buy: s.buy,
                sell: s.sell,
                dividend: s.dividend,
                fee: s.fee,
                split: s.split,
                deposit: s.deposit,
                withdrawal: s.withdrawal,
            },
        }
    }
}
