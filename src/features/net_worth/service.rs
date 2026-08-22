use rust_decimal::Decimal;

use crate::{
    error::{ApiError, ProblemKind},
    features::accounts::AccountRepository,
    types::{AccountBalance, HouseholdId},
};

use super::domain::NetWorthSummary;

#[derive(Clone)]
pub struct NetWorthService {
    account_repository: AccountRepository,
}

impl NetWorthService {
    pub fn new(account_repository: AccountRepository) -> Self {
        Self { account_repository }
    }

    pub async fn summary(&self, household_id: HouseholdId) -> Result<NetWorthSummary, ApiError> {
        let currency = self
            .account_repository
            .household_currency(household_id)
            .await?
            .ok_or_else(|| ApiError::new(ProblemKind::HouseholdNotFound))?;
        let accounts = self.account_repository.list(household_id).await?;
        let mut assets = Decimal::ZERO;
        let mut liabilities = Decimal::ZERO;

        for account in accounts {
            let raw_balance = account.calculated_balance.value();
            let economic_balance = if account.kind.is_liability() {
                -raw_balance
            } else {
                raw_balance
            };
            if economic_balance >= Decimal::ZERO {
                assets += economic_balance;
            } else {
                liabilities -= economic_balance;
            }
        }

        Ok(NetWorthSummary {
            currency,
            assets: AccountBalance::new(assets),
            liabilities: AccountBalance::new(liabilities),
            net_worth: AccountBalance::new(assets - liabilities),
        })
    }
}
