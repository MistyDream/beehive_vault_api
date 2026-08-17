use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    error::ApiError,
    pagination::{Pagination, PaginationQuery},
    types::{HouseholdId, TransferId},
};

use super::{
    TransfersModule,
    dto::{CreateTransferRequest, TransferResponse, UpdateTransferRequest},
};

pub(super) async fn create(
    State(module): State<TransfersModule>,
    Path(household_id): Path<HouseholdId>,
    Json(request): Json<CreateTransferRequest>,
) -> Result<(StatusCode, Json<TransferResponse>), ApiError> {
    let transfer = module.service.create(household_id, request.into()).await?;

    Ok((StatusCode::CREATED, Json(transfer.into())))
}

pub(super) async fn list(
    State(module): State<TransfersModule>,
    Path(household_id): Path<HouseholdId>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<TransferResponse>>, ApiError> {
    let transfers = module
        .service
        .list(household_id, Pagination::try_from(query)?)
        .await?;

    Ok(Json(transfers.into_iter().map(Into::into).collect()))
}

pub(super) async fn get(
    State(module): State<TransfersModule>,
    Path((household_id, transfer_id)): Path<(HouseholdId, TransferId)>,
) -> Result<Json<TransferResponse>, ApiError> {
    let transfer = module.service.get(household_id, transfer_id).await?;

    Ok(Json(transfer.into()))
}

pub(super) async fn update(
    State(module): State<TransfersModule>,
    Path((household_id, transfer_id)): Path<(HouseholdId, TransferId)>,
    Json(request): Json<UpdateTransferRequest>,
) -> Result<Json<TransferResponse>, ApiError> {
    let transfer = module
        .service
        .update(household_id, transfer_id, request.into())
        .await?;

    Ok(Json(transfer.into()))
}

pub(super) async fn delete(
    State(module): State<TransfersModule>,
    Path((household_id, transfer_id)): Path<(HouseholdId, TransferId)>,
) -> Result<StatusCode, ApiError> {
    module.service.delete(household_id, transfer_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
