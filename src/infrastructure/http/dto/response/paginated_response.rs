use serde::Serialize;

use crate::application::services::pagination::Page;

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

impl<T> From<Page<T>> for PaginatedResponse<T> {
    fn from(page: Page<T>) -> Self {
        PaginatedResponse {
            items: page.items,
            total: page.total,
            page: page.page,
            per_page: page.per_page,
        }
    }
}

impl<T> PaginatedResponse<T> {
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> PaginatedResponse<U> {
        PaginatedResponse {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            page: self.page,
            per_page: self.per_page,
        }
    }
}
