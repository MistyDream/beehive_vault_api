use chrono::{NaiveDate, Utc};

use crate::{
    error::{ApiError, ProblemKind, required_text},
    types::{
        AccountBalance, AccountId, BalanceSnapshotId, CurrencyCode, HouseholdId, InstitutionId,
    },
};

use super::{
    domain::{
        Account, AccountKind, BalanceSnapshot, BalanceSource, NewAccount, NewBalanceSnapshot,
    },
    repository::{AccountRepository, CreateBalanceOutcome},
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

#[derive(Default)]
pub struct UpdateBalanceCommand {
    pub amount: Option<AccountBalance>,
    pub balance_date: Option<NaiveDate>,
}

impl UpdateBalanceCommand {
    fn is_empty(&self) -> bool {
        self.amount.is_none() && self.balance_date.is_none()
    }
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
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        if command.currency != household_currency {
            return Err(ApiError::body_validation(
                "#/currency",
                "account_currency_mismatch",
                format!("account currency must match household currency {household_currency}"),
            ));
        }
        let current_date = self.current_date_for_household(household_id).await?;
        validate_balance_date_not_future(command.balance_date, current_date)?;
        self.validate_institution(command.institution_id).await?;

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
        if self
            .repository
            .household_currency(household_id)
            .await?
            .is_none()
        {
            return Err(ApiError::new(ProblemKind::HouseholdNotFound));
        }
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
            .ok_or_else(|| ApiError::new(ProblemKind::AccountNotFound))
    }

    pub async fn update(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        command: UpdateAccountCommand,
    ) -> Result<Account, ApiError> {
        if let Some(kind) = command.kind {
            let current = self.get(household_id, account_id).await?;
            if kind.is_liability() != current.kind.is_liability()
                && self
                    .repository
                    .has_transactions(household_id, account_id)
                    .await?
            {
                return Err(
                    ApiError::new(ProblemKind::AccountKindChangeForbidden).with_detail(
                        "An account with transactions cannot switch between asset and liability.",
                    ),
                );
            }
        }
        let name = command
            .name
            .map(|name| required_text(name, "name"))
            .transpose()?;
        if !command.remove_institution {
            self.validate_institution(command.institution_id).await?;
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
            return Err(ApiError::new(ProblemKind::AccountNotFound));
        }
        self.get(household_id, account_id).await
    }

    pub async fn archive(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<(), ApiError> {
        if self.repository.archive(household_id, account_id).await? == 0 {
            return Err(ApiError::new(ProblemKind::AccountNotFound));
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
        let current_date = self.current_date_for_household(household_id).await?;
        validate_balance_date_not_future(command.balance_date, current_date)?;
        match self
            .repository
            .create_balance(
                BalanceSnapshotId::new(),
                account_id,
                command.amount,
                command.balance_date,
                command.source.unwrap_or(BalanceSource::Manual),
            )
            .await?
        {
            CreateBalanceOutcome::Created(balance) => Ok(balance),
            CreateBalanceOutcome::NotAfterLatest => Err(ApiError::body_validation(
                "#/balanceDate",
                "balance_date_not_after_latest",
                "balance date must be strictly after the latest balance date",
            )),
        }
    }

    pub async fn list_balances(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
    ) -> Result<Vec<BalanceSnapshot>, ApiError> {
        self.get(household_id, account_id).await?;
        Ok(self.repository.list_balances(account_id).await?)
    }

    pub async fn update_balance(
        &self,
        household_id: HouseholdId,
        account_id: AccountId,
        balance_id: BalanceSnapshotId,
        command: UpdateBalanceCommand,
    ) -> Result<BalanceSnapshot, ApiError> {
        self.get(household_id, account_id).await?;
        if command.is_empty() {
            return Err(ApiError::body_validation(
                "#/",
                "empty_update",
                "amount or balanceDate must be provided",
            ));
        }
        if let Some(balance_date) = command.balance_date {
            let current_date = self.current_date_for_household(household_id).await?;
            validate_balance_date_not_future(balance_date, current_date)?;
        }

        self.repository
            .update_balance(
                household_id,
                account_id,
                balance_id,
                command.amount,
                command.balance_date,
            )
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::BalanceNotFound))
    }

    async fn current_date_for_household(
        &self,
        household_id: HouseholdId,
    ) -> Result<NaiveDate, ApiError> {
        let timezone = self
            .repository
            .household_timezone(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        timezone.date_at(Utc::now()).map_err(|_| {
            ApiError::new(ProblemKind::InternalError)
                .with_detail("The household timezone could not be evaluated.")
        })
    }

    async fn validate_institution(
        &self,
        institution_id: Option<InstitutionId>,
    ) -> Result<(), ApiError> {
        let Some(institution_id) = institution_id else {
            return Ok(());
        };
        if !self.repository.institution_exists(institution_id).await? {
            return Err(ApiError::body_validation(
                "#/institutionId",
                "invalid_institution",
                "institution must exist in the global catalog",
            ));
        }
        Ok(())
    }
}

fn validate_balance_date_not_future(
    balance_date: NaiveDate,
    current_date: NaiveDate,
) -> Result<(), ApiError> {
    if balance_date > current_date {
        return Err(ApiError::body_validation(
            "#/balanceDate",
            "balance_date_in_future",
            "balance date must not be in the future for the household timezone",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_balance_update_is_detected() {
        assert!(UpdateBalanceCommand::default().is_empty());
    }

    #[test]
    fn future_balance_date_is_rejected() {
        let current_date = NaiveDate::from_ymd_opt(2026, 8, 30).unwrap();
        let future_date = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();

        assert!(validate_balance_date_not_future(future_date, current_date).is_err());
        assert!(validate_balance_date_not_future(current_date, current_date).is_ok());
    }
}
