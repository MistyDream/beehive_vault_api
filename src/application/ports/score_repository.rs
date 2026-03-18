use std::future::Future;
use std::pin::Pin;

use crate::application::error::AppError;
use crate::domain::scoring::ScoringResult;

pub trait ScoreRepository: Send + Sync {
    fn save_scoring(
        &self,
        stock_id: i32,
        result: ScoringResult,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>>;
}
