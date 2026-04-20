use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(()))]
pub struct PositionsQueryParams {
    #[garde(inner(pattern(r"^(symbol|quantity|average_cost|total_cost|weight)$")))]
    pub sort_by: Option<String>,
    #[garde(inner(pattern(r"^(asc|desc)$")))]
    pub sort_dir: Option<String>,
    #[garde(inner(range(min = 1)))]
    pub page: Option<u32>,
    #[garde(inner(range(min = 1, max = 100)))]
    pub limit: Option<u32>,
}
