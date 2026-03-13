use actix_web::web;

use crate::infrastructure::http::controllers::account_controller::get_account;
use crate::infrastructure::http::controllers::stock_controller::{create_stock, get_stock};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_account);
    cfg.service(create_stock);
    cfg.service(get_stock);
}