use actix_web::web;

use crate::infrastructure::http::controllers::account_controller::get_account;
use crate::infrastructure::http::controllers::portfolio_controller::{
    create_portfolio, delete_portfolio, get_portfolio, list_portfolios, update_portfolio,
};
use crate::infrastructure::http::controllers::performance_controller::get_performance;
use crate::infrastructure::http::controllers::scoring_controller::get_portfolio_scoring;
use crate::infrastructure::http::controllers::stock_price_controller::{
    get_stock_latest_price, get_stock_price_history,
};
use crate::infrastructure::http::controllers::position_controller::{
    get_cash_balance, get_portfolio_summary, get_positions,
};
use crate::infrastructure::http::controllers::transaction_controller::{
    create_transaction, delete_transaction, get_transaction, get_transactions_stats,
    list_transactions, update_transaction,
};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_account);
    cfg.service(create_portfolio);
    cfg.service(list_portfolios);
    cfg.service(get_portfolio);
    cfg.service(update_portfolio);
    cfg.service(delete_portfolio);
    cfg.service(create_transaction);
    cfg.service(list_transactions);
    cfg.service(get_transactions_stats);
    cfg.service(get_transaction);
    cfg.service(update_transaction);
    cfg.service(delete_transaction);
    cfg.service(get_positions);
    cfg.service(get_cash_balance);
    cfg.service(get_portfolio_summary);
    cfg.service(get_performance);
    cfg.service(get_portfolio_scoring);
    cfg.service(get_stock_latest_price);
    cfg.service(get_stock_price_history);
}