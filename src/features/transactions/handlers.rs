use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    error::ApiError,
    types::{HouseholdId, TransactionId},
};

use super::{
    TransactionsModule,
    dto::{
        CreateTransactionRequest, ListTransactionsQuery, TransactionResponse,
        UpdateTransactionRequest,
    },
};

pub(super) async fn create(
    State(module): State<TransactionsModule>,
    Path(household_id): Path<HouseholdId>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), ApiError> {
    let transaction = module.service.create(household_id, request.into()).await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub(super) async fn list(
    State(module): State<TransactionsModule>,
    Path(household_id): Path<HouseholdId>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<Vec<TransactionResponse>>, ApiError> {
    let transactions = module.service.list(household_id, query.into()).await?;

    Ok(Json(transactions.into_iter().map(Into::into).collect()))
}

pub(super) async fn get(
    State(module): State<TransactionsModule>,
    Path((household_id, transaction_id)): Path<(HouseholdId, TransactionId)>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let transaction = module.service.get(household_id, transaction_id).await?;

    Ok(Json(transaction.into()))
}

pub(super) async fn update(
    State(module): State<TransactionsModule>,
    Path((household_id, transaction_id)): Path<(HouseholdId, TransactionId)>,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<Json<TransactionResponse>, ApiError> {
    let transaction = module
        .service
        .update(household_id, transaction_id, request.into())
        .await?;

    Ok(Json(transaction.into()))
}

pub(super) async fn delete(
    State(module): State<TransactionsModule>,
    Path((household_id, transaction_id)): Path<(HouseholdId, TransactionId)>,
) -> Result<StatusCode, ApiError> {
    module.service.delete(household_id, transaction_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
