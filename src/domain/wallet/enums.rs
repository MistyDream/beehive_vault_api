use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Buy,
    Sell,
    Dividend,
    Fee,
    Split,
    Deposit,
    Withdrawal,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Buy => "buy",
            TransactionType::Sell => "sell",
            TransactionType::Dividend => "dividend",
            TransactionType::Fee => "fee",
            TransactionType::Split => "split",
            TransactionType::Deposit => "deposit",
            TransactionType::Withdrawal => "withdrawal",
        }
    }
}

impl TryFrom<&str> for TransactionType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "buy" => Ok(TransactionType::Buy),
            "sell" => Ok(TransactionType::Sell),
            "dividend" => Ok(TransactionType::Dividend),
            "fee" => Ok(TransactionType::Fee),
            "split" => Ok(TransactionType::Split),
            "deposit" => Ok(TransactionType::Deposit),
            "withdrawal" => Ok(TransactionType::Withdrawal),
            other => Err(format!("unknown TransactionType: '{other}'")),
        }
    }
}

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
