use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::{
    features::{
        accounts::AccountKind,
        transactions::domain::{
            TransactionAmount, TransactionLabel, TransactionNote, TransferRole,
        },
    },
    types::{AccountId, HouseholdId, TransactionId, TransferId},
};

/// A strictly positive transfer magnitude used to derive both signed transaction amounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransferAmount(Decimal);

impl TransferAmount {
    pub fn new(value: Decimal) -> Result<Self, TransferAmountError> {
        if value <= Decimal::ZERO {
            return Err(TransferAmountError);
        }

        Ok(Self(value))
    }

    pub fn value(self) -> Decimal {
        self.0
    }

    pub fn signed_value(self, role: TransferRole, account_kind: AccountKind) -> Decimal {
        let increases_raw_balance = matches!(
            (role, account_kind.is_liability()),
            (TransferRole::Source, true) | (TransferRole::Destination, false)
        );

        if increases_raw_balance {
            self.0
        } else {
            -self.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("transfer amount must be greater than zero")]
pub struct TransferAmountError;

pub struct TransferMovement {
    pub transaction_id: TransactionId,
    pub account_id: AccountId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub amount: TransactionAmount,
    pub note: Option<TransactionNote>,
}

pub struct Transfer {
    pub id: TransferId,
    pub household_id: HouseholdId,
    pub source: TransferMovement,
    pub destination: TransferMovement,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Transfer {
    pub fn amount(&self) -> TransferAmount {
        TransferAmount(self.source.amount.value().abs())
    }
}

pub struct NewTransferMovement {
    pub transaction_id: TransactionId,
    pub account_id: AccountId,
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub amount: TransactionAmount,
    pub note: Option<TransactionNote>,
}

pub struct NewTransfer {
    pub id: TransferId,
    pub household_id: HouseholdId,
    pub source: NewTransferMovement,
    pub destination: NewTransferMovement,
}

pub struct TransferUpdate {
    pub id: TransferId,
    pub household_id: HouseholdId,
    pub source: NewTransferMovement,
    pub destination: NewTransferMovement,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount() -> TransferAmount {
        TransferAmount::new(Decimal::new(500, 0)).unwrap()
    }

    #[test]
    fn transfer_amount_must_be_positive() {
        assert!(TransferAmount::new(Decimal::ZERO).is_err());
        assert!(TransferAmount::new(Decimal::new(-1, 0)).is_err());
        assert!(TransferAmount::new(Decimal::new(1, 0)).is_ok());
    }

    #[test]
    fn asset_to_asset_uses_opposite_raw_amounts() {
        assert_eq!(
            amount().signed_value(TransferRole::Source, AccountKind::Checking),
            Decimal::new(-500, 0)
        );
        assert_eq!(
            amount().signed_value(TransferRole::Destination, AccountKind::Savings),
            Decimal::new(500, 0)
        );
    }

    #[test]
    fn liability_to_liability_uses_opposite_raw_amounts() {
        assert_eq!(
            amount().signed_value(TransferRole::Source, AccountKind::CreditCard),
            Decimal::new(500, 0)
        );
        assert_eq!(
            amount().signed_value(TransferRole::Destination, AccountKind::Loan),
            Decimal::new(-500, 0)
        );
    }

    #[test]
    fn asset_to_liability_uses_two_negative_raw_amounts() {
        assert_eq!(
            amount().signed_value(TransferRole::Source, AccountKind::Checking),
            Decimal::new(-500, 0)
        );
        assert_eq!(
            amount().signed_value(TransferRole::Destination, AccountKind::Loan),
            Decimal::new(-500, 0)
        );
    }

    #[test]
    fn liability_to_asset_uses_two_positive_raw_amounts() {
        assert_eq!(
            amount().signed_value(TransferRole::Source, AccountKind::Loan),
            Decimal::new(500, 0)
        );
        assert_eq!(
            amount().signed_value(TransferRole::Destination, AccountKind::Checking),
            Decimal::new(500, 0)
        );
    }
}
