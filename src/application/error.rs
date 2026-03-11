#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}
