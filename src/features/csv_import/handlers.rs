use axum::{Json, extract::State};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath},
    types::{AccountId, HouseholdId},
};

use super::{
    CsvImportModule,
    dto::{ConfirmRequest, ConfirmResponse, PreviewRequest, PreviewResponse},
};

pub(super) async fn preview(
    State(module): State<CsvImportModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
    ApiJson(request): ApiJson<PreviewRequest>,
) -> Result<Json<PreviewResponse>, ApiError> {
    let (currency, rows) = module
        .service
        .preview(household_id, account_id, &request.csv, &request.format)
        .await?;

    Ok(Json(PreviewResponse {
        account_id,
        currency,
        rows: rows.into_iter().map(Into::into).collect(),
    }))
}

pub(super) async fn confirm(
    State(module): State<CsvImportModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
    ApiJson(request): ApiJson<ConfirmRequest>,
) -> Result<Json<ConfirmResponse>, ApiError> {
    let rows = module
        .service
        .confirm(
            household_id,
            account_id,
            &request.csv,
            &request.format,
            &request.file_name,
            request.rows.into_iter().map(Into::into).collect(),
        )
        .await?;

    Ok(Json(ConfirmResponse {
        account_id,
        rows: rows.into_iter().map(Into::into).collect(),
    }))
}
