use std::future::Future;
use std::pin::Pin;

use crate::application::error::AppError;

/// Probe port for orchestration readiness checks.
///
/// Implementations verify that critical external dependencies (primarily the
/// database connection pool) are reachable. Used by the `/readyz` endpoint.
pub trait HealthChecker: Send + Sync {
    /// Returns `Ok(())` if the service is ready to handle traffic. Any
    /// underlying failure is mapped to `AppError::Internal` so the HTTP
    /// layer can translate it to `503 Service Unavailable`.
    fn readiness(&self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>>;
}
