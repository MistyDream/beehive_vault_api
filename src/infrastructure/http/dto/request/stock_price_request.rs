use chrono::NaiveDate;
use garde::Validate;
use serde::Deserialize;

/// `from` and `to` are intentionally optional at the DTO level so that a
/// missing parameter surfaces as the project's canonical `application/problem+json`
/// via `AppError::BadRequest` in the controller, instead of a framework-level
/// 400 that bypasses our error mapping.
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct PriceHistoryQuery {
    #[garde(skip)]
    pub from: Option<NaiveDate>,
    #[garde(skip)]
    pub to: Option<NaiveDate>,
}
