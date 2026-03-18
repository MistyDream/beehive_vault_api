use std::future::Future;
use std::pin::Pin;

use crate::application::error::AppError;
use crate::domain::market::metric_value::NewMetricValue;

pub trait MetricValueRepository: Send + Sync {
    fn bulk_insert_ignore(
        &self,
        values: Vec<NewMetricValue>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>>;
}
