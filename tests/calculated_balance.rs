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
async fn calculated_balances_follow_reconciliation_rules() {
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
            "name": "Calculated balance household",
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
        "1000.00",
    )
    .await;
    let loan_id = create_account(
        &application,
        household_id,
        "Personal loan",
        "loan",
        "300.00",
    )
    .await;

    create_transaction(
        &application,
        household_id,
        &checking_id,
        "2026-08-10",
        "Same-day transaction",
        "100.00",
        "expense",
    )
    .await;
    let later_transaction = create_transaction(
        &application,
        household_id,
        &checking_id,
        "2026-08-11",
        "Later transaction",
        "50.00",
        "expense",
    )
    .await;
    create_transaction(
        &application,
        household_id,
        &loan_id,
        "2026-08-11",
        "Debt increase",
        "20.00",
        "expense",
    )
    .await;

    let checking = get_account(&application, household_id, &checking_id).await;
    assert_eq!(checking["latestBalance"], "1000.0000");
    assert_eq!(checking["balanceDate"], "2026-08-10");
    assert_eq!(checking["calculatedBalance"], "950.0000");

    let summary = get_summary(&application, household_id).await;
    assert_eq!(summary["assets"], "950.0000");
    assert_eq!(summary["liabilities"], "320.0000");
    assert_eq!(summary["netWorth"], "630.0000");

    let later_transaction_id = later_transaction["id"].as_str().unwrap();
    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/transactions/{later_transaction_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    let checking = get_account(&application, household_id, &checking_id).await;
    assert_eq!(checking["calculatedBalance"], "1000.0000");

    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts/{checking_id}/balances"),
        json!({
            "amount": "900.00",
            "balanceDate": "2026-08-11",
            "source": "reconciliation"
        }),
        StatusCode::CREATED,
    )
    .await;
    create_transaction(
        &application,
        household_id,
        &checking_id,
        "2026-08-12",
        "Post-reconciliation transaction",
        "25.00",
        "expense",
    )
    .await;
    let checking = get_account(&application, household_id, &checking_id).await;
    assert_eq!(checking["latestBalance"], "900.0000");
    assert_eq!(checking["balanceDate"], "2026-08-11");
    assert_eq!(checking["calculatedBalance"], "875.0000");

    let transfer = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/transfers"),
        json!({
            "amount": "50.00",
            "source": {
                "accountId": checking_id,
                "bookingDate": "2026-08-12",
                "label": "Loan payment"
            },
            "destination": {
                "accountId": loan_id,
                "bookingDate": "2026-08-12",
                "label": "Payment received"
            }
        }),
        StatusCode::CREATED,
    )
    .await;
    let transfer_id = transfer["id"].as_str().unwrap();

    let checking = get_account(&application, household_id, &checking_id).await;
    let loan = get_account(&application, household_id, &loan_id).await;
    assert_eq!(checking["calculatedBalance"], "825.0000");
    assert_eq!(loan["calculatedBalance"], "270.0000");
    let summary = get_summary(&application, household_id).await;
    assert_eq!(summary["assets"], "825.0000");
    assert_eq!(summary["liabilities"], "270.0000");
    assert_eq!(summary["netWorth"], "555.0000");

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/transfers/{transfer_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    let checking = get_account(&application, household_id, &checking_id).await;
    let loan = get_account(&application, household_id, &loan_id).await;
    assert_eq!(checking["calculatedBalance"], "875.0000");
    assert_eq!(loan["calculatedBalance"], "320.0000");

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
            "balanceDate": "2026-08-10"
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
) -> Value {
    send_json(
        application,
        "POST",
        &format!("/v1/households/{household_id}/transactions"),
        json!({
            "accountId": account_id,
            "bookingDate": booking_date,
            "label": label,
            "amount": amount,
            "effect": "standard",
            "nature": nature
        }),
        StatusCode::CREATED,
    )
    .await
}

async fn get_account(application: &Router, household_id: &str, account_id: &str) -> Value {
    send_json(
        application,
        "GET",
        &format!("/v1/households/{household_id}/accounts/{account_id}"),
        Value::Null,
        StatusCode::OK,
    )
    .await
}

async fn get_summary(application: &Router, household_id: &str) -> Value {
    send_json(
        application,
        "GET",
        &format!("/v1/households/{household_id}/summary"),
        Value::Null,
        StatusCode::OK,
    )
    .await
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
