use serde::Serialize;

use crate::application::services::pagination::Page;

/// Build a RFC 8288 Link header for a paginated collection.
/// Returns `None` when there is a single page (no navigation needed).
pub fn build_link_header(path: &str, query: &str, page: u32, per_page: u32, total: u32) -> Option<String> {
    if per_page == 0 {
        return None;
    }
    let total_pages = total.div_ceil(per_page);
    if total_pages <= 1 {
        return None;
    }

    let base_params: Vec<String> = query
        .split('&')
        .filter(|p| !p.is_empty() && !p.starts_with("page="))
        .map(String::from)
        .collect();

    let make_url = |p: u32| -> String {
        let mut parts = base_params.clone();
        parts.push(format!("page={}", p));
        format!("{}?{}", path, parts.join("&"))
    };

    let mut links = vec![format!("<{}>; rel=\"first\"", make_url(1))];
    if page > 1 {
        links.push(format!("<{}>; rel=\"prev\"", make_url(page - 1)));
    }
    if page < total_pages {
        links.push(format!("<{}>; rel=\"next\"", make_url(page + 1)));
    }
    links.push(format!("<{}>; rel=\"last\"", make_url(total_pages)));
    Some(links.join(", "))
}

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
