#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Internal server error")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}
