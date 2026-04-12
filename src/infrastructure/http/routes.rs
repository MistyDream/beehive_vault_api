use actix_web::web;

use crate::infrastructure::http::controllers::account_controller::get_account;
use crate::infrastructure::http::controllers::portfolio_controller::{
    create_portfolio, delete_portfolio, get_portfolio, list_portfolios, update_portfolio,
};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_account);
    cfg.service(create_portfolio);
    cfg.service(list_portfolios);
    cfg.service(get_portfolio);
    cfg.service(update_portfolio);
    cfg.service(delete_portfolio);
}