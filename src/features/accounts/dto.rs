use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{
    AccountBalance, AccountId, BalanceSnapshotId, CurrencyCode, HouseholdId, InstitutionId,
};

use super::domain::{Account, AccountKind, AccountStatus, BalanceSnapshot, BalanceSource};
use super::service::{
    AccountCollection, AccountTotals, CreateAccountCommand, CreateBalanceCommand,
    UpdateAccountCommand, UpdateBalanceCommand,
};

#[derive(Deserialize)]
pub struct ListAccountsQuery {
    #[serde(default)]
    pub status: AccountStatus,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub institution_id: Option<InstitutionId>,
    pub name: String,
    pub kind: AccountKind,
    pub currency: CurrencyCode,
    pub initial_balance: AccountBalance,
    pub balance_date: NaiveDate,
}

impl From<CreateAccountRequest> for CreateAccountCommand {
    fn from(request: CreateAccountRequest) -> Self {
        Self {
            institution_id: request.institution_id,
            name: request.name,
            kind: request.kind,
            currency: request.currency,
            initial_balance: request.initial_balance,
            balance_date: request.balance_date,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub kind: Option<AccountKind>,
    pub institution_id: Option<InstitutionId>,
    #[serde(default)]
    pub remove_institution: bool,
}

impl From<UpdateAccountRequest> for UpdateAccountCommand {
    fn from(request: UpdateAccountRequest) -> Self {
        Self {
            name: request.name,
            kind: request.kind,
            institution_id: request.institution_id,
            remove_institution: request.remove_institution,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBalanceRequest {
    pub amount: AccountBalance,
    pub balance_date: NaiveDate,
    pub source: Option<BalanceSource>,
}

impl From<CreateBalanceRequest> for CreateBalanceCommand {
    fn from(request: CreateBalanceRequest) -> Self {
        Self {
            amount: request.amount,
            balance_date: request.balance_date,
            source: request.source,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBalanceRequest {
    amount: Option<AccountBalance>,
    balance_date: Option<NaiveDate>,
}

impl From<UpdateBalanceRequest> for UpdateBalanceCommand {
    fn from(request: UpdateBalanceRequest) -> Self {
        Self {
            amount: request.amount,
            balance_date: request.balance_date,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    id: AccountId,
    household_id: HouseholdId,
    institution_id: Option<InstitutionId>,
    name: String,
    kind: AccountKind,
    currency: CurrencyCode,
    latest_balance: Option<AccountBalance>,
    balance_date: Option<NaiveDate>,
    calculated_balance: AccountBalance,
    archived_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            household_id: account.household_id,
            institution_id: account.institution_id,
            name: account.name,
            kind: account.kind,
            currency: account.currency,
            latest_balance: account.latest_balance,
            balance_date: account.balance_date,
            calculated_balance: account.calculated_balance,
            archived_at: account.archived_at,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountTotalsResponse {
    daily: AccountBalance,
    savings: AccountBalance,
    liabilities: AccountBalance,
}

impl From<AccountTotals> for AccountTotalsResponse {
    fn from(totals: AccountTotals) -> Self {
        Self {
            daily: totals.daily,
            savings: totals.savings,
            liabilities: totals.liabilities,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCollectionResponse {
    items: Vec<AccountResponse>,
    totals: AccountTotalsResponse,
}

impl From<AccountCollection> for AccountCollectionResponse {
    fn from(collection: AccountCollection) -> Self {
        Self {
            items: collection.items.into_iter().map(Into::into).collect(),
            totals: collection.totals.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    id: BalanceSnapshotId,
    account_id: AccountId,
    amount: AccountBalance,
    balance_date: NaiveDate,
    source: BalanceSource,
    created_at: DateTime<Utc>,
}

impl From<BalanceSnapshot> for BalanceResponse {
    fn from(snapshot: BalanceSnapshot) -> Self {
        Self {
            id: snapshot.id,
            account_id: snapshot.account_id,
            amount: snapshot.amount,
            balance_date: snapshot.balance_date,
            source: snapshot.source,
            created_at: snapshot.created_at,
        }
    }
}
