use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketRegion {
    Americas,
    Europe,
    AsiaPacific,
    Other,
}

impl MarketRegion {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketRegion::Americas => "americas",
            MarketRegion::Europe => "europe",
            MarketRegion::AsiaPacific => "asia_pacific",
            MarketRegion::Other => "other",
        }
    }

    /// Infer the region from a Yahoo-formatted ticker suffix.
    /// No suffix → Americas (NYSE / NASDAQ). Unknown suffix → Other.
    pub fn from_symbol(symbol: &str) -> Self {
        match symbol.rsplit_once('.') {
            None => MarketRegion::Americas,
            Some((_, suffix)) => match suffix {
                "TO" | "V" => MarketRegion::Americas,
                "PA" | "AS" | "BR" | "DE" | "F" | "L" | "SW" | "MI" => MarketRegion::Europe,
                "T" | "HK" | "AX" | "SI" => MarketRegion::AsiaPacific,
                _ => MarketRegion::Other,
            },
        }
    }
}

impl TryFrom<&str> for MarketRegion {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "americas" => Ok(MarketRegion::Americas),
            "europe" => Ok(MarketRegion::Europe),
            "asia_pacific" => Ok(MarketRegion::AsiaPacific),
            "other" => Ok(MarketRegion::Other),
            other => Err(format!("unknown MarketRegion: '{other}'")),
        }
    }
}
