use std::sync::Arc;

use crate::application::services::stock_service::StockService;

#[derive(Clone)]
pub struct AppState {
    pub stock_service: Arc<StockService>,
}
