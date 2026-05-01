use std::future::Future;
use std::pin::Pin;

use uuid::Uuid;

use crate::application::error::AppError;
use crate::domain::wallet::portfolio::{NewPortfolio, Portfolio, UpdatePortfolio};

pub trait PortfolioRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>>;

    fn list_all(&self) -> Pin<Box<dyn Future<Output = Result<Vec<Portfolio>, AppError>> + Send + '_>>;

    fn insert(&self, new: NewPortfolio) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>>;

    fn update(&self, id: Uuid, data: UpdatePortfolio) -> Pin<Box<dyn Future<Output = Result<Portfolio, AppError>> + Send + '_>>;

    fn delete(&self, id: Uuid) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>>;
}
