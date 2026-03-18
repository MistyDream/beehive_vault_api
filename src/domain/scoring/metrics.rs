use crate::domain::market::enums::MetricCategory;

use super::formula::MetricRankData;

/// A metric extracted from an external source with its rank data and scoring direction.
pub struct ExtractedMetric {
    pub key: String,
    pub higher_is_better: bool,
    pub rank_data: MetricRankData,
}

pub fn category_for_metric(key: &str) -> Option<MetricCategory> {
    match key {
        "pettm" | "peg" | "ev2ebitda" | "ps" | "pb" | "pfcf" => Some(MetricCategory::Valuation),
        "rvn_growth_3y" | "ebitda_growth_3y" | "cashflow_growth_3y" | "earning_growth_3y" | "book_growth_3y" => Some(MetricCategory::Growth),
        "roic" | "oprt_margain" | "FCFmargin" | "roa" | "net_margain" => Some(MetricCategory::Profitability),
        "interest_coverage" | "debt2ebitda" | "quick_ratio" | "cash2debt" | "equity2asset" => Some(MetricCategory::FinancialHealth),
        "ForwardDividendYield" | "buyback_yield" | "dividend_growth_3y" | "payout" | "shareholder_yield" => Some(MetricCategory::InvestorReturn),
        _ => None,
    }
}
