use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{CategoryId, HouseholdId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CategoryKind {
    Income,
    Expense,
}

impl CategoryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Income => "income",
            Self::Expense => "expense",
        }
    }
}

impl TryFrom<&str> for CategoryKind {
    type Error = CategoryValueError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "income" => Ok(Self::Income),
            "expense" => Ok(Self::Expense),
            _ => Err(CategoryValueError::new("category kind", value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryName(String);

impl CategoryName {
    pub fn new(value: impl AsRef<str>) -> Result<Self, CategoryNameError> {
        let value = value.as_ref().trim();
        if value.is_empty() || value.chars().count() > 100 {
            return Err(CategoryNameError);
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
#[error("category name must contain between 1 and 100 characters")]
pub struct CategoryNameError;

#[derive(Debug, thiserror::Error)]
#[error("invalid {field}: {value}")]
pub struct CategoryValueError {
    field: &'static str,
    value: String,
}

impl CategoryValueError {
    fn new(field: &'static str, value: &str) -> Self {
        Self {
            field,
            value: value.to_owned(),
        }
    }
}

pub struct Category {
    pub id: CategoryId,
    pub household_id: HouseholdId,
    pub name: CategoryName,
    pub kind: CategoryKind,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewCategory {
    pub id: CategoryId,
    pub household_id: HouseholdId,
    pub name: CategoryName,
    pub kind: CategoryKind,
}

#[cfg(test)]
mod tests {
    use super::CategoryName;

    #[test]
    fn category_name_is_trimmed() {
        assert_eq!(
            CategoryName::new("  Pet care  ").unwrap().as_str(),
            "Pet care"
        );
    }

    #[test]
    fn category_name_rejects_invalid_values() {
        assert!(CategoryName::new(" ").is_err());
        assert!(CategoryName::new("x".repeat(101)).is_err());
    }
}
