use std::{fmt, str::FromStr};

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use crate::{
    features::transactions::domain::TransactionNature,
    types::{CategoryId, CurrencyCode},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Month {
    first_day: NaiveDate,
    next_month_first_day: NaiveDate,
}

impl Month {
    pub fn first_day(self) -> NaiveDate {
        self.first_day
    }

    pub fn next_month_first_day(self) -> NaiveDate {
        self.next_month_first_day
    }

    pub fn last_day(self) -> NaiveDate {
        self.next_month_first_day
            .pred_opt()
            .expect("a valid month always has a previous calendar day")
    }
}

impl FromStr for Month {
    type Err = MonthError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = value.as_bytes();
        if bytes.len() != 7
            || bytes[4] != b'-'
            || !bytes[..4].iter().all(u8::is_ascii_digit)
            || !bytes[5..].iter().all(u8::is_ascii_digit)
        {
            return Err(MonthError);
        }

        let first_day = NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d")
            .map_err(|_| MonthError)?;
        let (next_year, next_month) = if first_day.month() == 12 {
            (first_day.year().checked_add(1).ok_or(MonthError)?, 1)
        } else {
            (first_day.year(), first_day.month() + 1)
        };
        let next_month_first_day =
            NaiveDate::from_ymd_opt(next_year, next_month, 1).ok_or(MonthError)?;

        Ok(Self {
            first_day,
            next_month_first_day,
        })
    }
}

impl fmt::Display for Month {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}",
            self.first_day.year(),
            self.first_day.month()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("month must use the YYYY-MM format and identify a valid calendar month")]
pub struct MonthError;

#[derive(Clone)]
pub struct MonthlyFlowGroup {
    pub nature: TransactionNature,
    pub category_id: Option<CategoryId>,
    pub category_name: Option<String>,
    pub amount: Decimal,
    pub transaction_count: i64,
}

pub struct MonthlyFlowCategory {
    pub category_id: Option<CategoryId>,
    pub category_name: Option<String>,
    pub amount: Decimal,
    pub transaction_count: i64,
}

pub struct MonthlyFlowSection {
    pub total: Decimal,
    pub transaction_count: i64,
    pub categories: Vec<MonthlyFlowCategory>,
}

impl MonthlyFlowSection {
    pub fn from_groups(groups: impl Iterator<Item = MonthlyFlowGroup>) -> Self {
        let mut categories: Vec<_> = groups
            .map(|group| MonthlyFlowCategory {
                category_id: group.category_id,
                category_name: group.category_name,
                amount: group.amount,
                transaction_count: group.transaction_count,
            })
            .collect();

        categories.sort_by(|left, right| {
            right
                .amount
                .cmp(&left.amount)
                .then_with(|| {
                    left.category_name
                        .is_none()
                        .cmp(&right.category_name.is_none())
                })
                .then_with(|| left.category_name.cmp(&right.category_name))
        });

        Self {
            total: categories
                .iter()
                .fold(Decimal::new(0, 4), |total, category| {
                    total + category.amount
                }),
            transaction_count: categories
                .iter()
                .map(|category| category.transaction_count)
                .sum(),
            categories,
        }
    }
}

pub struct MonthlyFlowReport {
    pub month: Month,
    pub currency: CurrencyCode,
    pub income: MonthlyFlowSection,
    pub expenses: MonthlyFlowSection,
    pub net_flow: Decimal,
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn month_exposes_calendar_boundaries() {
        let month = "2024-02".parse::<Month>().unwrap();

        assert_eq!(month.to_string(), "2024-02");
        assert_eq!(
            month.first_day(),
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
        );
        assert_eq!(
            month.last_day(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
        assert_eq!(
            month.next_month_first_day(),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
    }

    #[test]
    fn month_rejects_invalid_or_non_canonical_values() {
        for value in ["2026-8", "26-08", "2026/08", "2026-13", "invalid"] {
            assert!(
                value.parse::<Month>().is_err(),
                "{value} should be rejected"
            );
        }
    }

    #[test]
    fn section_sums_and_sorts_groups() {
        let named_category_id = CategoryId::new();
        let section = MonthlyFlowSection::from_groups(
            [
                MonthlyFlowGroup {
                    nature: TransactionNature::Expense,
                    category_id: None,
                    category_name: None,
                    amount: Decimal::new(2500, 2),
                    transaction_count: 2,
                },
                MonthlyFlowGroup {
                    nature: TransactionNature::Expense,
                    category_id: Some(named_category_id),
                    category_name: Some("Food".to_owned()),
                    amount: Decimal::new(5000, 2),
                    transaction_count: 1,
                },
            ]
            .into_iter(),
        );

        assert_eq!(section.total, Decimal::new(7500, 2));
        assert_eq!(section.transaction_count, 3);
        assert_eq!(section.categories[0].category_id, Some(named_category_id));
        assert_eq!(section.categories[1].category_id, None);
    }
}
