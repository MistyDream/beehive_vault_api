use std::collections::HashMap;

use crate::domain::market::enums::MetricCategory;

use super::formula::compute_metric_score;
use super::metrics::{ExtractedMetric, category_for_metric};

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

/// Compute the full scoring result from extracted metrics.
/// Uses divergence-aware combination for sector/historical and equal weights across categories.
pub fn compute_scoring(metrics: &[ExtractedMetric]) -> ScoringResult {
    let mut by_category: HashMap<MetricCategory, Vec<IndicatorResult>> = HashMap::new();

    for metric in metrics {
        let Some(category) = category_for_metric(&metric.key) else { continue };
        let scores = compute_metric_score(&metric.rank_data, metric.higher_is_better);
        let combined = (combine_scores(scores.sector_score, scores.historical_score) * 100.0).round() / 100.0;

        by_category.entry(category).or_default().push(IndicatorResult {
            metric_key: metric.key.clone(),
            score: combined,
            sector_score: scores.sector_score,
            historical_score: scores.historical_score,
        });
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
}
