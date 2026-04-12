use chrono::NaiveDate;
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreateTransactionRequest {
    #[garde(skip)]
    pub stock_id: Option<i32>,
    #[garde(pattern(r"^(buy|sell|dividend|fee|split|deposit|withdrawal)$"))]
    pub transaction_type: String,
    #[garde(skip)]
    pub executed_at: NaiveDate,
    #[garde(skip)]
    pub quantity: Option<f64>,
    #[garde(skip)]
    pub unit_price: Option<f64>,
    #[garde(skip)]
    pub amount: Option<f64>,
    #[garde(skip)]
    pub fees: Option<f64>,
    #[garde(skip)]
    pub tax: Option<f64>,
    #[garde(skip)]
    pub split_from: Option<i32>,
    #[garde(skip)]
    pub split_to: Option<i32>,
    #[garde(length(min = 3, max = 3))]
    pub currency: String,
    #[garde(skip)]
    pub exchange_rate: Option<f64>,
    #[garde(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdateTransactionRequest {
    #[garde(skip)]
    pub stock_id: Option<i32>,
    #[garde(pattern(r"^(buy|sell|dividend|fee|split|deposit|withdrawal)$"))]
    pub transaction_type: String,
    #[garde(skip)]
    pub executed_at: NaiveDate,
    #[garde(skip)]
    pub quantity: Option<f64>,
    #[garde(skip)]
    pub unit_price: Option<f64>,
    #[garde(skip)]
    pub amount: Option<f64>,
    #[garde(skip)]
    pub fees: Option<f64>,
    #[garde(skip)]
    pub tax: Option<f64>,
    #[garde(skip)]
    pub split_from: Option<i32>,
    #[garde(skip)]
    pub split_to: Option<i32>,
    #[garde(length(min = 3, max = 3))]
    pub currency: String,
    #[garde(skip)]
    pub exchange_rate: Option<f64>,
    #[garde(length(max = 500))]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub transaction_type: Option<String>,
    pub stock_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct PerformanceQueryParams {
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}
