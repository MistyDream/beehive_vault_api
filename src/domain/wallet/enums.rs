use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioKind {
    Real,
    Virtual,
}

impl PortfolioKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PortfolioKind::Real => "real",
            PortfolioKind::Virtual => "virtual",
        }
    }
}

impl TryFrom<&str> for PortfolioKind {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "real" => Ok(PortfolioKind::Real),
            "virtual" => Ok(PortfolioKind::Virtual),
            other => Err(format!("unknown PortfolioKind: '{other}'")),
        }
    }
}
