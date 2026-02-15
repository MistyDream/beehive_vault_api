use actix_web::web;

use crate::infrastructure::http::controllers::account_controller::get_account;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_account);
}