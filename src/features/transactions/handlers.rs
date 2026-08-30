use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath, ApiQuery},
    types::{HouseholdId, TransactionId},
};

use super::{
    TransactionsModule,
    dto::{
        CreateTransactionRequest, ListTransactionsQuery, OperationPageResponse,
        TransactionOperationResponse, UpdateTransactionRequest,
    },
};

pub(super) async fn create(
    State(module): State<TransactionsModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
    ApiJson(request): ApiJson<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionOperationResponse>), ApiError> {
    let transaction = module.service.create(household_id, request.into()).await?;

    Ok((StatusCode::CREATED, Json(transaction.into())))
}

pub(super) async fn list(
    State(module): State<TransactionsModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
    ApiQuery(query): ApiQuery<ListTransactionsQuery>,
) -> Result<Json<OperationPageResponse>, ApiError> {
    Ok(Json(
        module
            .service
            .list(household_id, query.try_into()?)
            .await?
            .into(),
    ))
}

pub(super) async fn get(
    State(module): State<TransactionsModule>,
    ApiPath((household_id, transaction_id)): ApiPath<(HouseholdId, TransactionId)>,
) -> Result<Json<TransactionOperationResponse>, ApiError> {
    let transaction = module.service.get(household_id, transaction_id).await?;

    Ok(Json(transaction.into()))
}

pub(super) async fn update(
    State(module): State<TransactionsModule>,
    ApiPath((household_id, transaction_id)): ApiPath<(HouseholdId, TransactionId)>,
    ApiJson(request): ApiJson<UpdateTransactionRequest>,
) -> Result<Json<TransactionOperationResponse>, ApiError> {
    let transaction = module
        .service
        .update(household_id, transaction_id, request.into())
        .await?;

    Ok(Json(transaction.into()))
}

pub(super) async fn delete(
    State(module): State<TransactionsModule>,
    ApiPath((household_id, transaction_id)): ApiPath<(HouseholdId, TransactionId)>,
) -> Result<StatusCode, ApiError> {
    module.service.delete(household_id, transaction_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
