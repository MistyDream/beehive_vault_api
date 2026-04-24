//! HTTP integration tests for the bearer-auth middleware and the
//! `/healthz` / `/readyz` health endpoints.
//!
//! Exercises the Actix pipeline end-to-end (routes scoped under `/v1` are
//! wrapped with `BearerAuth`, healthchecks live outside the scope and are
//! therefore implicitly exempt). In-memory fakes keep the harness
//! hermetic — no DB, no network.

mod common;

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::http::header::CONTENT_TYPE;
use actix_web::{test, web, App};
use chrono::NaiveDate;
use serde_json::Value;

use beehive_vault_api::infrastructure::http::controllers::health_controller;
use beehive_vault_api::infrastructure::http::middleware::auth::BearerAuth;
use beehive_vault_api::infrastructure::http::routes::configure_routes;

use common::build_app_state;
use common::fakes::{test_price, test_stock, InMemoryStockPriceRepo, InMemoryStockRepo};

const PROBLEM_JSON: &str = "application/problem+json";
const TEST_KEY: &str = "bhv_test_0123456789abcdef0123456789abcdef0123456789abcdef0123456789ab";

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Mirror of the production pipeline: healthchecks at root, `/v1` wrapped
/// with `BearerAuth`. Takes `$key` so individual tests can assert the
/// middleware's view of "expected" vs "presented".
macro_rules! make_service {
    ($key:expr) => {{
        let stock_repo = Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")]));
        let price_repo = Arc::new(InMemoryStockPriceRepo::with_prices(vec![test_price(
            1,
            date(2026, 4, 24),
            "170.25",
        )]));
        let state = build_app_state(stock_repo, price_repo);
        test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(health_controller::configure)
                .service(
                    web::scope("/v1")
                        .wrap(BearerAuth::new($key.to_string()))
                        .configure(configure_routes),
                ),
        )
        .await
    }};
}

// =============================== /v1 auth ====================================

#[actix_web::test]
async fn v1_returns_401_when_authorization_header_is_missing() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get().uri("/v1/stocks/1/price").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        content_type.starts_with(PROBLEM_JSON),
        "expected application/problem+json, got {content_type:?}"
    );

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], 401);
    assert_eq!(body["type"], "/problems/unauthorized");
    assert_eq!(body["title"], "Unauthorized");
    assert!(
        body["detail"].as_str().is_some(),
        "detail must be present so clients can surface the reason"
    );
}

#[actix_web::test]
async fn v1_returns_401_when_bearer_token_does_not_match() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/price")
        .insert_header(("Authorization", "Bearer bhv_test_wrong_key"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["type"], "/problems/unauthorized");
}

#[actix_web::test]
async fn v1_returns_200_when_bearer_token_matches() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/price")
        .insert_header(("Authorization", format!("Bearer {TEST_KEY}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

// =============================== Healthchecks ================================

#[actix_web::test]
async fn healthz_returns_200_without_authorization() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn readyz_returns_200_without_authorization_when_health_checker_is_ready() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get().uri("/readyz").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}
