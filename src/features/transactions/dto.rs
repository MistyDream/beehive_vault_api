use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    features::{accounts::AccountKind, categories::CategoryKind},
    pagination::{Pagination, PaginationError, PaginationQuery},
    types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId},
    update::FieldUpdate,
};

use super::{
    domain::{TransactionEffect, TransactionNature, TransactionSource},
    operations::{
        AccountSummary, CategorySummary, Operation, OperationPage, TransactionOperation,
        TransferMovementOperation, TransferOperation,
    },
    service::{CreateTransactionCommand, ListTransactionsCommand, UpdateTransactionCommand},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionRequest {
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    amount: Decimal,
    effect: TransactionEffect,
    nature: TransactionNature,
    category_id: Option<CategoryId>,
    note: Option<String>,
}

impl From<CreateTransactionRequest> for CreateTransactionCommand {
    fn from(request: CreateTransactionRequest) -> Self {
        Self {
            account_id: request.account_id,
            booking_date: request.booking_date,
            label: request.label,
            amount: request.amount,
            effect: request.effect,
            nature: request.nature,
            category_id: request.category_id,
            note: request.note,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTransactionsQuery {
    account_id: Option<AccountId>,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    nature: Option<TransactionNature>,
    category_id: Option<CategoryId>,
    #[serde(default)]
    uncategorized: bool,
    source: Option<TransactionSource>,
    search: Option<String>,
    #[serde(flatten)]
    pagination: PaginationQuery,
}

impl TryFrom<ListTransactionsQuery> for ListTransactionsCommand {
    type Error = PaginationError;

    fn try_from(query: ListTransactionsQuery) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: query.account_id,
            date_from: query.date_from,
            date_to: query.date_to,
            nature: query.nature,
            category_id: query.category_id,
            uncategorized: query.uncategorized,
            source: query.source,
            search: query.search,
            pagination: Pagination::try_from(query.pagination)?,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransactionRequest {
    account_id: Option<AccountId>,
    booking_date: Option<NaiveDate>,
    label: Option<String>,
    amount: Option<Decimal>,
    effect: Option<TransactionEffect>,
    nature: Option<TransactionNature>,
    #[serde(default)]
    category_id: FieldUpdate<CategoryId>,
    #[serde(default)]
    note: FieldUpdate<String>,
}

impl From<UpdateTransactionRequest> for UpdateTransactionCommand {
    fn from(request: UpdateTransactionRequest) -> Self {
        Self {
            account_id: request.account_id,
            booking_date: request.booking_date,
            label: request.label,
            amount: request.amount,
            effect: request.effect,
            nature: request.nature,
            category_id: request.category_id,
            note: request.note,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum OperationTypeResponse {
    Transaction,
    Transfer,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountSummaryResponse {
    id: AccountId,
    name: String,
    kind: AccountKind,
    archived: bool,
}

impl From<AccountSummary> for AccountSummaryResponse {
    fn from(summary: AccountSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            kind: summary.kind,
            archived: summary.archived,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CategorySummaryResponse {
    id: CategoryId,
    name: String,
    kind: CategoryKind,
    archived: bool,
}

impl From<CategorySummary> for CategorySummaryResponse {
    fn from(summary: CategorySummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            kind: summary.kind,
            archived: summary.archived,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOperationResponse {
    operation_type: OperationTypeResponse,
    id: TransactionId,
    household_id: HouseholdId,
    booking_date: NaiveDate,
    label: String,
    nature: TransactionNature,
    amount: Decimal,
    effect: TransactionEffect,
    economic_amount: Decimal,
    account_amount: Decimal,
    account: AccountSummaryResponse,
    category: Option<CategorySummaryResponse>,
    origin: TransactionSource,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TransactionOperation> for TransactionOperationResponse {
    fn from(transaction: TransactionOperation) -> Self {
        Self {
            operation_type: OperationTypeResponse::Transaction,
            id: transaction.id,
            household_id: transaction.household_id,
            booking_date: transaction.booking_date,
            label: transaction.label.into_string(),
            nature: transaction.nature,
            amount: transaction.amounts.nominal.value(),
            effect: transaction.amounts.effect,
            economic_amount: transaction.amounts.economic,
            account_amount: transaction.amounts.account.value(),
            account: transaction.account.into(),
            category: transaction.category.map(Into::into),
            origin: transaction.origin,
            note: transaction.note.map(|note| note.into_string()),
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransferMovementOperationResponse {
    transaction_id: TransactionId,
    booking_date: NaiveDate,
    label: String,
    account_amount: Decimal,
    account: AccountSummaryResponse,
    note: Option<String>,
}

impl From<TransferMovementOperation> for TransferMovementOperationResponse {
    fn from(movement: TransferMovementOperation) -> Self {
        Self {
            transaction_id: movement.transaction_id,
            booking_date: movement.booking_date,
            label: movement.label.into_string(),
            account_amount: movement.account_amount.value(),
            account: movement.account.into(),
            note: movement.note.map(|note| note.into_string()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOperationResponse {
    operation_type: OperationTypeResponse,
    id: TransferId,
    household_id: HouseholdId,
    booking_date: NaiveDate,
    amount: Decimal,
    source: TransferMovementOperationResponse,
    destination: TransferMovementOperationResponse,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<TransferOperation> for TransferOperationResponse {
    fn from(transfer: TransferOperation) -> Self {
        Self {
            operation_type: OperationTypeResponse::Transfer,
            id: transfer.id,
            household_id: transfer.household_id,
            booking_date: transfer.booking_date,
            amount: transfer.amount.value(),
            source: transfer.source.into(),
            destination: transfer.destination.into(),
            created_at: transfer.created_at,
            updated_at: transfer.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum OperationResponse {
    Transaction(TransactionOperationResponse),
    Transfer(TransferOperationResponse),
}

impl From<Operation> for OperationResponse {
    fn from(operation: Operation) -> Self {
        match operation {
            Operation::Transaction(transaction) => Self::Transaction(transaction.into()),
            Operation::Transfer(transfer) => Self::Transfer(transfer.into()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPageResponse {
    items: Vec<OperationResponse>,
    page: i64,
    limit: i64,
    total: i64,
}

impl From<OperationPage> for OperationPageResponse {
    fn from(page: OperationPage) -> Self {
        Self {
            items: page.items.into_iter().map(Into::into).collect(),
            page: page.page,
            limit: page.limit,
            total: page.total,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn missing_optional_update_is_unchanged() {
        let request: UpdateTransactionRequest = serde_json::from_value(json!({})).unwrap();

        assert_eq!(request.category_id, FieldUpdate::Unchanged);
        assert_eq!(request.note, FieldUpdate::Unchanged);
    }

    #[test]
    fn null_optional_update_clears_the_value() {
        let request: UpdateTransactionRequest = serde_json::from_value(json!({
            "categoryId": null,
            "note": null
        }))
        .unwrap();

        assert_eq!(request.category_id, FieldUpdate::Clear);
        assert_eq!(request.note, FieldUpdate::Clear);
    }

    #[test]
    fn provided_optional_update_sets_the_value() {
        let category_id = CategoryId::new();
        let request: UpdateTransactionRequest = serde_json::from_value(json!({
            "categoryId": category_id,
            "note": "Monthly groceries"
        }))
        .unwrap();

        assert_eq!(request.category_id, FieldUpdate::Set(category_id));
        assert_eq!(
            request.note,
            FieldUpdate::Set("Monthly groceries".to_owned())
        );
    }
}
