use std::sync::Arc;

use crate::application::services::gurufocus_service::GurufocusService;
use crate::application::services::stock_service::StockService;

#[derive(Clone)]
pub struct AppState {
    pub stock_service: Arc<StockService>,
    pub gurufocus_service: Arc<GurufocusService>,
}
