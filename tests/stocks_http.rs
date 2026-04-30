//! HTTP integration tests for the stock CRUD + search endpoints.

mod common;

use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH, LOCATION};
use actix_web::test;
use beehive_vault_api::domain::market::enums::MarketRegion;
use beehive_vault_api::domain::market::isin::Isin;
use beehive_vault_api::domain::market::stock::Stock;
use serde_json::{Value, json};

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
async fn search_returns_422_when_q_is_only_whitespace() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    // %20%20 = two spaces — fails the trim-aware non_blank_min_2 validator.
    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=%20%20")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], 422);
    assert!(body["errors"].is_array());
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
    // `test_stock` builds an ISO 6166 ISIN of the form "US{id:010}".
    let stocks = vec![test_stock(42, "AAPL", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=US0000000042")
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
async fn search_handles_url_encoded_like_wildcards_safely() {
    // %25%25 decodes to `%%`. The prod adapter's escape neutralises both
    // wildcards so the SQL pattern stays narrow; the in-memory fake just does
    // a literal substring match. Either way the response must be a clean 200
    // with an empty array — never an unbounded match or a server error.
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")])),
        empty_price_repo(),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks?q=%25%25")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body, serde_json::json!([]));
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

// ============================ GET /v1/stocks/{isin} ==========================

#[actix_web::test]
async fn get_stock_returns_200_with_full_detail() {
    let stocks = vec![test_stock(42, "AAPL", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000042")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key(ETAG));
    assert!(resp.headers().contains_key(CACHE_CONTROL));
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], 42);
    assert_eq!(body["symbol"], "AAPL");
    assert_eq!(body["isin"], "US0000000042");
    assert_eq!(body["currency"], "USD");
    assert_eq!(body["market_region"], "europe");
    // Detail DTO must surface every field — slim DTO drops these.
    assert!(body.get("market").is_some());
    assert!(body.get("sector").is_some());
    assert!(body.get("industry").is_some());
    assert!(body.get("country").is_some());
}

#[actix_web::test]
async fn get_stock_returns_404_for_unknown_isin() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    // Format-valid ISIN that doesn't exist in the repo → 404 (not 400).
    let req = test::TestRequest::get()
        .uri("/v1/stocks/US9999999999")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn get_stock_returns_400_when_isin_format_invalid() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::get().uri("/v1/stocks/not-an-isin").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["detail"].as_str().unwrap_or_default().contains("isin"));
}

#[actix_web::test]
async fn get_stock_returns_304_when_if_none_match_matches() {
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![test_stock(1, "AAPL", "USD")])),
        empty_price_repo(),
    );

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let etag = resp.headers().get(ETAG).unwrap().to_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000001")
        .insert_header((IF_NONE_MATCH, etag.as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_MODIFIED);
}

// ============================ POST /v1/stocks ===============================

fn valid_create_body() -> Value {
    json!({
        "symbol": "TSLA",
        "name": "Tesla, Inc.",
        "isin": "US88160R1014",
        "currency": "USD",
        "market_region": "americas",
        "market": "NASDAQ",
        "sector": "Consumer Cyclical",
        "industry": "Auto Manufacturers",
        "country": "US"
    })
}

#[actix_web::test]
async fn create_stock_returns_201_with_location_and_full_body() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(valid_create_body())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);
    let location = resp
        .headers()
        .get(LOCATION)
        .expect("missing Location header")
        .to_str()
        .unwrap()
        .to_string();
    let body: Value = test::read_body_json(resp).await;
    let isin = body["isin"].as_str().expect("missing isin in create response");
    assert_eq!(location, format!("/v1/stocks/{isin}"));
    assert_eq!(body["symbol"], "TSLA");
    assert_eq!(body["isin"], "US88160R1014");
    assert_eq!(body["market_region"], "americas");

    // Round-trip: fetching the Location must return the same stock.
    let req = test::TestRequest::get().uri(&location).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let fetched: Value = test::read_body_json(resp).await;
    assert_eq!(fetched["symbol"], "TSLA");
}

#[actix_web::test]
async fn create_stock_returns_422_when_isin_format_invalid() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let mut body = valid_create_body();
    body["isin"] = json!("not-an-isin");
    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], 422);
    assert!(body["errors"].is_array());
}

#[actix_web::test]
async fn create_stock_returns_422_when_currency_format_invalid() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let mut body = valid_create_body();
    body["currency"] = json!("usd"); // lowercase fails ISO 4217 pattern
    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn create_stock_returns_409_when_symbol_already_exists() {
    let stocks = vec![test_stock(1, "TSLA", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(valid_create_body())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("symbol")
    );
}

#[actix_web::test]
async fn create_stock_returns_415_when_content_type_is_not_json() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .insert_header((CONTENT_TYPE, "text/plain"))
        .set_payload(serde_json::to_string(&valid_create_body()).unwrap())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[actix_web::test]
async fn create_stock_returns_422_when_symbol_is_too_long() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let mut body = valid_create_body();
    body["symbol"] = json!("A".repeat(21)); // garde max=20
    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn create_stock_returns_422_when_optional_field_is_empty_string() {
    // Inner gardes on `sector` enforce min=1 — explicit "" must fail validation
    // rather than slip through and produce a meaningless empty-string in the DB.
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let mut body = valid_create_body();
    body["sector"] = json!("");
    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn create_stock_returns_409_when_isin_already_exists() {
    // Existing stock has the same ISIN our payload uses: "ISIN{id:04}" with id=88160 →
    // doesn't match "US88160R1014", so seed one that does.
    let mut existing = test_stock(1, "OTHER", "USD");
    existing.isin = Isin::try_new("US88160R1014").unwrap();
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![existing])),
        empty_price_repo(),
    );

    let req = test::TestRequest::post()
        .uri("/v1/stocks")
        .set_json(valid_create_body())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body: Value = test::read_body_json(resp).await;
    assert!(body["detail"].as_str().unwrap_or_default().contains("isin"));
}

// ============================ PATCH /v1/stocks/{isin} ========================

#[actix_web::test]
async fn patch_stock_applies_partial_update_and_leaves_other_fields_untouched() {
    // Seed a fully-populated stock so the assertion can prove every untouched
    // field is preserved across the merge — not just `symbol`.
    let stock = Stock {
        id: 7,
        symbol: "AAPL".to_string(),
        name: "Apple Inc.".to_string(),
        isin: Isin::try_new("US0378331005").unwrap(),
        currency: "USD".to_string(),
        market_region: MarketRegion::Americas,
        market: Some("NASDAQ".to_string()),
        sector: Some("Technology".to_string()),
        industry: Some("Consumer Electronics".to_string()),
        country: Some("US".to_string()),
    };
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![stock])),
        empty_price_repo(),
    );

    let req = test::TestRequest::patch()
        .uri("/v1/stocks/US0378331005")
        .set_json(json!({ "sector": "Software" }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["id"], 7);
    assert_eq!(body["symbol"], "AAPL");
    assert_eq!(body["name"], "Apple Inc.");
    assert_eq!(body["isin"], "US0378331005");
    assert_eq!(body["currency"], "USD");
    assert_eq!(body["market_region"], "americas");
    assert_eq!(body["market"], "NASDAQ");
    assert_eq!(body["sector"], "Software");
    assert_eq!(body["industry"], "Consumer Electronics");
    assert_eq!(body["country"], "US");
}

#[actix_web::test]
async fn patch_stock_returns_404_for_unknown_isin() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::patch()
        .uri("/v1/stocks/US9999999999")
        .set_json(json!({ "sector": "Technology" }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn patch_stock_returns_422_when_isin_format_invalid() {
    let stocks = vec![test_stock(7, "AAPL", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::patch()
        .uri("/v1/stocks/US0000000007")
        .set_json(json!({ "isin": "bogus" }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[actix_web::test]
async fn patch_stock_returns_409_when_isin_collides_with_other() {
    // Two stocks with valid ISO 6166 ISINs — patch stock 2 with stock 1's ISIN
    // and expect a 409 from the UNIQUE-constraint mapping.
    let mut s1 = test_stock(1, "AAPL", "USD");
    s1.isin = Isin::try_new("US0378331005").unwrap();
    let mut s2 = test_stock(2, "MSFT", "USD");
    s2.isin = Isin::try_new("US5949181045").unwrap();
    let app = make_service!(
        Arc::new(InMemoryStockRepo::new(vec![s1, s2])),
        empty_price_repo(),
    );

    let req = test::TestRequest::patch()
        .uri("/v1/stocks/US5949181045")
        .set_json(json!({ "isin": "US0378331005" }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["detail"].as_str().unwrap_or_default().contains("isin"),
        "conflict detail should identify the colliding field"
    );
}

#[actix_web::test]
async fn patch_stock_returns_409_when_symbol_collides_with_other() {
    let stocks = vec![
        test_stock(1, "AAPL", "USD"),
        test_stock(2, "MSFT", "USD"),
    ];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    // Try to rename stock 2 (MSFT) to AAPL — already taken by stock 1.
    let req = test::TestRequest::patch()
        .uri("/v1/stocks/US0000000002")
        .set_json(json!({ "symbol": "AAPL" }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

// ============================ DELETE /v1/stocks/{isin} =======================

#[actix_web::test]
async fn delete_stock_returns_204_when_no_references() {
    let stocks = vec![test_stock(5, "AAPL", "USD")];
    let app = make_service!(Arc::new(InMemoryStockRepo::new(stocks)), empty_price_repo());

    let req = test::TestRequest::delete()
        .uri("/v1/stocks/US0000000005")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Subsequent GET must return 404 — the stock is really gone.
    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000005")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn delete_stock_returns_404_for_unknown_isin() {
    let app = make_service!(Arc::new(InMemoryStockRepo::default()), empty_price_repo());

    let req = test::TestRequest::delete()
        .uri("/v1/stocks/US9999999999")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn delete_stock_returns_409_when_transaction_references_it() {
    // Mirrors the prod path: `PgStockRepository::delete` maps the
    // `transactions_stock_id_fkey ON DELETE RESTRICT` violation to a 409.
    // The fake reproduces that mapping so the controller surface stays covered.
    let stocks = vec![test_stock(5, "AAPL", "USD")];
    let stock_repo = Arc::new(InMemoryStockRepo::with_fk_protected(stocks, &[5]));
    let app = make_service!(stock_repo, empty_price_repo());

    let req = test::TestRequest::delete()
        .uri("/v1/stocks/US0000000005")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("transactions"),
        "conflict reason should mention the blocking reference"
    );

    // Stock must NOT have been deleted by the failed call.
    let req = test::TestRequest::get()
        .uri("/v1/stocks/US0000000005")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
