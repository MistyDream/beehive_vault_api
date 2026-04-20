use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: u32,
}

pub fn paginate_slice<T: Clone>(items: Vec<T>, page: u32, limit: u32) -> Paginated<T> {
    let total = items.len() as u32;
    let page = page.max(1);
    let limit = limit.max(1);
    let start = ((page - 1) * limit) as usize;
    let end = (start + limit as usize).min(items.len());
    let page_items = if start >= items.len() {
        Vec::new()
    } else {
        items[start..end].to_vec()
    };
    Paginated { items: page_items, total }
}
