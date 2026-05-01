use garde::Validate;
use serde::Deserialize;

use crate::domain::market::enums::MarketRegion;
use crate::domain::market::isin::Isin;
use crate::domain::market::stock::{NewStock, StockPatch};

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct StockSearchQuery {
    /// Optional: a missing `q` produces a 400 from the controller; a present
    /// but invalid `q` produces a 422 from this validator.
    #[garde(inner(length(chars, max = 50), custom(non_blank_min_2)))]
    pub q: Option<String>,
}

fn non_blank_min_2(value: &String, _: &()) -> garde::Result {
    if value.trim().chars().count() < 2 {
        return Err(garde::Error::new(
            "must contain at least 2 non-whitespace characters",
        ));
    }
    Ok(())
}

// Patterns inlined because `garde(pattern(...))` requires a literal. ISIN
// check digit is intentionally not verified — format-only check.
//
// The ISIN regex below MUST stay in sync with `Isin::try_new` (domain). The
// `From<…StockRequest>` impls re-parse the validated string via `Isin::try_new`
// and `expect()` infallibility — any drift between this regex and the domain
// validator will turn into a runtime panic.
#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct CreateStockRequest {
    #[garde(length(chars, min = 1, max = 20))]
    pub symbol: String,
    #[garde(length(chars, min = 1, max = 255))]
    pub name: String,
    #[garde(pattern(r"^[A-Z]{2}[A-Z0-9]{9}[0-9]$"))]
    pub isin: String,
    #[garde(pattern(r"^[A-Z]{3}$"))]
    pub currency: String,
    #[garde(skip)]
    pub market_region: MarketRegion,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub market: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub sector: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub industry: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{2}$")))]
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct UpdateStockRequest {
    #[garde(inner(length(chars, min = 1, max = 20)))]
    pub symbol: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 255)))]
    pub name: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{2}[A-Z0-9]{9}[0-9]$")))]
    pub isin: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{3}$")))]
    pub currency: Option<String>,
    #[garde(skip)]
    pub market_region: Option<MarketRegion>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub market: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub sector: Option<String>,
    #[garde(inner(length(chars, min = 1, max = 64)))]
    pub industry: Option<String>,
    #[garde(inner(pattern(r"^[A-Z]{2}$")))]
    pub country: Option<String>,
}

impl From<CreateStockRequest> for NewStock {
    fn from(req: CreateStockRequest) -> Self {
        // garde already validated the ISIN format via the same regex used by
        // `Isin::try_new`, so this construction cannot fail.
        let isin = Isin::try_new(&req.isin).expect("isin format pre-validated by garde");
        NewStock {
            symbol: req.symbol,
            name: req.name,
            isin,
            currency: req.currency,
            market_region: req.market_region,
            market: req.market,
            sector: req.sector,
            industry: req.industry,
            country: req.country,
        }
    }
}

impl From<UpdateStockRequest> for StockPatch {
    fn from(req: UpdateStockRequest) -> Self {
        let isin = req
            .isin
            .map(|s| Isin::try_new(&s).expect("isin format pre-validated by garde"));
        StockPatch {
            symbol: req.symbol,
            name: req.name,
            isin,
            currency: req.currency,
            market_region: req.market_region,
            market: req.market,
            sector: req.sector,
            industry: req.industry,
            country: req.country,
        }
    }
}
