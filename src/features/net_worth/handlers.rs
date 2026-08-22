use axum::{Json, extract::State};
use serde::Serialize;

use crate::{
    error::ApiError,
    extract::ApiPath,
    types::{AccountBalance, CurrencyCode, HouseholdId},
};

use super::{NetWorthModule, domain::NetWorthSummary};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NetWorthResponse {
    currency: CurrencyCode,
    assets: AccountBalance,
    liabilities: AccountBalance,
    net_worth: AccountBalance,
}

impl From<NetWorthSummary> for NetWorthResponse {
    fn from(summary: NetWorthSummary) -> Self {
        Self {
            currency: summary.currency,
            assets: summary.assets,
            liabilities: summary.liabilities,
            net_worth: summary.net_worth,
        }
    }
}

pub(super) async fn summary(
    State(module): State<NetWorthModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
) -> Result<Json<NetWorthResponse>, ApiError> {
    Ok(Json(module.service.summary(household_id).await?.into()))
}
