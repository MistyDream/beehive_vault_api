use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    features::{accounts::AccountKind, categories::CategoryKind},
    types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId},
};

use super::domain::{
    TransactionAmounts, TransactionLabel, TransactionNature, TransactionNote, TransactionSource,
};

pub struct AccountSummary {
    pub id: AccountId,
    pub name: String,
    pub kind: AccountKind,
    pub archived: bool,
}

pub struct CategorySummary {
    pub id: CategoryId,
    pub name: String,
    pub kind: CategoryKind,
    pub archived: bool,
}

pub struct TransactionOperation {
    pub id: TransactionId,
    pub household_id: HouseholdId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub nature: TransactionNature,
    pub amounts: TransactionAmounts,
    pub account: AccountSummary,
    pub category: Option<CategorySummary>,
    pub origin: TransactionSource,
    pub note: Option<TransactionNote>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TransferMovementOperation {
    pub transaction_id: TransactionId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub account_amount: super::domain::TransactionAmount,
    pub account: AccountSummary,
    pub note: Option<TransactionNote>,
}

pub struct TransferOperation {
    pub id: TransferId,
    pub household_id: HouseholdId,
    pub booking_date: NaiveDate,
    pub amount: super::domain::TransactionNominalAmount,
    pub source: TransferMovementOperation,
    pub destination: TransferMovementOperation,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum Operation {
    Transaction(TransactionOperation),
    Transfer(TransferOperation),
}

pub struct OperationPage {
    pub items: Vec<Operation>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}
