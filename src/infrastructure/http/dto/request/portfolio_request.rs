use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreatePortfolioRequest {
    #[garde(length(min = 1, max = 120))]
    pub name: String,
    #[garde(pattern(r"^(real|virtual)$"))]
    pub kind: String,
    #[garde(length(min = 3, max = 3))]
    pub currency: Option<String>,
    #[garde(length(max = 500))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdatePortfolioRequest {
    #[garde(length(min = 1, max = 120))]
    pub name: String,
    #[garde(pattern(r"^(real|virtual)$"))]
    pub kind: String,
    #[garde(length(min = 3, max = 3))]
    pub currency: Option<String>,
    #[garde(length(max = 500))]
    pub description: Option<String>,
}
