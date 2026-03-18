/// Input data needed to score a single metric.
pub struct MetricRankData {
    pub current: f64,
    pub med: f64,
    pub industry_med: f64,
}

/// Result of scoring a single metric: two sub-scores.
pub struct MetricScore {
    pub sector_score: f64,
    pub historical_score: f64,
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
}
