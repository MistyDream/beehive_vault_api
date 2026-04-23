use std::future::Future;
use std::pin::Pin;

use crate::application::error::AppError;
use crate::domain::scoring::score_snapshot::ScoreSnapshot;

pub trait ScoreSnapshotRepository: Send + Sync {
    fn find_by_id(&self, snapshot_id: i32) -> Pin<Box<dyn Future<Output = Result<ScoreSnapshot, AppError>> + Send + '_>>;

    fn find_by_stock(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<Vec<ScoreSnapshot>, AppError>> + Send + '_>>;

    fn find_latest_by_stock(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<ScoreSnapshot, AppError>> + Send + '_>>;

    fn delete(&self, snapshot_id: i32) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>>;

    fn delete_by_stock(&self, stock_id: i32) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>>;
}
