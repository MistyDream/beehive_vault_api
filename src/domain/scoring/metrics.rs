use crate::domain::market::enums::MetricCategory;

use super::formula::{Brackets, MetricRankData};

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

/// Returns the absolute scoring brackets for a metric.
/// Thresholds [t0, t1, t2, t3] map to scores [20, 40, 60, 80].
pub fn brackets_for(key: &str) -> Option<Brackets> {
    let thresholds = match key {
        // Growth
        "rvn_growth_3y" => [-5.0, 0.0, 5.0, 12.0],
        "ebitda_growth_3y" => [-8.0, 0.0, 6.0, 14.0],
        "cashflow_growth_3y" => [-12.0, 0.0, 6.0, 16.0],
        "earning_growth_3y" => [-10.0, 0.0, 8.0, 18.0],
        "book_growth_3y" => [-3.0, 0.0, 4.0, 8.0],
        // Profitability
        "roic" => [3.0, 6.0, 10.0, 15.0],
        "oprt_margain" => [5.0, 10.0, 15.0, 22.0],
        "net_margain" => [3.0, 7.0, 12.0, 18.0],
        "FCFmargin" => [3.0, 6.0, 10.0, 16.0],
        "roa" => [2.0, 4.0, 7.0, 10.0],
        _ => return None,
    };
    Some(Brackets { thresholds })
}
