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
async fn household_transfers_can_be_managed_atomically() {
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
            "name": "Transfer test household",
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris"
        }),
        StatusCode::CREATED,
    )
    .await;
    let household_id = household["id"].as_str().unwrap();
    let checking_id = create_account(&application, household_id, "Checking", "checking").await;
    let savings_id = create_account(&application, household_id, "Savings", "savings").await;
    let card_id = create_account(&application, household_id, "Credit card", "credit_card").await;
    let loan_id = create_account(&application, household_id, "Loan", "loan").await;
    let corrected_account_id =
        create_account(&application, household_id, "Corrected account", "checking").await;
    let corrected_account = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/accounts/{corrected_account_id}"),
        json!({ "kind": "loan" }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(corrected_account["kind"], "loan");

    let asset_to_asset = create_transfer(
        &application,
        household_id,
        &checking_id,
        &savings_id,
        "Asset to asset",
    )
    .await;
    assert_eq!(asset_to_asset["amount"], "500.0000");
    assert_eq!(asset_to_asset["source"]["amount"], "-500.0000");
    assert_eq!(asset_to_asset["destination"]["amount"], "500.0000");

    let liability_to_liability = create_transfer(
        &application,
        household_id,
        &card_id,
        &loan_id,
        "Liability to liability",
    )
    .await;
    assert_eq!(liability_to_liability["source"]["amount"], "500.0000");
    assert_eq!(liability_to_liability["destination"]["amount"], "-500.0000");

    let asset_to_liability = create_transfer(
        &application,
        household_id,
        &savings_id,
        &card_id,
        "Asset to liability",
    )
    .await;
    assert_eq!(asset_to_liability["source"]["amount"], "-500.0000");
    assert_eq!(asset_to_liability["destination"]["amount"], "-500.0000");

    let liability_to_asset = create_transfer(
        &application,
        household_id,
        &loan_id,
        &checking_id,
        "Liability to asset",
    )
    .await;
    assert_eq!(liability_to_asset["source"]["amount"], "500.0000");
    assert_eq!(liability_to_asset["destination"]["amount"], "500.0000");

    let account_in_same_family = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/accounts/{savings_id}"),
        json!({ "kind": "investment" }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(account_in_same_family["kind"], "investment");
    send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/accounts/{checking_id}"),
        json!({ "kind": "loan" }),
        StatusCode::CONFLICT,
    )
    .await;

    let transfers = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transfers"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(transfers["items"].as_array().unwrap().len(), 4);
    assert_eq!(transfers["page"], 1);
    assert_eq!(transfers["limit"], 50);
    assert_eq!(transfers["total"], 4);
    let second_page = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transfers?page=2&limit=2"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(second_page["items"].as_array().unwrap().len(), 2);
    assert_eq!(second_page["page"], 2);
    assert_eq!(second_page["limit"], 2);
    assert_eq!(second_page["total"], 4);
    send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transfers?page=0"),
        Value::Null,
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;

    let transfer_transactions = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transactions?nature=transfer"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(transfer_transactions["items"].as_array().unwrap().len(), 4);
    assert_eq!(transfer_transactions["total"], 4);
    assert!(
        transfer_transactions["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|operation| operation["operationType"] == "transfer")
    );
    let consolidated_asset_transfer = transfer_transactions["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|operation| operation["id"] == asset_to_asset["id"])
        .expect("the transfer should be listed once as a consolidated operation");
    assert_eq!(consolidated_asset_transfer["bookingDate"], "2026-08-17");
    assert_eq!(consolidated_asset_transfer["amount"], "500.0000");
    assert_eq!(
        consolidated_asset_transfer["source"]["account"]["id"],
        checking_id
    );
    assert_eq!(
        consolidated_asset_transfer["destination"]["account"]["id"],
        savings_id
    );
    assert_eq!(
        consolidated_asset_transfer["source"]["accountAmount"],
        "-500.0000"
    );
    assert_eq!(
        consolidated_asset_transfer["destination"]["accountAmount"],
        "500.0000"
    );

    let transfer_id = asset_to_asset["id"].as_str().unwrap();
    let source_transaction_id = asset_to_asset["source"]["transactionId"].as_str().unwrap();
    send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/transactions/{source_transaction_id}"),
        json!({ "note": "Isolated update" }),
        StatusCode::CONFLICT,
    )
    .await;

    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts/{checking_id}/balances"),
        json!({
            "amount": "0.00",
            "balanceDate": "2026-08-18",
            "source": "reconciliation"
        }),
        StatusCode::CREATED,
    )
    .await;
    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/accounts/{checking_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    let updated = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/transfers/{transfer_id}"),
        json!({
            "amount": "550.00",
            "source": { "note": "Archived source remains valid" },
            "destination": {
                "bookingDate": "2026-08-16",
                "note": null
            }
        }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(updated["amount"], "550.0000");
    assert_eq!(updated["source"]["amount"], "-550.0000");
    assert_eq!(updated["source"]["note"], "Archived source remains valid");
    assert_eq!(updated["destination"]["bookingDate"], "2026-08-16");
    assert_eq!(updated["destination"]["note"], Value::Null);

    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/transfers"),
        transfer_body(&savings_id, &savings_id, "Invalid same account"),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/transfers"),
        json!({
            "amount": "10.00",
            "source": {
                "accountId": savings_id,
                "bookingDate": "9999-12-31",
                "label": "Future source"
            },
            "destination": {
                "accountId": card_id,
                "bookingDate": "2026-08-17",
                "label": "Future destination"
            }
        }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/transfers/{transfer_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transfers/{transfer_id}"),
        Value::Null,
        StatusCode::NOT_FOUND,
    )
    .await;
    send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/transactions/{source_transaction_id}"),
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

async fn create_account(
    application: &Router,
    household_id: &str,
    name: &str,
    kind: &str,
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
            "initialBalance": "1000.00",
            "balanceDate": "2026-08-10"
        }),
        StatusCode::CREATED,
    )
    .await;

    account["id"].as_str().unwrap().to_owned()
}

async fn create_transfer(
    application: &Router,
    household_id: &str,
    source_account_id: &str,
    destination_account_id: &str,
    label: &str,
) -> Value {
    send_json(
        application,
        "POST",
        &format!("/v1/households/{household_id}/transfers"),
        transfer_body(source_account_id, destination_account_id, label),
        StatusCode::CREATED,
    )
    .await
}

fn transfer_body(source_account_id: &str, destination_account_id: &str, label: &str) -> Value {
    json!({
        "amount": "500.00",
        "source": {
            "accountId": source_account_id,
            "bookingDate": "2026-08-17",
            "label": format!("{label} source"),
            "note": "Source movement"
        },
        "destination": {
            "accountId": destination_account_id,
            "bookingDate": "2026-08-17",
            "label": format!("{label} destination"),
            "note": "Destination movement"
        }
    })
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
    assert_eq!(response.status(), expected_status);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}
