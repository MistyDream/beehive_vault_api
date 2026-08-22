use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath},
    types::{AccountId, HouseholdId},
};

use super::{
    AccountsModule,
    dto::{
        AccountResponse, BalanceResponse, CreateAccountRequest, CreateBalanceRequest,
        UpdateAccountRequest,
    },
};

pub(super) async fn create(
    State(module): State<AccountsModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
    ApiJson(request): ApiJson<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), ApiError> {
    let account = module.service.create(household_id, request.into()).await?;
    Ok((StatusCode::CREATED, Json(account.into())))
}

pub(super) async fn list(
    State(module): State<AccountsModule>,
    ApiPath(household_id): ApiPath<HouseholdId>,
) -> Result<Json<Vec<AccountResponse>>, ApiError> {
    let accounts = module.service.list(household_id).await?;
    Ok(Json(accounts.into_iter().map(Into::into).collect()))
}

pub(super) async fn get(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
) -> Result<Json<AccountResponse>, ApiError> {
    Ok(Json(
        module.service.get(household_id, account_id).await?.into(),
    ))
}

pub(super) async fn update(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
    ApiJson(request): ApiJson<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    Ok(Json(
        module
            .service
            .update(household_id, account_id, request.into())
            .await?
            .into(),
    ))
}

pub(super) async fn archive(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
) -> Result<StatusCode, ApiError> {
    module.service.archive(household_id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn create_balance(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
    ApiJson(request): ApiJson<CreateBalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), ApiError> {
    let balance = module
        .service
        .create_balance(household_id, account_id, request.into())
        .await?;
    Ok((StatusCode::CREATED, Json(balance.into())))
}

pub(super) async fn list_balances(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
) -> Result<Json<Vec<BalanceResponse>>, ApiError> {
    let balances = module
        .service
        .list_balances(household_id, account_id)
        .await?;
    Ok(Json(balances.into_iter().map(Into::into).collect()))
}
