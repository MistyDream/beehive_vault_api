use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{
    AccountBalance, AccountId, BalanceSnapshotId, CurrencyCode, HouseholdId, InstitutionId,
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

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    #[default]
    Active,
    Archived,
}

impl AccountKind {
    pub fn as_str(self) -> &'static str {
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

    pub const fn is_liability(self) -> bool {
        matches!(self, Self::CreditCard | Self::Loan | Self::OtherLiability)
    }
}

impl TryFrom<&str> for AccountKind {
    type Error = AccountValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "checking" => Ok(Self::Checking),
            "savings" => Ok(Self::Savings),
            "cash" => Ok(Self::Cash),
            "investment" => Ok(Self::Investment),
            "credit_card" => Ok(Self::CreditCard),
            "loan" => Ok(Self::Loan),
            "other_asset" => Ok(Self::OtherAsset),
            "other_liability" => Ok(Self::OtherLiability),
            _ => Err(AccountValueError::new("account kind", value)),
        }
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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Import => "import",
            Self::Synchronization => "synchronization",
            Self::Reconciliation => "reconciliation",
        }
    }
}

impl TryFrom<&str> for BalanceSource {
    type Error = AccountValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "manual" => Ok(Self::Manual),
            "import" => Ok(Self::Import),
            "synchronization" => Ok(Self::Synchronization),
            "reconciliation" => Ok(Self::Reconciliation),
            _ => Err(AccountValueError::new("balance source", value)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid {field}: {value}")]
pub struct AccountValueError {
    field: &'static str,
    value: String,
}

impl AccountValueError {
    fn new(field: &'static str, value: &str) -> Self {
        Self {
            field,
            value: value.to_owned(),
        }
    }
}

pub struct Account {
    pub id: AccountId,
    pub household_id: HouseholdId,
    pub institution_id: Option<InstitutionId>,
    pub name: String,
    pub kind: AccountKind,
    pub currency: CurrencyCode,
    pub latest_balance: Option<AccountBalance>,
    pub balance_date: Option<NaiveDate>,
    pub calculated_balance: AccountBalance,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewAccount {
    pub id: AccountId,
    pub household_id: HouseholdId,
    pub institution_id: Option<InstitutionId>,
    pub name: String,
    pub kind: AccountKind,
    pub currency: CurrencyCode,
}

pub struct BalanceSnapshot {
    pub id: BalanceSnapshotId,
    pub account_id: AccountId,
    pub amount: AccountBalance,
    pub balance_date: NaiveDate,
    pub source: BalanceSource,
    pub created_at: DateTime<Utc>,
}

pub struct NewBalanceSnapshot {
    pub id: BalanceSnapshotId,
    pub account_id: AccountId,
    pub amount: AccountBalance,
    pub balance_date: NaiveDate,
    pub source: BalanceSource,
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
