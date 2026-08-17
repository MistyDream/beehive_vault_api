use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId};

use super::{
    domain::{Transaction, TransactionNature, TransactionSource, TransferRole},
    service::{
        CreateTransactionCommand, FieldUpdate, ListTransactionsCommand, UpdateTransactionCommand,
    },
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
    source: Option<TransactionSource>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl From<ListTransactionsQuery> for ListTransactionsCommand {
    fn from(query: ListTransactionsQuery) -> Self {
        Self {
            account_id: query.account_id,
            date_from: query.date_from,
            date_to: query.date_to,
            nature: query.nature,
            category_id: query.category_id,
            source: query.source,
            search: query.search,
            limit: query.limit,
            offset: query.offset,
        }
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
    #[serde(default, deserialize_with = "deserialize_field_update")]
    category_id: FieldUpdate<CategoryId>,
    #[serde(default, deserialize_with = "deserialize_field_update")]
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

fn deserialize_field_update<'de, D, T>(deserializer: D) -> Result<FieldUpdate<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(|value| match value {
        Some(value) => FieldUpdate::Set(value),
        None => FieldUpdate::Clear,
    })
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
