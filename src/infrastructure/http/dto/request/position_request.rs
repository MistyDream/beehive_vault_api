use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PositionsQueryParams {
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
