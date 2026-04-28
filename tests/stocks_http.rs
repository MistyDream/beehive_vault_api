//! HTTP integration tests for the stock search endpoint.

mod common;

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use actix_web::test;
use serde_json::Value;

use common::fakes::{InMemoryStockPriceRepo, InMemoryStockRepo, test_stock};

const PROBLEM_JSON: &str = "application/problem+json";
const X_RESULT_TRUNCATED: &str = "x-result-truncated";

/// Tests never reach the price repository — wire a default fake.
fn empty_price_repo() -> Arc<InMemoryStockPriceRepo> {
    Arc::new(InMemoryStockPriceRepo::default())
}

// ============================ GET /v1/stocks?q= ==============================

#[actix_web::test]
async fn search_returns_200_with_matches_and_cache_headers() {
    let stocks = vec![
        test_stock(1, "AAPL", "USD"),
        test_stock(2, "MSFT", "USD"),
        test_stock(3, "GOOGL", "USD"),
    ];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=AAPL")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key(ETAG), "missing ETag header");
    assert!(
        resp.headers().contains_key(CACHE_CONTROL),
        "missing Cache-Control header"
    );
    assert!(
        !resp.headers().contains_key(X_RESULT_TRUNCATED),
        "X-Result-Truncated should be absent when below cap"
    );

    let body: Value = test::read_body_json(resp).await;
    let arr = body.as_array().expect("response body must be a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], 1);
    assert_eq!(arr[0]["symbol"], "AAPL");
    assert_eq!(arr[0]["name"], "Stock 1");
    assert_eq!(arr[0]["currency"], "USD");
    // Slim DTO must NOT include the dropped fields.
    assert!(arr[0].get("isin").is_none());
    assert!(arr[0].get("market").is_none());
    assert!(arr[0].get("sector").is_none());
    assert!(arr[0].get("market_region").is_none());
}

#[actix_web::test]
async fn search_returns_empty_array_when_no_match() {
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")])),
        empty_price_repo(),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=ZZZ")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body, serde_json::json!([]));
}

#[actix_web::test]
async fn search_returns_400_when_q_is_missing() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::get().uri("/v1/stocks").to_request();
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
        body["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("'q' is required")
    );
}

#[actix_web::test]
async fn search_returns_422_when_q_is_too_short() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::get().uri("/v1/stocks?q=a").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], 422);
    assert!(
        body["errors"].is_array(),
        "validation response must include field-level errors"
    );
}

#[actix_web::test]
async fn search_returns_422_when_q_is_too_long() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let long_q = "a".repeat(51);
    let uri = format!("/v1/stocks?q={long_q}");
    let req = test::TestRequest::get().uri(&uri).to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn search_returns_400_when_q_is_whitespace_only() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    // %20%20 = two spaces — passes garde length(min=2) but trim leaves 0 chars.
    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=%20%20")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("non-whitespace characters")
    );
}

#[actix_web::test]
async fn search_is_case_insensitive() {
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")])),
        empty_price_repo(),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=aapl")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["symbol"], "AAPL");
}

#[actix_web::test]
async fn search_matches_on_name() {
    // `test_stock` builds name = "Stock {id}".
    let stocks = vec![test_stock(1, "AAPL", "USD"), test_stock(2, "MSFT", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=Stock")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn search_matches_on_isin() {
    // `test_stock` builds isin = "ISIN{id:04}".
    let stocks = vec![test_stock(42, "AAPL", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=ISIN0042")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], 42);
}

#[actix_web::test]
async fn search_returns_304_when_if_none_match_matches() {
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")])),
        empty_price_repo(),
    );

    // First request: capture the ETag.
    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=AAPL")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let etag = resp
        .headers()
        .get(ETAG)
        .expect("first response missing ETag")
        .to_str()
        .unwrap()
        .to_string();

    // Conditional GET with the captured ETag → 304.
    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=AAPL")
        .insert_header((IF_NONE_MATCH, etag.as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
}

#[actix_web::test]
async fn search_sets_x_result_truncated_when_cap_reached() {
    // 51 stocks all matching `?q=Stock` (their name is "Stock {id}") so the
    // fake's 50-row cap bites and the controller surfaces the truncation flag.
    let stocks: Vec<_> = (1..=51)
        .map(|i| test_stock(i, &format!("S{i}"), "USD"))
        .collect();
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=Stock")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(X_RESULT_TRUNCATED)
            .and_then(|v| v.to_str().ok()),
        Some("true"),
        "X-Result-Truncated should be 'true' when cap is reached"
    );
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 50);
}

#[actix_web::test]
async fn search_omits_x_result_truncated_when_below_cap() {
    let stocks: Vec<_> = (1..=5)
        .map(|i| test_stock(i, &format!("S{i}"), "USD"))
        .collect();
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=Stock")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        !resp.headers().contains_key(X_RESULT_TRUNCATED),
        "X-Result-Truncated should be absent when below cap"
    );
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 5);
}
