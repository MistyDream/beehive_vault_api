use actix_web::web;

use crate::infrastructure::http::controllers::account_controller::get_account;
use crate::infrastructure::http::controllers::stock_controller::{
    create_stock, delete_stock, get_stock, list_stocks, update_stock,
};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_account);
    cfg.service(list_stocks);
    cfg.service(create_stock);
    cfg.service(update_stock);
    cfg.service(delete_stock);
    cfg.service(get_stock);
}