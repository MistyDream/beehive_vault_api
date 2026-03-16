#[derive(Debug, Clone)]
pub struct Stock {
    pub id: i32,
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}

pub struct StockFilter {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub isin: Option<String>,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub page: i64,
    pub per_page: i64,
}

pub struct Paginated<T> {
    pub data: Vec<T>,
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
}

pub struct UpdateStock {
    pub symbol: String,
    pub name: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}

pub struct NewStock {
    pub symbol: String,
    pub name: String,
    pub isin: String,
    pub currency: Option<String>,
    pub market: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub country: Option<String>,
}
