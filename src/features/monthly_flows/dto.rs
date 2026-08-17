use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::types::{CategoryId, CurrencyCode};

use super::domain::{MonthlyFlowCategory, MonthlyFlowReport, MonthlyFlowSection};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyFlowResponse {
    month: String,
    date_from: NaiveDate,
    date_to: NaiveDate,
    currency: CurrencyCode,
    income: MonthlyFlowSectionResponse,
    expenses: MonthlyFlowSectionResponse,
    net_flow: Decimal,
}

impl From<MonthlyFlowReport> for MonthlyFlowResponse {
    fn from(report: MonthlyFlowReport) -> Self {
        Self {
            month: report.month.to_string(),
            date_from: report.month.first_day(),
            date_to: report.month.last_day(),
            currency: report.currency,
            income: report.income.into(),
            expenses: report.expenses.into(),
            net_flow: report.net_flow,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonthlyFlowSectionResponse {
    total: Decimal,
    transaction_count: i64,
    categories: Vec<MonthlyFlowCategoryResponse>,
}

impl From<MonthlyFlowSection> for MonthlyFlowSectionResponse {
    fn from(section: MonthlyFlowSection) -> Self {
        Self {
            total: section.total,
            transaction_count: section.transaction_count,
            categories: section.categories.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonthlyFlowCategoryResponse {
    category_id: Option<CategoryId>,
    category_name: Option<String>,
    amount: Decimal,
    transaction_count: i64,
}

impl From<MonthlyFlowCategory> for MonthlyFlowCategoryResponse {
    fn from(category: MonthlyFlowCategory) -> Self {
        Self {
            category_id: category.category_id,
            category_name: category.category_name,
            amount: category.amount,
            transaction_count: category.transaction_count,
        }
    }
}
