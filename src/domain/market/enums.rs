use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricCategory {
    Valuation,
    Profitability,
    Growth,
    FinancialHealth,
    InvestorReturn,
}

impl MetricCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricCategory::Valuation => "valuation",
            MetricCategory::Profitability => "profitability",
            MetricCategory::Growth => "growth",
            MetricCategory::FinancialHealth => "financial_health",
            MetricCategory::InvestorReturn => "investor_return",
        }
    }
}

impl TryFrom<&str> for MetricCategory {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "valuation" => Ok(MetricCategory::Valuation),
            "profitability" => Ok(MetricCategory::Profitability),
            "growth" => Ok(MetricCategory::Growth),
            "financial_health" => Ok(MetricCategory::FinancialHealth),
            "investor_return" => Ok(MetricCategory::InvestorReturn),
            other => Err(format!("unknown MetricCategory: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDataType {
    Percent,
    Multiple,
    Currency,
    Bool,
}

impl MetricDataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricDataType::Percent => "percent",
            MetricDataType::Multiple => "multiple",
            MetricDataType::Currency => "currency",
            MetricDataType::Bool => "bool",
        }
    }
}

impl TryFrom<&str> for MetricDataType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "percent" => Ok(MetricDataType::Percent),
            "multiple" => Ok(MetricDataType::Multiple),
            "currency" => Ok(MetricDataType::Currency),
            "bool" => Ok(MetricDataType::Bool),
            other => Err(format!("unknown MetricDataType: '{other}'")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricPeriod {
    FY,
    TTM,
    Q,
}

impl MetricPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricPeriod::FY => "FY",
            MetricPeriod::TTM => "TTM",
            MetricPeriod::Q => "Q",
        }
    }
}

impl TryFrom<&str> for MetricPeriod {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "FY" => Ok(MetricPeriod::FY),
            "TTM" => Ok(MetricPeriod::TTM),
            "Q" => Ok(MetricPeriod::Q),
            other => Err(format!("unknown MetricPeriod: '{other}'")),
        }
    }
}
