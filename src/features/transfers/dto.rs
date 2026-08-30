use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    types::{AccountId, HouseholdId, TransactionId, TransferId},
    update::FieldUpdate,
};

use super::{
    domain::{Transfer, TransferMovement, TransferPage},
    service::{
        CreateTransferCommand, TransferMovementCommand, UpdateTransferCommand,
        UpdateTransferMovementCommand,
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferMovementRequest {
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    note: Option<String>,
}

impl From<TransferMovementRequest> for TransferMovementCommand {
    fn from(request: TransferMovementRequest) -> Self {
        Self {
            account_id: request.account_id,
            booking_date: request.booking_date,
            label: request.label,
            note: request.note,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateTransferRequest {
    amount: Decimal,
    source: TransferMovementRequest,
    destination: TransferMovementRequest,
}

impl From<CreateTransferRequest> for CreateTransferCommand {
    fn from(request: CreateTransferRequest) -> Self {
        Self {
            amount: request.amount,
            source: request.source.into(),
            destination: request.destination.into(),
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransferMovementRequest {
    account_id: Option<AccountId>,
    booking_date: Option<NaiveDate>,
    label: Option<String>,
    #[serde(default)]
    note: FieldUpdate<String>,
}

impl From<UpdateTransferMovementRequest> for UpdateTransferMovementCommand {
    fn from(request: UpdateTransferMovementRequest) -> Self {
        Self {
            account_id: request.account_id,
            booking_date: request.booking_date,
            label: request.label,
            note: request.note,
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateTransferRequest {
    amount: Option<Decimal>,
    source: Option<UpdateTransferMovementRequest>,
    destination: Option<UpdateTransferMovementRequest>,
}

impl From<UpdateTransferRequest> for UpdateTransferCommand {
    fn from(request: UpdateTransferRequest) -> Self {
        Self {
            amount: request.amount,
            source: request.source.map(Into::into),
            destination: request.destination.map(Into::into),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferMovementResponse {
    transaction_id: TransactionId,
    account_id: AccountId,
    booking_date: NaiveDate,
    label: String,
    amount: Decimal,
    note: Option<String>,
}

impl From<TransferMovement> for TransferMovementResponse {
    fn from(movement: TransferMovement) -> Self {
        Self {
            transaction_id: movement.transaction_id,
            account_id: movement.account_id,
            booking_date: movement.booking_date,
            label: movement.label.into_string(),
            amount: movement.amount.value(),
            note: movement.note.map(|note| note.into_string()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    id: TransferId,
    household_id: HouseholdId,
    amount: Decimal,
    source: TransferMovementResponse,
    destination: TransferMovementResponse,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Transfer> for TransferResponse {
    fn from(transfer: Transfer) -> Self {
        let amount = transfer.amount().value();

        Self {
            id: transfer.id,
            household_id: transfer.household_id,
            amount,
            source: transfer.source.into(),
            destination: transfer.destination.into(),
            created_at: transfer.created_at,
            updated_at: transfer.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferPageResponse {
    items: Vec<TransferResponse>,
    page: i64,
    limit: i64,
    total: i64,
}

impl From<TransferPage> for TransferPageResponse {
    fn from(page: TransferPage) -> Self {
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
    fn missing_movement_note_is_unchanged() {
        let request: UpdateTransferRequest = serde_json::from_value(json!({
            "source": {}
        }))
        .unwrap();

        assert_eq!(request.source.unwrap().note, FieldUpdate::Unchanged);
    }

    #[test]
    fn null_movement_note_clears_the_value() {
        let request: UpdateTransferRequest = serde_json::from_value(json!({
            "destination": { "note": null }
        }))
        .unwrap();

        assert_eq!(request.destination.unwrap().note, FieldUpdate::Clear);
    }
}
