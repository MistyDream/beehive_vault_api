//! ISO 6166 securities identifier — domain value object.
//!
//! An ISIN is the canonical, internationally standardized identifier for a
//! tradable security. Format: 2 uppercase letters (ISO 3166-1 alpha-2 country
//! code), 9 alphanumerics (national security identifier), and 1 numeric check
//! digit. We validate the **format** only — the Luhn-mod-10 check digit is
//! intentionally not verified, since the upstream provider (yfinance / manual
//! input) is authoritative on identity and recomputing the digit would reject
//! some legitimate-but-unconventionally-issued ISINs.
//!
//! Used as the public URL identifier for stocks (`/v1/stocks/{isin}`) and as
//! the `isin` field on the `Stock` entity. Constructed only via `try_new`
//! (or its `TryFrom`/`FromStr` aliases) so a `Isin` value in scope is a
//! statically-validated invariant — repositories, services, and controllers
//! never need to re-check the format.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Isin(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IsinParseError {
    #[error("isin must be exactly 12 characters")]
    WrongLength,
    #[error(
        "isin format must be ISO 6166 (2 uppercase letters, 9 alphanumerics, 1 digit)"
    )]
    BadFormat,
}

impl Isin {
    pub fn try_new(raw: &str) -> Result<Self, IsinParseError> {
        if raw.len() != 12 {
            return Err(IsinParseError::WrongLength);
        }
        let mut chars = raw.chars();
        let valid = chars.by_ref().take(2).all(|c| c.is_ascii_uppercase())
            && chars
                .by_ref()
                .take(9)
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            && chars.next().is_some_and(|c| c.is_ascii_digit());
        if !valid {
            return Err(IsinParseError::BadFormat);
        }
        Ok(Isin(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for Isin {
    type Err = IsinParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

impl TryFrom<&str> for Isin {
    type Error = IsinParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_new(s)
    }
}

impl TryFrom<String> for Isin {
    type Error = IsinParseError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_new(&s)
    }
}

impl fmt::Display for Isin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// Custom Deserialize that runs validation — so `serde_json` parsing of a body
// containing an ISIN field rejects invalid values at deserialization time
// rather than letting them propagate into the domain.
impl<'de> Deserialize<'de> for Isin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Isin::try_new(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_isin() {
        assert_eq!(Isin::try_new("US0378331005").unwrap().as_str(), "US0378331005");
        assert_eq!(Isin::try_new("FR0000131906").unwrap().as_str(), "FR0000131906");
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(Isin::try_new("US037833100"), Err(IsinParseError::WrongLength));
        assert_eq!(Isin::try_new("US03783310055"), Err(IsinParseError::WrongLength));
    }

    #[test]
    fn rejects_lowercase_country() {
        assert_eq!(Isin::try_new("us0378331005"), Err(IsinParseError::BadFormat));
    }

    #[test]
    fn rejects_non_digit_check() {
        assert_eq!(Isin::try_new("US037833100A"), Err(IsinParseError::BadFormat));
    }

    #[test]
    fn rejects_invalid_country() {
        assert_eq!(Isin::try_new("U10378331005"), Err(IsinParseError::BadFormat));
    }

    #[test]
    fn from_str_alias_works() {
        let isin: Isin = "US0378331005".parse().unwrap();
        assert_eq!(isin.as_str(), "US0378331005");
    }

    #[test]
    fn serialize_is_transparent_string() {
        let isin = Isin::try_new("US0378331005").unwrap();
        let json = serde_json::to_string(&isin).unwrap();
        assert_eq!(json, "\"US0378331005\"");
    }

    #[test]
    fn deserialize_validates_format() {
        let valid: Isin = serde_json::from_str("\"US0378331005\"").unwrap();
        assert_eq!(valid.as_str(), "US0378331005");
        let invalid: Result<Isin, _> = serde_json::from_str("\"bogus\"");
        assert!(invalid.is_err());
    }
}
