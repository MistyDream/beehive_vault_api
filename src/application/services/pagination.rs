pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_LIMIT: u32 = 25;
pub const MAX_LIMIT: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

impl SortDirection {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("asc") => SortDirection::Asc,
            _ => SortDirection::Desc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

pub fn paginate_slice<T: Clone>(items: Vec<T>, page: u32, limit: u32) -> Page<T> {
    let total = items.len() as u32;
    let per_page = limit.clamp(1, MAX_LIMIT);
    let page = page.max(1);
    let start = ((page - 1) * per_page) as usize;
    let end = (start + per_page as usize).min(items.len());
    let page_items = if start >= items.len() {
        Vec::new()
    } else {
        items[start..end].to_vec()
    };
    Page {
        items: page_items,
        total,
        page,
        per_page,
    }
}
