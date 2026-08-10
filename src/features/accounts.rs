use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    error::{ApiError, required_text},
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountKind {
    Checking,
    Savings,
    Cash,
    Investment,
    CreditCard,
    Loan,
    OtherAsset,
    OtherLiability,
}

impl AccountKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Checking => "checking",
            Self::Savings => "savings",
            Self::Cash => "cash",
            Self::Investment => "investment",
            Self::CreditCard => "credit_card",
            Self::Loan => "loan",
            Self::OtherAsset => "other_asset",
            Self::OtherLiability => "other_liability",
        }
    }

    pub fn is_liability(self) -> bool {
        matches!(self, Self::CreditCard | Self::Loan | Self::OtherLiability)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BalanceSource {
    Manual,
    Import,
    Synchronization,
    Reconciliation,
}

impl BalanceSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Import => "import",
            Self::Synchronization => "synchronization",
            Self::Reconciliation => "reconciliation",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    institution_id: Option<Uuid>,
    name: String,
    kind: AccountKind,
    currency: String,
    initial_balance: Decimal,
    balance_date: NaiveDate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    name: Option<String>,
    kind: Option<AccountKind>,
    institution_id: Option<Uuid>,
    #[serde(default)]
    remove_institution: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBalanceRequest {
    amount: Decimal,
    balance_date: NaiveDate,
    source: Option<BalanceSource>,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    id: Uuid,
    household_id: Uuid,
    institution_id: Option<Uuid>,
    name: String,
    kind: String,
    currency: String,
    latest_balance: Option<Decimal>,
    balance_date: Option<NaiveDate>,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    id: Uuid,
    account_id: Uuid,
    amount: Decimal,
    balance_date: NaiveDate,
    source: String,
    created_at: DateTime<Utc>,
}

pub async fn create(
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), ApiError> {
    let name = required_text(request.name, "name")?;
    let household_currency = household_currency(&state, household_id).await?;
    let currency = crate::error::currency_code(request.currency)?;
    if currency != household_currency {
        return Err(ApiError::BadRequest(format!(
            "account currency must match household currency {household_currency}"
        )));
    }
    validate_institution(&state, household_id, request.institution_id).await?;

    let account_id = Uuid::now_v7();
    let mut transaction = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO accounts (id, household_id, institution_id, name, kind, currency) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(account_id)
    .bind(household_id)
    .bind(request.institution_id)
    .bind(name)
    .bind(request.kind.as_str())
    .bind(currency)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO account_balance_snapshots \
         (id, account_id, amount, balance_date, source) VALUES ($1, $2, $3, $4, 'manual')",
    )
    .bind(Uuid::now_v7())
    .bind(account_id)
    .bind(request.initial_balance)
    .bind(request.balance_date)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(fetch_account(&state, household_id, account_id).await?),
    ))
}

pub async fn list(
    State(state): State<AppState>,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<AccountResponse>>, ApiError> {
    let accounts = sqlx::query_as::<_, AccountResponse>(
        "SELECT a.id, a.household_id, a.institution_id, a.name, a.kind, a.currency, \
         latest.amount AS latest_balance, latest.balance_date, a.archived_at, \
         a.created_at, a.updated_at FROM accounts a \
         LEFT JOIN LATERAL (SELECT amount, balance_date FROM account_balance_snapshots \
         WHERE account_id = a.id ORDER BY balance_date DESC LIMIT 1) latest ON true \
         WHERE a.household_id = $1 AND a.archived_at IS NULL ORDER BY lower(a.name)",
    )
    .bind(household_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(accounts))
}

pub async fn get(
    State(state): State<AppState>,
    Path((household_id, account_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AccountResponse>, ApiError> {
    Ok(Json(fetch_account(&state, household_id, account_id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Path((household_id, account_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    let name = request
        .name
        .map(|name| required_text(name, "name"))
        .transpose()?;
    if !request.remove_institution {
        validate_institution(&state, household_id, request.institution_id).await?;
    }
    let kind = request.kind.map(AccountKind::as_str);
    let result = sqlx::query(
        "UPDATE accounts SET \
         name = COALESCE($3, name), kind = COALESCE($4, kind), \
         institution_id = CASE WHEN $6 THEN NULL ELSE COALESCE($5, institution_id) END, \
         updated_at = now() WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
    )
    .bind(household_id)
    .bind(account_id)
    .bind(name)
    .bind(kind)
    .bind(request.institution_id)
    .bind(request.remove_institution)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Account not found".to_owned()));
    }
    Ok(Json(fetch_account(&state, household_id, account_id).await?))
}

pub async fn archive(
    State(state): State<AppState>,
    Path((household_id, account_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let result = sqlx::query(
        "UPDATE accounts SET archived_at = now(), updated_at = now() \
         WHERE household_id = $1 AND id = $2 AND archived_at IS NULL",
    )
    .bind(household_id)
    .bind(account_id)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Account not found".to_owned()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_balance(
    State(state): State<AppState>,
    Path((household_id, account_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateBalanceRequest>,
) -> Result<(StatusCode, Json<BalanceResponse>), ApiError> {
    fetch_account(&state, household_id, account_id).await?;
    let source = request.source.unwrap_or(BalanceSource::Manual);
    let balance = sqlx::query_as::<_, BalanceResponse>(
        "INSERT INTO account_balance_snapshots (id, account_id, amount, balance_date, source) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, account_id, amount, balance_date, source, created_at",
    )
    .bind(Uuid::now_v7())
    .bind(account_id)
    .bind(request.amount)
    .bind(request.balance_date)
    .bind(source.as_str())
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(balance)))
}

pub async fn list_balances(
    State(state): State<AppState>,
    Path((household_id, account_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<BalanceResponse>>, ApiError> {
    fetch_account(&state, household_id, account_id).await?;
    let balances = sqlx::query_as::<_, BalanceResponse>(
        "SELECT id, account_id, amount, balance_date, source, created_at \
         FROM account_balance_snapshots WHERE account_id = $1 \
         ORDER BY balance_date DESC",
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(balances))
}

async fn fetch_account(
    state: &AppState,
    household_id: Uuid,
    account_id: Uuid,
) -> Result<AccountResponse, ApiError> {
    sqlx::query_as::<_, AccountResponse>(
        "SELECT a.id, a.household_id, a.institution_id, a.name, a.kind, a.currency, \
         latest.amount AS latest_balance, latest.balance_date, a.archived_at, \
         a.created_at, a.updated_at FROM accounts a \
         LEFT JOIN LATERAL (SELECT amount, balance_date FROM account_balance_snapshots \
         WHERE account_id = a.id ORDER BY balance_date DESC LIMIT 1) latest ON true \
         WHERE a.household_id = $1 AND a.id = $2 AND a.archived_at IS NULL",
    )
    .bind(household_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("Account not found".to_owned()))
}

async fn household_currency(state: &AppState, household_id: Uuid) -> Result<String, ApiError> {
    sqlx::query_scalar("SELECT base_currency FROM households WHERE id = $1")
        .bind(household_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))
}

async fn validate_institution(
    state: &AppState,
    household_id: Uuid,
    institution_id: Option<Uuid>,
) -> Result<(), ApiError> {
    let Some(institution_id) = institution_id else {
        return Ok(());
    };
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM institutions \
         WHERE household_id = $1 AND id = $2 AND archived_at IS NULL)",
    )
    .bind(household_id)
    .bind(institution_id)
    .fetch_one(&state.db)
    .await?;
    if !exists {
        return Err(ApiError::BadRequest(
            "institution does not belong to the household".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AccountKind;

    #[test]
    fn liability_kinds_are_explicit() {
        assert!(AccountKind::Loan.is_liability());
        assert!(AccountKind::CreditCard.is_liability());
        assert!(!AccountKind::Checking.is_liability());
        assert!(!AccountKind::Investment.is_liability());
    }
}
