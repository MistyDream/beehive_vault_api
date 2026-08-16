use std::{fmt, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use uuid::Uuid;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
        #[serde(transparent)]
        #[sqlx(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn value(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(value).map(Self)
            }
        }
    };
}

uuid_id!(HouseholdId);
uuid_id!(InstitutionId);
uuid_id!(AccountId);
uuid_id!(BalanceSnapshotId);
uuid_id!(CategoryId);
uuid_id!(TransactionId);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, sqlx::Type)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    pub fn new(value: impl AsRef<str>) -> Result<Self, CurrencyCodeError> {
        let normalized = value.as_ref().trim().to_uppercase();
        if normalized.len() != 3
            || !normalized
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            return Err(CurrencyCodeError);
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for CurrencyCode {
    type Err = CurrencyCodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for CurrencyCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("currency must be a three-letter ASCII code")]
pub struct CurrencyCodeError;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct AccountBalance(Decimal);

impl AccountBalance {
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    pub fn value(self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Money {
    amount: AccountBalance,
    currency: CurrencyCode,
}

impl Money {
    pub fn new(amount: AccountBalance, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    pub fn amount(&self) -> AccountBalance {
        self.amount
    }

    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("timezone must be a valid IANA time zone")]
pub struct TimeZoneIdError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct TimeZoneId(String);

impl TimeZoneId {
    pub fn new(value: impl AsRef<str>) -> Result<Self, TimeZoneIdError> {
        let value = value.as_ref().trim();

        value
            .parse::<chrono_tz::Tz>()
            .map_err(|_| TimeZoneIdError)?;

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn date_at(&self, instant: DateTime<Utc>) -> Result<NaiveDate, TimeZoneIdError> {
        let timezone = self
            .as_str()
            .parse::<chrono_tz::Tz>()
            .map_err(|_| TimeZoneIdError)?;

        Ok(instant.with_timezone(&timezone).date_naive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_code_normalizes_valid_input() {
        assert_eq!(CurrencyCode::new(" eur ").unwrap().as_str(), "EUR");
    }

    #[test]
    fn currency_code_rejects_invalid_input() {
        assert!(CurrencyCode::new("EU").is_err());
        assert!(CurrencyCode::new("€UR").is_err());
    }

    #[test]
    fn household_id_round_trips_through_json() {
        let household_id = HouseholdId::new();
        let json = serde_json::to_string(&household_id).unwrap();
        let decoded: HouseholdId = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, household_id);
    }

    #[test]
    fn time_zone_id_trims_valid_value() {
        let timezone = TimeZoneId::new(" Europe/Paris ").unwrap();

        assert_eq!(timezone.as_str(), "Europe/Paris");
    }

    #[test]
    fn time_zone_id_rejects_unknown_value() {
        assert!(TimeZoneId::new("Not/AZone").is_err());
    }

    #[test]
    fn time_zone_id_converts_utc_instant_to_local_date() {
        let timezone = TimeZoneId::new("Europe/Paris").unwrap();
        let instant = "2026-08-11T22:30:00Z".parse::<DateTime<Utc>>().unwrap();

        assert_eq!(
            timezone.date_at(instant).unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap()
        );
    }
}
