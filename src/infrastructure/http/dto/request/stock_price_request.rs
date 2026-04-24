use chrono::NaiveDate;
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct PriceHistoryQuery {
    #[garde(skip)]
    pub from: NaiveDate,
    #[garde(skip)]
    pub to: NaiveDate,
}
