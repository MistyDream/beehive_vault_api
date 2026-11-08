use axum::{Json, extract::State};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath},
    types::{AccountId, HouseholdId},
};

use super::{
    CsvImportModule,
    dto::{PreviewRequest, PreviewResponse},
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
