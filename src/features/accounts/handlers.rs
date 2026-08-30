use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::ApiError,
    extract::{ApiJson, ApiPath, ApiQuery},
    types::{AccountId, BalanceSnapshotId, HouseholdId},
};

use super::{
    AccountsModule,
    dto::{
        AccountCollectionResponse, AccountResponse, BalanceResponse, CreateAccountRequest,
        CreateBalanceRequest, ListAccountsQuery, UpdateAccountRequest, UpdateBalanceRequest,
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
    ApiQuery(query): ApiQuery<ListAccountsQuery>,
) -> Result<Json<AccountCollectionResponse>, ApiError> {
    Ok(Json(
        module
            .service
            .list(household_id, query.status)
            .await?
            .into(),
    ))
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

pub(super) async fn restore(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id)): ApiPath<(HouseholdId, AccountId)>,
) -> Result<Json<AccountResponse>, ApiError> {
    Ok(Json(
        module
            .service
            .restore(household_id, account_id)
            .await?
            .into(),
    ))
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

pub(super) async fn update_balance(
    State(module): State<AccountsModule>,
    ApiPath((household_id, account_id, balance_id)): ApiPath<(
        HouseholdId,
        AccountId,
        BalanceSnapshotId,
    )>,
    ApiJson(request): ApiJson<UpdateBalanceRequest>,
) -> Result<Json<BalanceResponse>, ApiError> {
    let balance = module
        .service
        .update_balance(household_id, account_id, balance_id, request.into())
        .await?;
    Ok(Json(balance.into()))
}
