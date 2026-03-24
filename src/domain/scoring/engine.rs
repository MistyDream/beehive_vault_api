use std::collections::HashMap;

use crate::domain::market::enums::MetricCategory;

use super::formula::{bracket_sub_weights, compute_bracket_score, compute_metric_score, compute_payout_score, payout_sub_weights};
use super::metrics::{ExtractedMetric, category_for_metric, brackets_for};

/// Full scoring result for a stock.
pub struct ScoringResult {
    pub global_score: f64,
    pub categories: Vec<CategoryScore>,
}

pub struct CategoryScore {
    pub category: MetricCategory,
    pub score: f64,
    pub weight: f64,
    pub indicators: Vec<IndicatorResult>,
}

pub struct IndicatorResult {
    pub metric_key: String,
    pub score: f64,
    pub absolute_score: Option<f64>,
    pub sector_score: f64,
    pub historical_score: f64,
}

/// Combine sector and historical sub-scores using a divergence-aware formula.
///
/// When scores converge → behaves like a simple average.
/// When scores diverge → the stronger signal is progressively protected.
///
/// Formula: M + 0.35 × w(D) × B
///   M = (S + H) / 2
///   D = |S - H|
///   B = max(0, S - 50, H - 50)
///   w(D) = (D / 100)^1.5
fn combine_scores(sector: f64, historical: f64) -> f64 {
    let m = (sector + historical) / 2.0;
    let d = (sector - historical).abs();
    let b = f64::max(0.0, f64::max(sector - 50.0, historical - 50.0));
    let w = (d / 100.0_f64).powf(1.5);
    (m + 0.35 * w * b).clamp(0.0, 100.0)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn score_metric(metric: &ExtractedMetric) -> IndicatorResult {
    if metric.key == "payout" {
        let s = compute_payout_score(&metric.rank_data);
        let (w_a, w_s, w_h) = payout_sub_weights(
            metric.rank_data.industry_med,
            metric.rank_data.med,
        );
        return IndicatorResult {
            metric_key: metric.key.clone(),
            score: round2(s.absolute_score * w_a + s.sector_score * w_s + s.historical_score * w_h),
            absolute_score: Some(s.absolute_score),
            sector_score: s.sector_score,
            historical_score: s.historical_score,
        };
    }
    if let Some(brackets) = brackets_for(&metric.key) {
        let s = compute_bracket_score(&metric.rank_data, &brackets, metric.higher_is_better);
        let (w_a, w_s, w_h) = bracket_sub_weights(
            metric.rank_data.industry_med,
            metric.rank_data.med,
            &brackets,
        );
        IndicatorResult {
            metric_key: metric.key.clone(),
            score: round2(s.absolute_score * w_a + s.sector_score * w_s + s.historical_score * w_h),
            absolute_score: Some(s.absolute_score),
            sector_score: s.sector_score,
            historical_score: s.historical_score,
        }
    } else {
        let s = compute_metric_score(&metric.rank_data, metric.higher_is_better);
        IndicatorResult {
            metric_key: metric.key.clone(),
            score: round2(combine_scores(s.sector_score, s.historical_score)),
            absolute_score: None,
            sector_score: s.sector_score,
            historical_score: s.historical_score,
        }
    }
}

/// Compute the full scoring result from extracted metrics.
pub fn compute_scoring(metrics: &[ExtractedMetric]) -> ScoringResult {
    let mut by_category: HashMap<MetricCategory, Vec<IndicatorResult>> = HashMap::new();

    for metric in metrics {
        let Some(category) = category_for_metric(&metric.key) else { continue };
        by_category.entry(category).or_default().push(score_metric(metric));
    }

    let num_categories = by_category.len() as f64;
    let weight = if num_categories > 0.0 { 1.0 / num_categories } else { 0.0 };
    let weight = (weight * 100.0).round() / 100.0;

    let categories: Vec<CategoryScore> = by_category
        .into_iter()
        .map(|(category, indicators)| {
            let avg = indicators.iter().map(|i| i.score).sum::<f64>() / indicators.len() as f64;
            let avg = (avg * 100.0).round() / 100.0;
            CategoryScore { category, score: avg, weight, indicators }
        })
        .collect();

    let global_score = if categories.is_empty() {
        0.0
    } else {
        let sum: f64 = categories.iter().map(|c| c.score * c.weight).sum();
        (sum * 100.0).round() / 100.0
    };

    ScoringResult { global_score, categories }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_nvidia_per() {
        let score = combine_scores(27.0, 85.0);
        assert!((score - 61.4).abs() < 0.5, "NVIDIA PER: {score}");
    }

    #[test]
    fn test_combine_value_company() {
        let score = combine_scores(100.0, 40.0);
        assert!((score - 78.1).abs() < 0.5, "Value company: {score}");
    }

    #[test]
    fn test_combine_convergence_good() {
        let score = combine_scores(90.0, 85.0);
        assert!((score - 87.5).abs() < 0.5, "Good convergence: {score}");
    }

    #[test]
    fn test_combine_convergence_bad() {
        let score = combine_scores(15.0, 20.0);
        assert!((score - 17.5).abs() < 0.5, "Bad convergence: {score}");
    }

    #[test]
    fn test_combine_neutral() {
        let score = combine_scores(60.0, 60.0);
        assert!((score - 60.0).abs() < f64::EPSILON, "Neutral: {score}");
    }

    #[test]
    fn test_combine_perfect() {
        let score = combine_scores(100.0, 100.0);
        assert!((score - 100.0).abs() < f64::EPSILON, "Perfect: {score}");
    }

    #[test]
    fn test_combine_zero() {
        let score = combine_scores(0.0, 0.0);
        assert!((score - 0.0).abs() < f64::EPSILON, "Zero: {score}");
    }

    #[test]
    fn test_nvidia_investor_return() {
        use super::super::formula::MetricRankData;

        let metrics = vec![
            ExtractedMetric {
                key: "ForwardDividendYield".into(),
                higher_is_better: true,
                rank_data: MetricRankData { current: 0.03, med: 0.0, industry_med: 1.64 },
            },
            ExtractedMetric {
                key: "buyback_yield".into(),
                higher_is_better: true,
                rank_data: MetricRankData { current: 1.13, med: 0.89, industry_med: -0.15 },
            },
            ExtractedMetric {
                key: "dividend_growth_3y".into(),
                higher_is_better: true,
                rank_data: MetricRankData { current: 28.6, med: 14.5, industry_med: 10.2 },
            },
            ExtractedMetric {
                key: "payout".into(),
                higher_is_better: false,
                rank_data: MetricRankData { current: 0.01, med: 0.08, industry_med: 0.375 },
            },
            ExtractedMetric {
                key: "shareholder_yield".into(),
                higher_is_better: true,
                rank_data: MetricRankData { current: 1.18, med: 0.86, industry_med: 0.14 },
            },
        ];

        let result = compute_scoring(&metrics);

        for cat in &result.categories {
            println!("\n{:?} — score: {:.2}", cat.category, cat.score);
            for ind in &cat.indicators {
                println!(
                    "  {:25} score: {:6.2} | abs: {:>6} | sect: {:6.2} | hist: {:6.2}",
                    ind.metric_key,
                    ind.score,
                    ind.absolute_score.map_or("-".into(), |v| format!("{v:.2}")),
                    ind.sector_score,
                    ind.historical_score,
                );
            }
        }
        println!("\nGlobal: {:.2}", result.global_score);
    }
}
