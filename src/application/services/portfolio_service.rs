use std::sync::Arc;

use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::ports::portfolio_repository::PortfolioRepository;
use crate::domain::wallet::portfolio::{NewPortfolio, Portfolio, UpdatePortfolio};

pub struct PortfolioService {
    repo: Arc<dyn PortfolioRepository>,
}

impl PortfolioService {
    pub fn new(repo: Arc<dyn PortfolioRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, new: NewPortfolio) -> Result<Portfolio, AppError> {
        self.repo.insert(new).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Portfolio, AppError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list(&self) -> Result<Vec<Portfolio>, AppError> {
        self.repo.list_all().await
    }

    pub async fn update(&self, id: Uuid, data: UpdatePortfolio) -> Result<Portfolio, AppError> {
        self.repo.update(id, data).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        let deleted = self.repo.delete(id).await?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }
}
