/// Input data needed to score a single metric.
pub struct MetricRankData {
    pub current: f64,
    pub med: f64,
    pub industry_med: f64,
}

/// Result of scoring a metric with 2 sub-scores (valuation-style).
pub struct MetricScore {
    pub sector_score: f64,
    pub historical_score: f64,
}

/// Result of scoring a growth metric with 3 sub-scores.
pub struct BracketMetricScore {
    pub absolute_score: f64,
    pub sector_score: f64,
    pub historical_score: f64,
}

/// Bracket thresholds for growth absolute scoring.
/// Defines 4 boundaries between 5 scoring bands (0-20, 20-40, 40-60, 60-80, 80-100).
pub struct Brackets {
    pub thresholds: [f64; 4],
}

/// Compute the sector and historical sub-scores for a metric.
///
/// Both sub-scores use the same formula:
/// - pivot (median) = 60 points
/// - 20% below median = 100 (capped)
/// - double the median = 0 (capped)
/// - linear interpolation between these points
pub fn compute_metric_score(data: &MetricRankData, higher_is_better: bool) -> MetricScore {
    MetricScore {
        sector_score: score_vs_median(data.current, data.industry_med, higher_is_better),
        historical_score: score_vs_median(data.current, data.med, higher_is_better),
    }
}

/// Compute the 3 sub-scores for a growth metric.
/// Uses normalized difference instead of ratio for sector/historical (handles negatives).
pub fn compute_bracket_score(data: &MetricRankData, brackets: &Brackets) -> BracketMetricScore {
    BracketMetricScore {
        absolute_score: score_from_brackets(data.current, brackets),
        sector_score: score_normalized_diff(data.current, data.industry_med),
        historical_score: score_normalized_diff(data.current, data.med),
    }
}

/// Score using normalized symmetric difference.
/// Works correctly with negative values (growth metrics).
///
/// d = (current - median) / (|current| + |median| + ε)
/// score = 60 + 40·d if d >= 0, 60 + 60·d if d < 0
fn score_normalized_diff(current: f64, median: f64) -> f64 {
    let denom = current.abs() + median.abs() + 1e-9;
    let d = (current - median) / denom;
    let score = if d >= 0.0 {
        60.0 + 40.0 * d
    } else {
        60.0 + 60.0 * d
    };
    (score.clamp(0.0, 100.0) * 100.0).round() / 100.0
}

/// Score a value against fixed brackets with linear interpolation.
///
/// Thresholds [t0, t1, t2, t3] map to scores [20, 40, 60, 80].
/// Floor and ceiling are derived from neighboring intervals:
///   floor = t0 - (t1 - t0) → score 0
///   ceiling = t3 + (t3 - t2) → score 100
fn score_from_brackets(current: f64, brackets: &Brackets) -> f64 {
    let [t0, t1, t2, t3] = brackets.thresholds;
    let floor = t0 - (t1 - t0);
    let ceiling = t3 + (t3 - t2);

    let score = if current <= floor {
        0.0
    } else if current <= t0 {
        20.0 * (current - floor) / (t0 - floor)
    } else if current <= t1 {
        20.0 + 20.0 * (current - t0) / (t1 - t0)
    } else if current <= t2 {
        40.0 + 20.0 * (current - t1) / (t2 - t1)
    } else if current <= t3 {
        60.0 + 20.0 * (current - t2) / (t3 - t2)
    } else if current <= ceiling {
        80.0 + 20.0 * (current - t3) / (ceiling - t3)
    } else {
        100.0
    };

    (score.clamp(0.0, 100.0) * 100.0).round() / 100.0
}

/// Minimum weight for any sub-score in bracket-based dynamic weighting.
const MIN_BRACKET_WEIGHT: f64 = 0.15;

/// Compute dynamic weights (absolute, sector, historical) for bracket-based metrics.
///
/// Sector and historical weights are inversely proportional to how well their
/// respective medians score in the brackets — a high-scoring median means the
/// comparison is less informative, so its weight decreases.
/// Absolute absorbs the remainder. All weights are floored at `MIN_BRACKET_WEIGHT`.
pub fn bracket_sub_weights(
    industry_med: f64,
    hist_med: f64,
    brackets: &Brackets,
) -> (f64, f64, f64) {
    let s_bracket = score_from_brackets(industry_med, brackets) / 100.0;
    let h_bracket = score_from_brackets(hist_med, brackets) / 100.0;

    // Inverse: low bracket score → median is weak → comparison more informative → higher weight
    let s_raw = 1.0 - s_bracket;
    let h_raw = 1.0 - h_bracket;
    let sh_sum = (s_raw + h_raw).max(1e-9);

    // Map total sector+historical raw [0, 2] → [2×MIN, 1−MIN]
    let sh_total = MIN_BRACKET_WEIGHT * 2.0
        + (1.0 - 3.0 * MIN_BRACKET_WEIGHT) * (sh_sum / 2.0);

    let mut s_w = sh_total * s_raw / sh_sum;
    let mut h_w = sh_total * h_raw / sh_sum;

    // Enforce individual floors
    s_w = s_w.max(MIN_BRACKET_WEIGHT);
    h_w = h_w.max(MIN_BRACKET_WEIGHT);
    let a_w = (1.0 - s_w - h_w).max(MIN_BRACKET_WEIGHT);

    // Normalize to sum = 1
    let total = a_w + s_w + h_w;
    (a_w / total, s_w / total, h_w / total)
}

fn score_vs_median(current: f64, median: f64, higher_is_better: bool) -> f64 {
    if median == 0.0 {
        return 60.0;
    }

    let ratio = if higher_is_better {
        median / current
    } else {
        current / median
    };

    let score = if ratio <= 0.80 {
        100.0
    } else if ratio <= 1.0 {
        100.0 - (ratio - 0.80) / 0.20 * 40.0
    } else if ratio <= 2.0 {
        60.0 - (ratio - 1.0) / 1.0 * 60.0
    } else {
        0.0
    };

    (score.clamp(0.0, 100.0) * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_nvidia_sector() {
        let score = score_vs_median(45.80, 29.56, false);
        assert!((score - 27.1).abs() < 0.5);
    }

    #[test]
    fn test_per_nvidia_historical() {
        let score = score_vs_median(45.80, 52.32, false);
        assert!((score - 84.9).abs() < 0.5);
    }

    #[test]
    fn test_at_median() {
        let score = score_vs_median(30.0, 30.0, false);
        assert!((score - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_twenty_percent_below_lower_is_better() {
        let score = score_vs_median(24.0, 30.0, false);
        assert!((score - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_double_median_lower_is_better() {
        let score = score_vs_median(60.0, 30.0, false);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_beyond_double_capped_at_zero() {
        let score = score_vs_median(100.0, 30.0, false);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_higher_is_better_above_median() {
        let score = score_vs_median(170.0, 10.0, true);
        assert!((score - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_higher_is_better_below_median() {
        let score = score_vs_median(5.0, 10.0, true);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_zero_median_returns_neutral() {
        let score = score_vs_median(45.0, 0.0, false);
        assert!((score - 60.0).abs() < f64::EPSILON);
    }

    // Revenue Growth brackets: [-5, 0, 5, 12] → floor=-10, ceiling=19
    fn revenue_brackets() -> Brackets {
        Brackets { thresholds: [-5.0, 0.0, 5.0, 12.0] }
    }

    #[test]
    fn test_brackets_at_thresholds() {
        let b = revenue_brackets();
        assert!((score_from_brackets(-10.0, &b) - 0.0).abs() < f64::EPSILON, "floor");
        assert!((score_from_brackets(-5.0, &b) - 20.0).abs() < f64::EPSILON, "t0");
        assert!((score_from_brackets(0.0, &b) - 40.0).abs() < f64::EPSILON, "t1");
        assert!((score_from_brackets(5.0, &b) - 60.0).abs() < f64::EPSILON, "t2");
        assert!((score_from_brackets(12.0, &b) - 80.0).abs() < f64::EPSILON, "t3");
        assert!((score_from_brackets(19.0, &b) - 100.0).abs() < f64::EPSILON, "ceiling");
    }

    #[test]
    fn test_brackets_interpolation() {
        let b = revenue_brackets();
        // Midpoint of 0-5% band → score 50
        let score = score_from_brackets(2.5, &b);
        assert!((score - 50.0).abs() < 0.1, "mid band: {score}");
    }

    #[test]
    fn test_brackets_below_floor() {
        let b = revenue_brackets();
        assert!((score_from_brackets(-20.0, &b) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_brackets_above_ceiling() {
        let b = revenue_brackets();
        assert!((score_from_brackets(70.5, &b) - 100.0).abs() < f64::EPSILON, "NVIDIA revenue");
    }

    // Normalized diff tests (growth sector/historical)
    #[test]
    fn test_normdiff_at_median() {
        let score = score_normalized_diff(10.0, 10.0);
        assert!((score - 60.0).abs() < 0.1, "at median: {score}");
    }

    #[test]
    fn test_normdiff_nvidia_vs_negative_sector() {
        // current=97.9, median=-1.5 → huge positive, score ~100
        let score = score_normalized_diff(97.9, -1.5);
        assert!((score - 100.0).abs() < 0.5, "NVIDIA vs neg sector: {score}");
    }

    #[test]
    fn test_normdiff_negative_worse_than_negative_median() {
        // current=-5, median=-1.5 → worse than median, score ~28
        let score = score_normalized_diff(-5.0, -1.5);
        assert!(score < 40.0, "neg worse than neg median: {score}");
    }

    #[test]
    fn test_normdiff_negative_vs_positive_median() {
        // current=-5, median=10 → very bad, score ~0
        let score = score_normalized_diff(-5.0, 10.0);
        assert!((score - 0.0).abs() < 0.5, "neg vs pos median: {score}");
    }

    #[test]
    fn test_normdiff_below_median_positive() {
        // current=5, median=10 → below but positive, score ~40
        let score = score_normalized_diff(5.0, 10.0);
        assert!((score - 40.0).abs() < 0.5, "below median: {score}");
    }

    #[test]
    fn test_normdiff_above_median() {
        // current=15, median=10 → above, score ~68
        let score = score_normalized_diff(15.0, 10.0);
        assert!((score - 68.0).abs() < 0.5, "above median: {score}");
    }

    #[test]
    fn test_normdiff_both_negative_worse() {
        // current=-20, median=-10 → worse, score ~40
        let score = score_normalized_diff(-20.0, -10.0);
        assert!((score - 40.0).abs() < 0.5, "both neg worse: {score}");
    }

    // Dynamic bracket weights tests
    #[test]
    fn test_weights_sum_to_one() {
        let b = revenue_brackets();
        let (a, s, h) = bracket_sub_weights(-1.5, 20.0, &b);
        assert!((a + s + h - 1.0).abs() < 1e-9, "sum: {}", a + s + h);
    }

    #[test]
    fn test_weights_floor_respected() {
        let b = revenue_brackets();
        let (a, s, h) = bracket_sub_weights(-1.5, 20.0, &b);
        assert!(a >= MIN_BRACKET_WEIGHT - 1e-9, "abs: {a}");
        assert!(s >= MIN_BRACKET_WEIGHT - 1e-9, "sector: {s}");
        assert!(h >= MIN_BRACKET_WEIGHT - 1e-9, "hist: {h}");
    }

    #[test]
    fn test_weights_high_median_gets_low_weight() {
        let b = revenue_brackets();
        // sector med at ceiling (bracket=100) vs hist med at floor (bracket=0)
        let (_, s_high, h_low) = bracket_sub_weights(19.0, -10.0, &b);
        // sector should have less weight than historical
        assert!(s_high < h_low, "high median sector ({s_high}) should weigh less than low median hist ({h_low})");
    }

    #[test]
    fn test_weights_both_medians_neutral() {
        let b = revenue_brackets();
        // Both medians at t1 (bracket=40) → sector and historical should be roughly equal
        let (_, s, h) = bracket_sub_weights(0.0, 0.0, &b);
        assert!((s - h).abs() < 1e-9, "equal medians should give equal weights: s={s}, h={h}");
    }

    #[test]
    fn test_weights_both_medians_perfect() {
        let b = revenue_brackets();
        // Both medians at ceiling → both get minimum weight, absolute gets most
        let (a, s, h) = bracket_sub_weights(19.0, 19.0, &b);
        assert!(a > s, "absolute ({a}) should dominate when both medians are perfect (s={s})");
        assert!((s - h).abs() < 1e-9, "s and h should be equal: s={s}, h={h}");
    }

    #[test]
    fn test_weights_both_medians_terrible() {
        let b = revenue_brackets();
        // Both medians at floor → both get maximum weight, absolute gets minimum
        let (a, s, h) = bracket_sub_weights(-10.0, -10.0, &b);
        assert!(a < s, "absolute ({a}) should be minimal when both medians are terrible (s={s})");
        assert!((s - h).abs() < 1e-9, "s and h should be equal: s={s}, h={h}");
    }
}
