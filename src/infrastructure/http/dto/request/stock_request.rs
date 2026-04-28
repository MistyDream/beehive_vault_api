use garde::Validate;
use serde::Deserialize;

/// `q` is intentionally optional at the DTO level so that a missing parameter
/// surfaces as the project's canonical `application/problem+json` via
/// `AppError::BadRequest` in the controller, instead of a framework-level 400
/// that bypasses our error mapping.
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct StockSearchQuery {
    #[garde(inner(length(min = 2, max = 50)))]
    pub q: Option<String>,
}
