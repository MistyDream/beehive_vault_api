use crate::application::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("DB pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    #[error("Diesel error: {0}")]
    Diesel(#[from] diesel::result::Error),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Conversion error: {0}")]
    Conversion(String),
}

impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        match &err {
            DbError::Diesel(diesel::result::Error::NotFound) => AppError::NotFound,
            _ => AppError::Internal(Box::new(err)),
        }
    }
}