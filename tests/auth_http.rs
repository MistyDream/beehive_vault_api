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
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE, WWW_AUTHENTICATE};
use actix_web::{test, web, App};
use chrono::NaiveDate;
use serde_json::Value;

use beehive_vault_api::infrastructure::http::controllers::health_controller;
use beehive_vault_api::infrastructure::http::middleware::auth::BearerAuth;
use beehive_vault_api::infrastructure::http::routes::configure_routes;

use common::build_app_state;
use common::fakes::{
    test_price, test_stock, InMemoryStockPriceRepo, InMemoryStockRepo, NotReadyHealthChecker,
};

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

    let req = test::TestRequest::get().uri("/v1/stocks/US0000000001/price").to_request();
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
        .uri("/v1/stocks/US0000000001/price")
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
        .uri("/v1/stocks/US0000000001/price")
        .insert_header(("Authorization", format!("Bearer {TEST_KEY}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn v1_includes_www_authenticate_header_on_401() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get().uri("/v1/stocks/US0000000001/price").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let www_auth = resp
        .headers()
        .get(WWW_AUTHENTICATE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        www_auth.starts_with("Bearer"),
        "RFC 9110 §15.5.2 requires Bearer in WWW-Authenticate, got {www_auth:?}"
    );
}

#[actix_web::test]
async fn v1_accepts_extra_whitespace_between_scheme_and_token() {
    let app = make_service!(TEST_KEY);
    // RFC 7235 §2.1 allows 1*SP between scheme and credentials.
    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001/price")
        .insert_header(("Authorization", format!("Bearer   {TEST_KEY}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn v1_accepts_lowercase_bearer_scheme() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001/price")
        .insert_header(("Authorization", format!("bearer {TEST_KEY}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn v1_returns_401_for_same_length_wrong_token() {
    let app = make_service!(TEST_KEY);
    // Same length as TEST_KEY but every byte differs — exercises the
    // per-byte constant-time comparison, not just the length early-return.
    let wrong = "x".repeat(TEST_KEY.len());

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001/price")
        .insert_header(("Authorization", format!("Bearer {wrong}")))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn v1_returns_401_for_non_bearer_scheme() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001/price")
        .insert_header(("Authorization", "Basic dXNlcjpwYXNz"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn v1_returns_401_for_empty_bearer_token() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001/price")
        .insert_header(("Authorization", "Bearer "))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
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

#[actix_web::test]
async fn readyz_returns_503_when_health_checker_is_not_ready() {
    // Bypass the macro so we can swap the HealthChecker port for a failing one.
    let stock_repo = Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")]));
    let price_repo = Arc::new(InMemoryStockPriceRepo::with_prices(vec![test_price(
        1,
        date(2026, 4, 24),
        "170.25",
    )]));
    let mut state = build_app_state(stock_repo, price_repo);
    state.health_checker = Arc::new(NotReadyHealthChecker);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(health_controller::configure),
    )
    .await;

    let req = test::TestRequest::get().uri("/readyz").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[actix_web::test]
async fn healthz_sets_cache_control_no_store() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;

    let cc = resp
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert_eq!(cc, "no-store");
}

#[actix_web::test]
async fn readyz_sets_cache_control_no_store() {
    let app = make_service!(TEST_KEY);

    let req = test::TestRequest::get().uri("/readyz").to_request();
    let resp = test::call_service(&app, req).await;

    let cc = resp
        .headers()
        .get(CACHE_CONTROL)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert_eq!(cc, "no-store");
}
