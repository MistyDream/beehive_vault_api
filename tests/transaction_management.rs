use std::env;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use beehive_vault_api::app;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn household_transactions_can_be_managed() {
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

    let household = send_json(
        &application,
        "POST",
        "/v1/households",
        json!({
            "name": "Transaction test household",
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris"
        }),
        StatusCode::CREATED,
    )
    .await;
    let household_id = household["id"].as_str().unwrap();

    let account = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "institutionId": null,
            "name": "Everyday account",
            "kind": "checking",
            "currency": "EUR",
            "initialBalance": "1000.00",
            "balanceDate": "2026-08-10"
        }),
        StatusCode::CREATED,
    )
    .await;
    let account_id = account["id"].as_str().unwrap();

    let categories = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/categories?kind=expense"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    let food_category = categories
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["name"] == "Alimentation")
        .expect("the initial food category should exist");
    let category_id = food_category["id"].as_str().unwrap();

    let created = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/transactions"),
        json!({
            "accountId": account_id,
            "bookingDate": "2026-08-10",
            "label": "Grocery store",
            "amount": "-42.50",
            "nature": "expense",
            "categoryId": category_id,
            "note": "Weekly groceries"
        }),
        StatusCode::CREATED,
    )
    .await;
    let transaction_id = created["id"].as_str().unwrap();
    assert_eq!(created["amount"], "-42.5000");
    assert_eq!(created["nature"], "expense");
    assert_eq!(created["categoryId"], category_id);
    assert_eq!(created["transferId"], Value::Null);
    assert_eq!(created["transferRole"], Value::Null);
    assert_eq!(created["source"], "manual");

    let fetched = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transactions/{transaction_id}"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(fetched["label"], "Grocery store");

    let transactions = send_json(
        &application,
        "GET",
        &format!(
            "/v1/households/{household_id}/transactions?accountId={account_id}&nature=expense&search=grocery&page=1&limit=1"
        ),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(transactions.as_array().unwrap().len(), 1);

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/categories/{category_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    let transaction_with_archived_category = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/transactions/{transaction_id}"),
        json!({ "note": "Receipt checked" }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        transaction_with_archived_category["categoryId"],
        category_id
    );
    assert_eq!(
        transaction_with_archived_category["note"],
        "Receipt checked"
    );

    let updated = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/transactions/{transaction_id}"),
        json!({
            "label": "Supermarket",
            "categoryId": null,
            "note": null
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["label"], "Supermarket");
    assert_eq!(updated["categoryId"], Value::Null);
    assert_eq!(updated["note"], Value::Null);

    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/transactions"),
        json!({
            "accountId": account_id,
            "bookingDate": "9999-12-31",
            "label": "Future transaction",
            "amount": "10.00",
            "nature": "income"
        }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/transactions/{transaction_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transactions/{transaction_id}"),
        Value::Null,
        StatusCode::NOT_FOUND,
    )
    .await;

    sqlx::query("DELETE FROM households WHERE id = $1::uuid")
        .bind(household_id)
        .execute(&db)
        .await
        .expect("test household cleanup should succeed");
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
