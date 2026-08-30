use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    features::accounts::AccountKind,
    types::{AccountId, CategoryId, HouseholdId, TransactionId, TransferId},
};

/// A non-zero signed amount representing a transaction's raw effect on an account balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionAmount(Decimal);

impl TransactionAmount {
    pub fn new(value: Decimal) -> Result<Self, TransactionAmountError> {
        if value == Decimal::ZERO {
            return Err(TransactionAmountError);
        }

        Ok(Self(value))
    }

    pub fn value(self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transaction amount must not be zero")]
pub struct TransactionAmountError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionEffect {
    Standard,
    Reversal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionNominalAmount(Decimal);

impl TransactionNominalAmount {
    pub fn new(value: Decimal) -> Result<Self, TransactionNominalAmountError> {
        const MAXIMUM_EXCLUSIVE: i64 = 10_000_000_000_000_000;

        if value <= Decimal::ZERO || value.scale() > 4 || value >= Decimal::from(MAXIMUM_EXCLUSIVE)
        {
            return Err(TransactionNominalAmountError);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transaction amount must be positive with at most 16 integer and 4 fractional digits")]
pub struct TransactionNominalAmountError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionAmounts {
    pub nominal: TransactionNominalAmount,
    pub effect: TransactionEffect,
    pub economic: Decimal,
    pub account: TransactionAmount,
}

impl TransactionAmounts {
    pub fn from_input(
        nature: TransactionNature,
        effect: TransactionEffect,
        nominal: TransactionNominalAmount,
        account_kind: AccountKind,
    ) -> Result<Self, TransactionAmountsError> {
        let nominal_value = nominal.value();
        let economic = match (nature, effect) {
            (TransactionNature::Income, TransactionEffect::Standard)
            | (TransactionNature::Expense, TransactionEffect::Reversal) => nominal_value,
            (TransactionNature::Income, TransactionEffect::Reversal)
            | (TransactionNature::Expense, TransactionEffect::Standard) => -nominal_value,
            (TransactionNature::Transfer, _) => return Err(TransactionAmountsError),
        };
        let account_value = if account_kind.is_liability() {
            -economic
        } else {
            economic
        };

        Ok(Self {
            nominal,
            effect,
            economic,
            account: TransactionAmount::new(account_value)
                .expect("a positive nominal amount always produces a non-zero account amount"),
        })
    }

    pub fn from_stored(
        nature: TransactionNature,
        account_amount: TransactionAmount,
        account_kind: AccountKind,
    ) -> Result<Self, TransactionAmountsError> {
        if nature == TransactionNature::Transfer {
            return Err(TransactionAmountsError);
        }
        let account_value = account_amount.value();
        let economic = if account_kind.is_liability() {
            -account_value
        } else {
            account_value
        };
        let effect = match (nature, economic.is_sign_positive()) {
            (TransactionNature::Income, true) | (TransactionNature::Expense, false) => {
                TransactionEffect::Standard
            }
            (TransactionNature::Income, false) | (TransactionNature::Expense, true) => {
                TransactionEffect::Reversal
            }
            (TransactionNature::Transfer, _) => return Err(TransactionAmountsError),
        };
        let nominal =
            TransactionNominalAmount::new(economic.abs()).map_err(|_| TransactionAmountsError)?;

        Ok(Self {
            nominal,
            effect,
            economic,
            account: account_amount,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transaction amounts require an income or expense nature")]
pub struct TransactionAmountsError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionLabel(String);

impl TransactionLabel {
    pub fn new(value: impl AsRef<str>) -> Result<Self, TransactionLabelError> {
        let value = value.as_ref().trim();
        if value.is_empty() || value.chars().count() > 500 {
            return Err(TransactionLabelError);
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transaction label must contain between 1 and 500 characters")]
pub struct TransactionLabelError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionNature {
    Income,
    Expense,
    Transfer,
}

impl TransactionNature {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
            Self::Transfer => "transfer",
        }
    }
}

impl TryFrom<&str> for TransactionNature {
    type Error = TransactionValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "income" => Ok(Self::Income),
            "expense" => Ok(Self::Expense),
            "transfer" => Ok(Self::Transfer),
            _ => Err(TransactionValueError::new("transaction nature", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionSource {
    Manual,
    Import,
}

impl TransactionSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Import => "import",
        }
    }
}

impl TryFrom<&str> for TransactionSource {
    type Error = TransactionValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "manual" => Ok(Self::Manual),
            "import" => Ok(Self::Import),
            _ => Err(TransactionValueError::new("transaction source", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferRole {
    Source,
    Destination,
}

impl TransferRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Destination => "destination",
        }
    }
}

impl TryFrom<&str> for TransferRole {
    type Error = TransferValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "source" => Ok(Self::Source),
            "destination" => Ok(Self::Destination),
            _ => Err(TransferValueError::new("transfer role", value)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid {field}: {value}")]
pub struct TransactionValueError {
    field: &'static str,
    value: String,
}

impl TransactionValueError {
    fn new(field: &'static str, value: &str) -> Self {
        Self {
            field,
            value: value.to_owned(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid {field}: {value}")]
pub struct TransferValueError {
    field: &'static str,
    value: String,
}

impl TransferValueError {
    fn new(field: &'static str, value: &str) -> Self {
        Self {
            field,
            value: value.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionNote(String);

impl TransactionNote {
    pub fn new(value: impl AsRef<str>) -> Result<Self, TransactionNoteError> {
        let value = value.as_ref().trim();

        if value.is_empty() || value.chars().count() > 2_000 {
            return Err(TransactionNoteError);
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transaction note must contain between 1 and 2000 characters")]
pub struct TransactionNoteError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionDetails {
    Income {
        category_id: Option<CategoryId>,
    },
    Expense {
        category_id: Option<CategoryId>,
    },
    Transfer {
        transfer_id: TransferId,
        role: TransferRole,
    },
}

impl TransactionDetails {
    pub fn nature(&self) -> TransactionNature {
        match self {
            Self::Income { .. } => TransactionNature::Income,
            Self::Expense { .. } => TransactionNature::Expense,
            Self::Transfer { .. } => TransactionNature::Transfer,
        }
    }

    pub fn category_id(&self) -> Option<CategoryId> {
        match self {
            Self::Income { category_id } | Self::Expense { category_id } => *category_id,
            Self::Transfer { .. } => None,
        }
    }

    pub fn transfer_id(&self) -> Option<TransferId> {
        match self {
            Self::Income { .. } | Self::Expense { .. } => None,
            Self::Transfer { transfer_id, .. } => Some(*transfer_id),
        }
    }

    pub fn transfer_role(&self) -> Option<TransferRole> {
        match self {
            Self::Income { .. } | Self::Expense { .. } => None,
            Self::Transfer { role, .. } => Some(*role),
        }
    }
}

pub struct Transaction {
    pub id: TransactionId,
    pub account_id: AccountId,
    pub household_id: HouseholdId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub amount: TransactionAmount,
    pub details: TransactionDetails,
    pub source: TransactionSource,
    pub note: Option<TransactionNote>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

pub struct NewTransaction {
    pub id: TransactionId,
    pub account_id: AccountId,
    pub household_id: HouseholdId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub amount: TransactionAmount,
    pub details: TransactionDetails,
    pub source: TransactionSource,
    pub note: Option<TransactionNote>,
}

pub struct TransactionUpdate {
    pub id: TransactionId,
    pub household_id: HouseholdId,
    pub account_id: AccountId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub amount: TransactionAmount,
    pub details: TransactionDetails,
    pub note: Option<TransactionNote>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_amount_accepts_positive_and_negative_values() {
        let positive = Decimal::new(1250, 2);
        let negative = Decimal::new(-1250, 2);

        assert_eq!(TransactionAmount::new(positive).unwrap().value(), positive);
        assert_eq!(TransactionAmount::new(negative).unwrap().value(), negative);
    }

    #[test]
    fn transaction_amount_rejects_zero() {
        assert!(TransactionAmount::new(Decimal::ZERO).is_err());
    }

    #[test]
    fn nominal_amount_requires_positive_database_precision() {
        assert!(TransactionNominalAmount::new(Decimal::new(4250, 2)).is_ok());
        assert!(TransactionNominalAmount::new(Decimal::ZERO).is_err());
        assert!(TransactionNominalAmount::new(Decimal::new(-1, 0)).is_err());
        assert!(TransactionNominalAmount::new(Decimal::new(1, 5)).is_err());
        assert!(TransactionNominalAmount::new(Decimal::from(10_000_000_000_000_000_i64)).is_err());
    }

    #[test]
    fn transaction_amounts_derive_asset_and_liability_signs() {
        let nominal = TransactionNominalAmount::new(Decimal::new(4250, 2)).unwrap();
        let asset = TransactionAmounts::from_input(
            TransactionNature::Expense,
            TransactionEffect::Standard,
            nominal,
            AccountKind::Checking,
        )
        .unwrap();
        let liability = TransactionAmounts::from_input(
            TransactionNature::Expense,
            TransactionEffect::Standard,
            nominal,
            AccountKind::CreditCard,
        )
        .unwrap();

        assert_eq!(asset.economic, Decimal::new(-4250, 2));
        assert_eq!(asset.account.value(), Decimal::new(-4250, 2));
        assert_eq!(liability.economic, Decimal::new(-4250, 2));
        assert_eq!(liability.account.value(), Decimal::new(4250, 2));
    }

    #[test]
    fn stored_reversal_round_trips_to_nominal_semantics() {
        let stored = TransactionAmount::new(Decimal::new(2500, 2)).unwrap();
        let amounts = TransactionAmounts::from_stored(
            TransactionNature::Expense,
            stored,
            AccountKind::Checking,
        )
        .unwrap();

        assert_eq!(amounts.nominal.value(), Decimal::new(2500, 2));
        assert_eq!(amounts.effect, TransactionEffect::Reversal);
        assert_eq!(amounts.economic, Decimal::new(2500, 2));
    }

    #[test]
    fn transaction_label_is_trimmed() {
        assert_eq!(
            TransactionLabel::new("  Grocery store  ").unwrap().as_str(),
            "Grocery store"
        );
    }

    #[test]
    fn transaction_label_rejects_invalid_values() {
        assert!(TransactionLabel::new(" ").is_err());
        assert!(TransactionLabel::new("x".repeat(501)).is_err());
    }

    #[test]
    fn transaction_nature_uses_database_values() {
        assert_eq!(
            TransactionNature::try_from("income").unwrap(),
            TransactionNature::Income
        );
        assert_eq!(TransactionNature::Expense.as_str(), "expense");
        assert_eq!(TransactionNature::Transfer.as_str(), "transfer");
        assert!(TransactionNature::try_from("unknown").is_err());
    }

    #[test]
    fn transaction_source_uses_database_values() {
        assert_eq!(
            TransactionSource::try_from("manual").unwrap(),
            TransactionSource::Manual
        );
        assert_eq!(TransactionSource::Import.as_str(), "import");
        assert!(TransactionSource::try_from("unknown").is_err());
    }

    #[test]
    fn transfer_role_uses_database_values() {
        assert_eq!(
            TransferRole::try_from("source").unwrap(),
            TransferRole::Source
        );
        assert_eq!(
            TransferRole::try_from("destination").unwrap(),
            TransferRole::Destination
        );
        assert_eq!(TransferRole::Source.as_str(), "source");
        assert!(TransferRole::try_from("unknown").is_err());
    }

    #[test]
    fn transaction_note_is_trimmed() {
        assert_eq!(
            TransactionNote::new("  Monthly payment  ")
                .unwrap()
                .as_str(),
            "Monthly payment"
        );
    }

    #[test]
    fn transaction_note_rejects_invalid_values() {
        assert!(TransactionNote::new(" ").is_err());
        assert!(TransactionNote::new("x".repeat(2_001)).is_err());
    }

    #[test]
    fn income_details_expose_only_the_category() {
        let category_id = CategoryId::new();
        let details = TransactionDetails::Income {
            category_id: Some(category_id),
        };

        assert_eq!(details.nature(), TransactionNature::Income);
        assert_eq!(details.category_id(), Some(category_id));
        assert_eq!(details.transfer_id(), None);
        assert_eq!(details.transfer_role(), None);
    }

    #[test]
    fn expense_details_expose_only_the_category() {
        let category_id = CategoryId::new();
        let details = TransactionDetails::Expense {
            category_id: Some(category_id),
        };

        assert_eq!(details.nature(), TransactionNature::Expense);
        assert_eq!(details.category_id(), Some(category_id));
        assert_eq!(details.transfer_id(), None);
        assert_eq!(details.transfer_role(), None);
    }

    #[test]
    fn transfer_details_expose_only_the_transfer() {
        let transfer_id = TransferId::new();
        let details = TransactionDetails::Transfer {
            transfer_id,
            role: TransferRole::Destination,
        };

        assert_eq!(details.nature(), TransactionNature::Transfer);
        assert_eq!(details.category_id(), None);
        assert_eq!(details.transfer_id(), Some(transfer_id));
        assert_eq!(details.transfer_role(), Some(TransferRole::Destination));
    }
}
