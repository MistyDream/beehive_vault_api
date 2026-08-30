use std::env;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use beehive_vault_api::app;
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn balance_dates_and_corrections_follow_reconciliation_rules() {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must target an isolated test database");
    let db = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("test database should be reachable");
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("test migrations should succeed");
    let application = app::build(db.clone());

    let current_date = Utc::now()
        .with_timezone(&chrono_tz::Europe::Paris)
        .date_naive();
    let initial_date = current_date - Duration::days(3);
    let reconciliation_date = current_date - Duration::days(2);
    let latest_date = current_date - Duration::days(1);
    let future_date = current_date + Duration::days(1);

    let household = send_json(
        &application,
        "POST",
        "/v1/households",
        json!({
            "name": format!("Balance lifecycle household {}", Uuid::now_v7()),
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris"
        }),
        StatusCode::CREATED,
    )
    .await;
    let household_id = household["id"].as_str().unwrap();

    let future_initial_balance = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "name": "Future account",
            "kind": "checking",
            "currency": "EUR",
            "initialBalance": "100.00",
            "balanceDate": future_date
        }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_validation_error(
        &future_initial_balance,
        "#/balanceDate",
        "balance_date_in_future",
    );

    let account = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "name": "Checking account",
            "kind": "checking",
            "currency": "EUR",
            "initialBalance": "100.00",
            "balanceDate": initial_date
        }),
        StatusCode::CREATED,
    )
    .await;
    let account_id = account["id"].as_str().unwrap();
    let balances_uri = format!("/v1/households/{household_id}/accounts/{account_id}/balances");

    let future_balance = send_json(
        &application,
        "POST",
        &balances_uri,
        json!({ "amount": "150.00", "balanceDate": future_date }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_validation_error(&future_balance, "#/balanceDate", "balance_date_in_future");

    let non_chronological_balance = send_json(
        &application,
        "POST",
        &balances_uri,
        json!({ "amount": "150.00", "balanceDate": initial_date }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_validation_error(
        &non_chronological_balance,
        "#/balanceDate",
        "balance_date_not_after_latest",
    );

    let reconciliation = send_json(
        &application,
        "POST",
        &balances_uri,
        json!({
            "amount": "200.00",
            "balanceDate": reconciliation_date,
            "source": "reconciliation"
        }),
        StatusCode::CREATED,
    )
    .await;
    let reconciliation_id = reconciliation["id"].as_str().unwrap();

    let empty_update = send_json(
        &application,
        "PATCH",
        &format!("{balances_uri}/{reconciliation_id}"),
        json!({}),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_validation_error(&empty_update, "#/", "empty_update");

    let future_update = send_json(
        &application,
        "PATCH",
        &format!("{balances_uri}/{reconciliation_id}"),
        json!({ "balanceDate": future_date }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_validation_error(&future_update, "#/balanceDate", "balance_date_in_future");

    let amount_correction = send_json(
        &application,
        "PATCH",
        &format!("{balances_uri}/{reconciliation_id}"),
        json!({ "amount": "225.00" }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(amount_correction["amount"], "225.0000");
    assert_eq!(amount_correction["source"], "reconciliation");
    assert_eq!(
        amount_correction["balanceDate"],
        reconciliation_date.to_string()
    );

    let missing_balance = send_json(
        &application,
        "PATCH",
        &format!("{balances_uri}/{}", Uuid::now_v7()),
        json!({ "amount": "250.00" }),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_balance["code"], "balance_not_found");

    send_json(
        &application,
        "POST",
        &balances_uri,
        json!({ "amount": "250.00", "balanceDate": latest_date }),
        StatusCode::CREATED,
    )
    .await;

    let duplicate_date = send_json(
        &application,
        "PATCH",
        &format!("{balances_uri}/{reconciliation_id}"),
        json!({ "balanceDate": latest_date }),
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(duplicate_date["code"], "duplicate_balance_date");

    let corrected = send_json(
        &application,
        "PATCH",
        &format!("{balances_uri}/{reconciliation_id}"),
        json!({
            "amount": "300.00",
            "balanceDate": current_date
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(corrected["amount"], "300.0000");
    assert_eq!(corrected["balanceDate"], current_date.to_string());
    assert_eq!(corrected["source"], "reconciliation");

    let balances = send_json(
        &application,
        "GET",
        &balances_uri,
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(balances[0]["id"], reconciliation_id);
    assert_eq!(balances[0]["balanceDate"], current_date.to_string());

    sqlx::query("DELETE FROM households WHERE id = $1::uuid")
        .bind(household_id)
        .execute(&db)
        .await
        .expect("test household cleanup should succeed");
}

fn assert_validation_error(problem: &Value, pointer: &str, code: &str) {
    assert_eq!(problem["code"], "validation_error");
    assert_eq!(problem["errors"][0]["pointer"], pointer);
    assert_eq!(problem["errors"][0]["code"], code);
}

async fn send_json(
    application: &Router,
    method: &str,
    uri: &str,
    body: Value,
    expected_status: StatusCode,
) -> Value {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(if body.is_null() {
            Body::empty()
        } else {
            Body::from(body.to_string())
        })
        .unwrap();
    let response = application.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    assert_eq!(
        status,
        expected_status,
        "unexpected response body: {}",
        String::from_utf8_lossy(&bytes)
    );
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}
