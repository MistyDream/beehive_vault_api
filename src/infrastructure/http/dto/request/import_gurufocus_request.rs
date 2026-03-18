use std::collections::HashMap;

use chrono::Utc;
use serde::Deserialize;

use crate::domain::market::metric_value::NewMetricValue;
use crate::domain::scoring::{ExtractedMetric, MetricRankData};

const KNOWN_METRICS: &[(&str, bool)] = &[
    // Valuation (lower is better)
    ("pettm", false), ("peg", false), ("ev2ebitda", false), ("ps", false), ("pb", false), ("pfcf", false),
    // Growth (higher is better)
    ("rvn_growth_3y", true), ("ebitda_growth_3y", true), ("cashflow_growth_3y", true), ("earning_growth_3y", true), ("book_growth_3y", true),
    // Profitability (higher is better)
    ("roic", true), ("oprt_margain", true), ("FCFmargin", true), ("roa", true), ("net_margain", true),
    // Financial Health (higher is better, except debt2ebitda)
    ("interest_coverage", true), ("debt2ebitda", false), ("quick_ratio", true), ("cash2debt", true), ("equity2asset", true),
    // Investor Return (higher is better, except payout)
    ("ForwardDividendYield", true), ("buyback_yield", true), ("dividend_growth_3y", true), ("payout", false), ("shareholder_yield", true),
];

#[derive(Debug, Deserialize)]
pub struct ImportGurufocusRequest(pub HashMap<String, serde_json::Value>);

impl ImportGurufocusRequest {
    pub fn into_metric_values(&self, stock_id: i32) -> Vec<NewMetricValue> {
        let now = Utc::now().naive_utc();
        let today = Utc::now().date_naive();

        KNOWN_METRICS
            .iter()
            .filter_map(|&(key, _)| {
                let json_val = self.0.get(key)?;
                let value = json_val.as_f64()?;
                Some(NewMetricValue {
                    stock_id,
                    metric_key: key.to_string(),
                    period: "TTM".to_string(),
                    period_end: today,
                    value,
                    unit: None,
                    currency: None,
                    source: "gurufocus".to_string(),
                    fetched_at: now,
                })
            })
            .collect()
    }

    pub fn extract_rank_data(&self) -> Vec<ExtractedMetric> {
        KNOWN_METRICS
            .iter()
            .filter_map(|&(key, higher_is_better)| {
                let rank_key = format!("{key}_industry_rank");
                let rank = self.0.get(&rank_key)?;
                let rank_obj = rank.as_object()?;

                let current = rank_obj.get("current")?.as_f64()?;
                let med = rank_obj.get("med")?.as_f64()?;
                let industry_med = rank_obj.get("industry_med")?.as_f64()?;

                Some(ExtractedMetric {
                    key: key.to_string(),
                    higher_is_better,
                    rank_data: MetricRankData { current, med, industry_med },
                })
            })
            .collect()
    }
}
