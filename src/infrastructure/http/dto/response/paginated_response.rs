use serde::Serialize;

use crate::domain::market::stock::Paginated;

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

impl<T: Serialize, D: Into<T>> From<Paginated<D>> for PaginatedResponse<T> {
    fn from(p: Paginated<D>) -> Self {
        PaginatedResponse {
            data: p.data.into_iter().map(Into::into).collect(),
            page: p.page,
            per_page: p.per_page,
            total: p.total,
        }
    }
}
