use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct StockSearchQuery {
    /// Optional so a missing parameter yields our problem+json 400, not garde's 400.
    #[garde(inner(length(min = 2, max = 50)))]
    pub q: Option<String>,
}
