use garde::Validate;
use serde::Deserialize;

use crate::domain::wallet::enums::PortfolioKind;
use crate::domain::wallet::portfolio::{NewPortfolio, UpdatePortfolio};

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

impl From<CreatePortfolioRequest> for NewPortfolio {
    fn from(req: CreatePortfolioRequest) -> Self {
        NewPortfolio {
            name: req.name,
            // Safe: garde validated the pattern "^(real|virtual)$"
            kind: PortfolioKind::try_from(req.kind.as_str()).unwrap(),
            currency: req.currency.unwrap_or_else(|| "EUR".to_owned()),
            description: req.description,
        }
    }
}

impl From<UpdatePortfolioRequest> for UpdatePortfolio {
    fn from(req: UpdatePortfolioRequest) -> Self {
        NewPortfolio {
            name: req.name,
            kind: PortfolioKind::try_from(req.kind.as_str()).unwrap(),
            currency: req.currency.unwrap_or_else(|| "EUR".to_owned()),
            description: req.description,
        }
    }
}
