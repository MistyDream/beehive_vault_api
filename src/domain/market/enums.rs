use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    ///
    /// Important: `Other` has no cron entry in the price scheduler, so any
    /// stock that falls through the match below will silently never refresh.
    /// Prefer extending this match over shipping stocks that land in `Other`.
    pub fn from_symbol(symbol: &str) -> Self {
        match symbol.rsplit_once('.') {
            None => MarketRegion::Americas,
            Some((_, suffix)) => match suffix {
                "TO" | "V" => MarketRegion::Americas,
                "PA" | "AS" | "BR" | "DE" | "F" | "L" | "SW" | "MI" => MarketRegion::Europe,
                "T" | "HK" | "AX" | "SI" | "NS" | "BO" | "KS" | "KQ" | "SS" | "SZ" | "NZ" => {
                    MarketRegion::AsiaPacific
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_suffix_defaults_to_americas() {
        // NYSE / NASDAQ tickers carry no suffix.
        assert_eq!(MarketRegion::from_symbol("AAPL"), MarketRegion::Americas);
        assert_eq!(MarketRegion::from_symbol("MSFT"), MarketRegion::Americas);
    }

    #[test]
    fn toronto_and_venture_go_to_americas() {
        assert_eq!(MarketRegion::from_symbol("DRX.TO"), MarketRegion::Americas);
        assert_eq!(MarketRegion::from_symbol("ABC.V"), MarketRegion::Americas);
    }

    #[test]
    fn european_exchange_suffixes() {
        assert_eq!(MarketRegion::from_symbol("OR.PA"), MarketRegion::Europe);
        assert_eq!(MarketRegion::from_symbol("SAP.DE"), MarketRegion::Europe);
        assert_eq!(MarketRegion::from_symbol("HSBA.L"), MarketRegion::Europe);
        assert_eq!(MarketRegion::from_symbol("NESN.SW"), MarketRegion::Europe);
    }

    #[test]
    fn asia_pacific_exchange_suffixes() {
        assert_eq!(MarketRegion::from_symbol("7203.T"), MarketRegion::AsiaPacific);
        assert_eq!(MarketRegion::from_symbol("0700.HK"), MarketRegion::AsiaPacific);
        assert_eq!(MarketRegion::from_symbol("BHP.AX"), MarketRegion::AsiaPacific);
        assert_eq!(MarketRegion::from_symbol("Z74.SI"), MarketRegion::AsiaPacific);
        // India — NSE + BSE
        assert_eq!(MarketRegion::from_symbol("RELIANCE.NS"), MarketRegion::AsiaPacific);
        assert_eq!(MarketRegion::from_symbol("TCS.BO"), MarketRegion::AsiaPacific);
        // Korea — KRX + KOSDAQ
        assert_eq!(MarketRegion::from_symbol("005930.KS"), MarketRegion::AsiaPacific);
        assert_eq!(MarketRegion::from_symbol("091990.KQ"), MarketRegion::AsiaPacific);
        // Greater China — Shanghai + Shenzhen
        assert_eq!(MarketRegion::from_symbol("600519.SS"), MarketRegion::AsiaPacific);
        assert_eq!(MarketRegion::from_symbol("000001.SZ"), MarketRegion::AsiaPacific);
        // New Zealand
        assert_eq!(MarketRegion::from_symbol("AIR.NZ"), MarketRegion::AsiaPacific);
    }

    #[test]
    fn unknown_suffix_falls_back_to_other() {
        assert_eq!(MarketRegion::from_symbol("XYZ.ZZ"), MarketRegion::Other);
        assert_eq!(MarketRegion::from_symbol("FOO.BAR"), MarketRegion::Other);
    }

    #[test]
    fn as_str_try_from_roundtrip() {
        for region in [
            MarketRegion::Americas,
            MarketRegion::Europe,
            MarketRegion::AsiaPacific,
            MarketRegion::Other,
        ] {
            assert_eq!(MarketRegion::try_from(region.as_str()), Ok(region));
        }
    }

    #[test]
    fn try_from_rejects_unknown() {
        assert!(MarketRegion::try_from("mars").is_err());
    }
}
