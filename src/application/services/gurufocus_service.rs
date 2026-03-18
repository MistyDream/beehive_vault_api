use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::metric_value_repository::MetricValueRepository;
use crate::application::ports::score_repository::ScoreRepository;
use crate::domain::market::metric_value::NewMetricValue;
use crate::domain::scoring::{ExtractedMetric, compute_scoring};

pub struct GurufocusService {
    metric_repo: Arc<dyn MetricValueRepository>,
    score_repo: Arc<dyn ScoreRepository>,
}

impl GurufocusService {
    pub fn new(
        metric_repo: Arc<dyn MetricValueRepository>,
        score_repo: Arc<dyn ScoreRepository>,
    ) -> Self {
        Self { metric_repo, score_repo }
    }

    pub async fn import_and_score(
        &self,
        stock_id: i32,
        values: Vec<NewMetricValue>,
        rank_data: Vec<ExtractedMetric>,
    ) -> Result<usize, AppError> {
        let imported = if values.is_empty() {
            0
        } else {
            self.metric_repo.bulk_insert_ignore(values).await?
        };

        let scoring = compute_scoring(&rank_data);
        self.score_repo.save_scoring(stock_id, scoring).await?;

        Ok(imported)
    }
}
