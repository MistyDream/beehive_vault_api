use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    pagination::{Pagination, PaginationError, PaginationQuery},
    types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId},
    update::FieldUpdate,
};

use super::{
    domain::{Transaction, TransactionNature, TransactionSource, TransferRole},
    service::{CreateTransactionCommand, ListTransactionsCommand, UpdateTransactionCommand},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionRequest {
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    amount: Decimal,
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
            nature: request.nature,
            category_id: request.category_id,
            note: request.note,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    id: TransactionId,
    household_id: HouseholdId,
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    amount: Decimal,
    nature: TransactionNature,
    category_id: Option<CategoryId>,
    transfer_id: Option<TransferId>,
    transfer_role: Option<TransferRole>,
    source: TransactionSource,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        let nature = transaction.details.nature();
        let category_id = transaction.details.category_id();
        let transfer_id = transaction.details.transfer_id();
        let transfer_role = transaction.details.transfer_role();

        Self {
            id: transaction.id,
            household_id: transaction.household_id,
            account_id: transaction.account_id,
            booking_date: transaction.booking_date,
            label: transaction.label.into_string(),
            amount: transaction.amount.value(),
            nature,
            category_id,
            transfer_id,
            transfer_role,
            source: transaction.source,
            note: transaction.note.map(|note| note.into_string()),
            created_at: transaction.created_at,
            updated_at: transaction.updated_at,
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
