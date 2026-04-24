//! HTTP integration tests for the stock price endpoints.
//!
//! Exercises the Actix pipeline end-to-end with in-memory fakes so the test
//! harness stays hermetic (no DB, no network).

mod common;

use std::sync::Arc;

use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use actix_web::http::StatusCode;
use actix_web::{test, web, App};
use chrono::NaiveDate;
use serde_json::Value;

const PROBLEM_JSON: &str = "application/problem+json";

use beehive_vault_api::infrastructure::http::routes::configure_routes;

use common::fakes::{test_price, test_stock, InMemoryStockPriceRepo, InMemoryStockRepo};
use common::build_app_state;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Macro so each test gets its own inferred Service type (avoids having to
/// spell out the full actix `Service` signature in a helper fn).
macro_rules! make_service {
    ($stock_repo:expr, $price_repo:expr $(,)?) => {{
        let state = build_app_state($stock_repo, $price_repo);
        test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(web::scope("/v1").configure(configure_routes)),
        )
        .await
    }};
}

// ======================= GET /v1/stocks/{stock_id}/price =====================

#[actix_web::test]
async fn latest_price_returns_200_with_cache_headers_and_currency_from_stock() {
    let stock = test_stock(1, "AAPL", "USD");
    let price = test_price(1, date(2026, 4, 24), "170.25");

    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::with_prices(vec![price])),
    );

    let req = test::TestRequest::get().uri("/v1/stocks/1/price").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key(ETAG), "missing ETag header");
    assert!(resp.headers().contains_key(CACHE_CONTROL), "missing Cache-Control");

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["price_date"], "2026-04-24");
    // Decimal is serialised as string (rust_decimal/serde-str).
    assert_eq!(body["close"], "170.25");
    assert_eq!(body["currency"], "USD");
    assert_eq!(body["source"], "yahoo");
}

#[actix_web::test]
async fn latest_price_returns_404_when_stock_is_unknown() {
    let app = make_service!(
        Arc::new(InMemoryStockRepo::default()),
        Arc::new(InMemoryStockPriceRepo::new()),
    );

    let req = test::TestRequest::get().uri("/v1/stocks/999/price").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // RFC 9457 Problem Details: error body must be application/problem+json
    // with the documented shape.
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
    assert_eq!(body["status"], 404);
    assert_eq!(body["title"], "Not Found");
    assert!(body["detail"].is_string());
    assert!(body["type"].is_string());
}

#[actix_web::test]
async fn latest_price_returns_404_when_stock_exists_but_has_no_price() {
    let stock = test_stock(1, "AAPL", "USD");
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::new()),
    );

    let req = test::TestRequest::get().uri("/v1/stocks/1/price").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn latest_price_returns_304_when_if_none_match_matches() {
    let stock = test_stock(1, "AAPL", "USD");
    let price = test_price(1, date(2026, 4, 24), "170.25");
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::with_prices(vec![price])),
    );

    // First request to capture the ETag + Cache-Control.
    let req = test::TestRequest::get().uri("/v1/stocks/1/price").to_request();
    let resp = test::call_service(&app, req).await;
    let etag = resp
        .headers()
        .get(ETAG)
        .expect("ETag on first response")
        .to_str()
        .unwrap()
        .to_string();
    let cache_control = resp
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control on first response")
        .to_str()
        .unwrap()
        .to_string();

    // Replay with If-None-Match → 304 Not Modified.
    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/price")
        .insert_header((IF_NONE_MATCH, etag.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);

    // RFC 9110 §15.4.5: 304 MUST preserve ETag + Cache-Control and MUST NOT
    // carry a body.
    assert_eq!(
        resp.headers().get(ETAG).and_then(|v| v.to_str().ok()),
        Some(etag.as_str()),
        "304 must echo back the same ETag"
    );
    assert_eq!(
        resp.headers().get(CACHE_CONTROL).and_then(|v| v.to_str().ok()),
        Some(cache_control.as_str()),
        "304 must preserve Cache-Control"
    );
    let body = test::read_body(resp).await;
    assert!(body.is_empty(), "304 must not carry a body");
}

#[actix_web::test]
async fn latest_price_returns_304_when_if_none_match_is_wildcard() {
    // RFC 9110 §13.1.2: `If-None-Match: *` must match any current representation.
    let stock = test_stock(1, "AAPL", "USD");
    let price = test_price(1, date(2026, 4, 24), "170.25");
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::with_prices(vec![price])),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/price")
        .insert_header((IF_NONE_MATCH, "*"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);

    // Same 304 invariants as the matching-ETag case.
    assert!(resp.headers().contains_key(ETAG), "304 must carry an ETag header");
    assert!(
        resp.headers().contains_key(CACHE_CONTROL),
        "304 must preserve Cache-Control"
    );
    let body = test::read_body(resp).await;
    assert!(body.is_empty(), "304 must not carry a body");
}

// ======================== GET /v1/stocks/{id}/prices =========================

#[actix_web::test]
async fn price_history_returns_prices_in_range_with_envelope_currency() {
    let stock = test_stock(1, "AAPL", "USD");
    let prices = vec![
        test_price(1, date(2026, 4, 22), "168.00"),
        test_price(1, date(2026, 4, 23), "169.00"),
        test_price(1, date(2026, 4, 24), "170.25"),
    ];
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::with_prices(prices)),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/prices?from=2026-04-23&to=2026-04-24")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["currency"], "USD");
    let prices = body["prices"].as_array().expect("prices array");
    assert_eq!(prices.len(), 2, "expected only the two dates inside the range");
    assert_eq!(prices[0]["price_date"], "2026-04-23");
    assert_eq!(prices[0]["close"], "169.00");
    assert_eq!(prices[1]["price_date"], "2026-04-24");
    assert_eq!(prices[1]["close"], "170.25");
}

#[actix_web::test]
async fn price_history_returns_empty_list_for_stock_with_no_prices() {
    let stock = test_stock(1, "AAPL", "USD");
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::new()),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/prices?from=2026-04-01&to=2026-04-24")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["currency"], "USD");
    assert_eq!(body["prices"].as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn price_history_returns_400_for_inverted_range() {
    let stock = test_stock(1, "AAPL", "USD");
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::new()),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/prices?from=2026-04-24&to=2026-04-01")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

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
    assert_eq!(body["status"], 400);
    assert_eq!(body["title"], "Bad Request");
    // `detail` must surface the human-readable reason produced by the service.
    assert!(
        body["detail"].as_str().unwrap_or_default().contains("on or before"),
        "expected detail to mention the range constraint, got {:?}",
        body["detail"]
    );
}

#[actix_web::test]
async fn price_history_returns_400_when_from_is_missing() {
    // Missing `from` must go through the canonical AppError::BadRequest path
    // so the response is application/problem+json, not a framework-level 400.
    let stock = test_stock(1, "AAPL", "USD");
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        Arc::new(InMemoryStockPriceRepo::new()),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/1/prices?to=2026-04-24")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

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
    assert_eq!(body["status"], 400);
    assert!(
        body["detail"].as_str().unwrap_or_default().contains("'from'"),
        "expected detail to mention the missing 'from' parameter, got {:?}",
        body["detail"]
    );
}

#[actix_web::test]
async fn price_history_returns_404_for_unknown_stock() {
    let app = make_service!(
        Arc::new(InMemoryStockRepo::default()),
        Arc::new(InMemoryStockPriceRepo::new()),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/999/prices?from=2026-04-01&to=2026-04-24")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
