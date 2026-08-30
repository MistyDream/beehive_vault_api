use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath, ApiQuery},
    pagination::{Pagination, PaginationQuery},
    types::{HouseholdId, TransferId},
};

use super::{
    TransfersModule,
    dto::{CreateTransferRequest, TransferPageResponse, TransferResponse, UpdateTransferRequest},
};

pub(super) async fn create(
    State(module): State<TransfersModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
    ApiJson(request): ApiJson<CreateTransferRequest>,
) -> Result<(StatusCode, Json<TransferResponse>), ApiError> {
    let transfer = module.service.create(household_id, request.into()).await?;

    Ok((StatusCode::CREATED, Json(transfer.into())))
}

pub(super) async fn list(
    State(module): State<TransfersModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
    ApiQuery(query): ApiQuery<PaginationQuery>,
) -> Result<Json<TransferPageResponse>, ApiError> {
    let page = module
        .service
        .list(household_id, Pagination::try_from(query)?)
        .await?;

    Ok(Json(page.into()))
}

pub(super) async fn get(
    State(module): State<TransfersModule>,
    ApiPath((household_id, transfer_id)): ApiPath<(HouseholdId, TransferId)>,
) -> Result<Json<TransferResponse>, ApiError> {
    let transfer = module.service.get(household_id, transfer_id).await?;

    Ok(Json(transfer.into()))
}

pub(super) async fn update(
    State(module): State<TransfersModule>,
    ApiPath((household_id, transfer_id)): ApiPath<(HouseholdId, TransferId)>,
    ApiJson(request): ApiJson<UpdateTransferRequest>,
) -> Result<Json<TransferResponse>, ApiError> {
    let transfer = module
        .service
        .update(household_id, transfer_id, request.into())
        .await?;

    Ok(Json(transfer.into()))
}

pub(super) async fn delete(
    State(module): State<TransfersModule>,
    ApiPath((household_id, transfer_id)): ApiPath<(HouseholdId, TransferId)>,
) -> Result<StatusCode, ApiError> {
    module.service.delete(household_id, transfer_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
