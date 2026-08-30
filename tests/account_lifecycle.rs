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
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn account_collections_archive_and_restore_follow_the_client_contract() {
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
            "name": "Account lifecycle household",
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris"
        }),
        StatusCode::CREATED,
    )
    .await;
    let household_id = household["id"].as_str().unwrap();

    let checking_id = create_account(
        &application,
        household_id,
        "Checking account",
        "checking",
        "100.00",
    )
    .await;
    create_account(&application, household_id, "Cash wallet", "cash", "-20.00").await;
    create_account(
        &application,
        household_id,
        "Savings account",
        "savings",
        "200.00",
    )
    .await;
    create_account(
        &application,
        household_id,
        "Personal loan",
        "loan",
        "300.00",
    )
    .await;
    let empty_account_id = create_account(
        &application,
        household_id,
        "Empty account",
        "checking",
        "0.00",
    )
    .await;

    let active_accounts = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/accounts"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(active_accounts["items"].as_array().unwrap().len(), 5);
    assert_eq!(active_accounts["totals"]["daily"], "80.0000");
    assert_eq!(active_accounts["totals"]["savings"], "200.0000");
    assert_eq!(active_accounts["totals"]["liabilities"], "300.0000");

    let invalid_status = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/accounts?status=unknown"),
        Value::Null,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_eq!(invalid_status["code"], "validation_error");
    assert_eq!(invalid_status["errors"][0]["location"], "query");

    let nonzero_archive = send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/accounts/{checking_id}"),
        Value::Null,
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(nonzero_archive["code"], "account_balance_not_zero");

    create_transaction(
        &application,
        household_id,
        &empty_account_id,
        "2026-08-21",
        "Temporary income",
        "10.00",
        "income",
    )
    .await;
    let transaction_balance_archive = send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/accounts/{empty_account_id}"),
        Value::Null,
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(
        transaction_balance_archive["code"],
        "account_balance_not_zero"
    );
    create_transaction(
        &application,
        household_id,
        &empty_account_id,
        "2026-08-22",
        "Balancing expense",
        "-10.00",
        "expense",
    )
    .await;
    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/accounts/{empty_account_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;

    let archived_account = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/accounts/{empty_account_id}"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert!(archived_account["archivedAt"].is_string());

    let archived_update = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/accounts/{empty_account_id}"),
        json!({ "name": "Still archived" }),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(archived_update["code"], "account_not_found");

    let archived_accounts = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/accounts?status=archived"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(archived_accounts["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        archived_accounts["items"][0]["id"],
        empty_account_id.as_str()
    );
    assert_eq!(archived_accounts["totals"]["daily"], "0.0000");

    let restore_uri = format!("/v1/households/{household_id}/accounts/{empty_account_id}/restore");
    let restored = send_json(
        &application,
        "POST",
        &restore_uri,
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert!(restored["archivedAt"].is_null());
    let restored_again = send_json(
        &application,
        "POST",
        &restore_uri,
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(restored_again["id"], empty_account_id.as_str());
    assert!(restored_again["archivedAt"].is_null());

    let missing_account_id = Uuid::now_v7();
    let missing_restore = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts/{missing_account_id}/restore"),
        Value::Null,
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(missing_restore["code"], "account_not_found");

    let active_accounts = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/accounts?status=active"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(active_accounts["items"].as_array().unwrap().len(), 5);

    sqlx::query("DELETE FROM households WHERE id = $1::uuid")
        .bind(household_id)
        .execute(&db)
        .await
        .expect("test household cleanup should succeed");
}

async fn create_account(
    application: &Router,
    household_id: &str,
    name: &str,
    kind: &str,
    initial_balance: &str,
) -> String {
    let account = send_json(
        application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "institutionId": null,
            "name": name,
            "kind": kind,
            "currency": "EUR",
            "initialBalance": initial_balance,
            "balanceDate": "2026-08-20"
        }),
        StatusCode::CREATED,
    )
    .await;

    account["id"].as_str().unwrap().to_owned()
}

async fn create_transaction(
    application: &Router,
    household_id: &str,
    account_id: &str,
    booking_date: &str,
    label: &str,
    amount: &str,
    nature: &str,
) {
    send_json(
        application,
        "POST",
        &format!("/v1/households/{household_id}/transactions"),
        json!({
            "accountId": account_id,
            "bookingDate": booking_date,
            "label": label,
            "amount": amount,
            "nature": nature
        }),
        StatusCode::CREATED,
    )
    .await;
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
