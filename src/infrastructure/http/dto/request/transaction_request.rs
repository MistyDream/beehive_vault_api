use chrono::NaiveDate;
use garde::Validate;
use serde::Deserialize;

use crate::application::error::AppError;
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::{NewTransaction, UpdateTransaction};

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreateTransactionRequest {
    #[garde(inner(range(min = 1)))]
    pub stock_id: Option<i32>,
    #[garde(pattern(r"^(buy|sell|dividend|fee|split|deposit|withdrawal)$"))]
    pub transaction_type: String,
    #[garde(skip)]
    pub executed_at: NaiveDate,
    #[garde(inner(range(min = 0.0, max = 1e12)))]
    pub quantity: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e12)))]
    pub unit_price: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e12)))]
    pub amount: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e9)))]
    pub fees: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e9)))]
    pub tax: Option<f64>,
    #[garde(inner(range(min = 1)))]
    pub split_from: Option<i32>,
    #[garde(inner(range(min = 1)))]
    pub split_to: Option<i32>,
    #[garde(length(min = 3, max = 3))]
    pub currency: String,
    #[garde(inner(range(min = 0.000001, max = 1e6)))]
    pub exchange_rate: Option<f64>,
    #[garde(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdateTransactionRequest {
    #[garde(inner(range(min = 1)))]
    pub stock_id: Option<i32>,
    #[garde(pattern(r"^(buy|sell|dividend|fee|split|deposit|withdrawal)$"))]
    pub transaction_type: String,
    #[garde(skip)]
    pub executed_at: NaiveDate,
    #[garde(inner(range(min = 0.0, max = 1e12)))]
    pub quantity: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e12)))]
    pub unit_price: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e12)))]
    pub amount: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e9)))]
    pub fees: Option<f64>,
    #[garde(inner(range(min = 0.0, max = 1e9)))]
    pub tax: Option<f64>,
    #[garde(inner(range(min = 1)))]
    pub split_from: Option<i32>,
    #[garde(inner(range(min = 1)))]
    pub split_to: Option<i32>,
    #[garde(length(min = 3, max = 3))]
    pub currency: String,
    #[garde(inner(range(min = 0.000001, max = 1e6)))]
    pub exchange_rate: Option<f64>,
    #[garde(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct TransactionQueryParams {
    #[garde(inner(pattern(r"^(buy|sell|dividend|fee|split|deposit|withdrawal)(,(buy|sell|dividend|fee|split|deposit|withdrawal))*$")))]
    pub transaction_types: Option<String>,
    #[garde(inner(range(min = 1)))]
    pub stock_id: Option<i32>,
    #[garde(skip)]
    pub from_date: Option<NaiveDate>,
    #[garde(skip)]
    pub to_date: Option<NaiveDate>,
    #[garde(inner(pattern(r"^(amount|transaction_type|executed_at)$")))]
    pub sort_by: Option<String>,
    #[garde(inner(pattern(r"^(asc|desc)$")))]
    pub sort_dir: Option<String>,
    #[garde(inner(range(min = 1)))]
    pub page: Option<u32>,
    #[garde(inner(range(min = 1, max = 100)))]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct TransactionStatsQueryParams {
    #[garde(inner(range(min = 1)))]
    pub stock_id: Option<i32>,
    #[garde(skip)]
    pub from_date: Option<NaiveDate>,
    #[garde(skip)]
    pub to_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct PerformanceQueryParams {
    #[garde(skip)]
    pub from_date: Option<NaiveDate>,
    #[garde(skip)]
    pub to_date: Option<NaiveDate>,
}

impl CreateTransactionRequest {
    pub fn into_new_transaction(self, portfolio_id: i32) -> Result<NewTransaction, AppError> {
        Ok(NewTransaction {
            portfolio_id,
            stock_id: self.stock_id,
            transaction_type: TransactionType::try_from(self.transaction_type.as_str())
                .map_err(AppError::BadRequest)?,
            executed_at: self.executed_at,
            quantity: self.quantity,
            unit_price: self.unit_price,
            amount: self.amount,
            fees: self.fees.unwrap_or(0.0),
            tax: self.tax.unwrap_or(0.0),
            split_from: self.split_from,
            split_to: self.split_to,
            currency: self.currency,
            exchange_rate: self.exchange_rate.unwrap_or(1.0),
            notes: self.notes,
        })
    }
}

impl UpdateTransactionRequest {
    pub fn into_update_transaction(self, portfolio_id: i32) -> Result<UpdateTransaction, AppError> {
        Ok(NewTransaction {
            portfolio_id,
            stock_id: self.stock_id,
            transaction_type: TransactionType::try_from(self.transaction_type.as_str())
                .map_err(AppError::BadRequest)?,
            executed_at: self.executed_at,
            quantity: self.quantity,
            unit_price: self.unit_price,
            amount: self.amount,
            fees: self.fees.unwrap_or(0.0),
            tax: self.tax.unwrap_or(0.0),
            split_from: self.split_from,
            split_to: self.split_to,
            currency: self.currency,
            exchange_rate: self.exchange_rate.unwrap_or(1.0),
            notes: self.notes,
        })
    }
}
