use chrono::NaiveDate;

use crate::{
    error::{ApiError, required_text},
    types::{
        AccountBalance, AccountId, BalanceSnapshotId, CurrencyCode, HouseholdId, InstitutionId,
    },
};

use super::{
    domain::{
        Account, AccountKind, BalanceSnapshot, BalanceSource, NewAccount, NewBalanceSnapshot,
    },
    repository::AccountRepository,
};

pub struct CreateAccountCommand {
    pub institution_id: Option<InstitutionId>,
    pub name: String,
    pub kind: AccountKind,
    pub currency: CurrencyCode,
    pub initial_balance: AccountBalance,
    pub balance_date: NaiveDate,
}

pub struct UpdateAccountCommand {
    pub name: Option<String>,
    pub kind: Option<AccountKind>,
    pub institution_id: Option<InstitutionId>,
    pub remove_institution: bool,
}

pub struct CreateBalanceCommand {
    pub amount: AccountBalance,
    pub balance_date: NaiveDate,
    pub source: Option<BalanceSource>,
}

#[derive(Clone)]
pub struct AccountService {
    repository: AccountRepository,
}

impl AccountService {
    pub fn new(repository: AccountRepository) -> Self {
        Self { repository }
    }

    pub async fn create(
        &self,
        household_id: HouseholdId,
        command: CreateAccountCommand,
    ) -> Result<Account, ApiError> {
        let name = required_text(command.name, "name")?;
        let household_currency = self
            .repository
            .household_currency(household_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Household not found".to_owned()))?;
        if command.currency != household_currency {
            return Err(ApiError::BadRequest(format!(
                "account currency must match household currency {household_currency}"
            )));
        }
        self.validate_institution(household_id, command.institution_id)
            .await?;

        let account_id = AccountId::new();
        self.repository
            .create(
                NewAccount {
                    id: account_id,
                    household_id,
                    institution_id: command.institution_id,
                    name,
                    kind: command.kind,
                    currency: command.currency,
                },
                NewBalanceSnapshot {
                    id: BalanceSnapshotId::new(),
                    account_id,
                    amount: command.initial_balance,
                    balance_date: command.balance_date,
                    source: BalanceSource::Manual,
                },
            )
            .await?;

        self.get(household_id, account_id).await
    }

    pub async fn list(&self, household_id: HouseholdId) -> Result<Vec<Account>, ApiError> {
        Ok(self.repository.list(household_id).await?)
    }

    pub async fn get(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<Account, ApiError> {
        self.repository
            .find(household_id, account_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Account not found".to_owned()))
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        command: UpdateAccountCommand,
    ) -> Result<Account, ApiError> {
        let name = command
            .name
            .map(|name| required_text(name, "name"))
            .transpose()?;
        if !command.remove_institution {
            self.validate_institution(household_id, command.institution_id)
                .await?;
        }
        let affected = self
            .repository
            .update(
                household_id,
                account_id,
                name.as_deref(),
                command.kind,
                command.institution_id,
                command.remove_institution,
            )
            .await?;
        if affected == 0 {
            return Err(ApiError::NotFound("Account not found".to_owned()));
        }
        self.get(household_id, account_id).await
    }

    pub async fn archive(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<(), ApiError> {
        if self.repository.archive(household_id, account_id).await? == 0 {
            return Err(ApiError::NotFound("Account not found".to_owned()));
        }
        Ok(())
    }

    pub async fn create_balance(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        command: CreateBalanceCommand,
    ) -> Result<BalanceSnapshot, ApiError> {
        self.get(household_id, account_id).await?;
        Ok(self
            .repository
            .create_balance(
                BalanceSnapshotId::new(),
                account_id,
                command.amount,
                command.balance_date,
                command.source.unwrap_or(BalanceSource::Manual),
            )
            .await?)
    }

    pub async fn list_balances(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<Vec<BalanceSnapshot>, ApiError> {
        self.get(household_id, account_id).await?;
        Ok(self.repository.list_balances(account_id).await?)
    }

    async fn validate_institution(
        &self,
        household_id: HouseholdId,
        institution_id: Option<InstitutionId>,
    ) -> Result<(), ApiError> {
        let Some(institution_id) = institution_id else {
            return Ok(());
        };
        if !self
            .repository
            .active_institution_exists(household_id, institution_id)
            .await?
        {
            return Err(ApiError::BadRequest(
                "institution does not belong to the household".to_owned(),
            ));
        }
        Ok(())
    }
}
