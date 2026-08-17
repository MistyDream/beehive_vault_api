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
async fn monthly_flows_are_aggregated_and_traceable() {
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

    let household_id = create_household(&application, "Monthly flow household").await;
    let checking_id = create_account(&application, &household_id, "Checking", "checking").await;
    let credit_card_id =
        create_account(&application, &household_id, "Credit card", "credit_card").await;
    let salary_category_id = category_id(&application, &household_id, "Salaire").await;
    let food_category_id = category_id(&application, &household_id, "Alimentation").await;

    create_transaction(
        &application,
        &household_id,
        &checking_id,
        "2026-08-01",
        "Salary",
        "3000.00",
        "income",
        Some(&salary_category_id),
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &credit_card_id,
        "2026-08-02",
        "Card reward",
        "-100.00",
        "income",
        None,
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &checking_id,
        "2026-08-03",
        "Groceries",
        "-200.00",
        "expense",
        Some(&food_category_id),
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &credit_card_id,
        "2026-08-04",
        "Card groceries",
        "80.00",
        "expense",
        Some(&food_category_id),
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &checking_id,
        "2026-08-05",
        "Grocery refund",
        "20.00",
        "expense",
        Some(&food_category_id),
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &checking_id,
        "2026-08-06",
        "Cash purchase",
        "-40.00",
        "expense",
        None,
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &credit_card_id,
        "2026-08-07",
        "Uncategorized card purchase",
        "10.00",
        "expense",
        None,
    )
    .await;
    create_transaction(
        &application,
        &household_id,
        &checking_id,
        "2026-07-31",
        "Previous month",
        "-500.00",
        "expense",
        None,
    )
    .await;
    let deleted_transaction_id = create_transaction(
        &application,
        &household_id,
        &checking_id,
        "2026-08-08",
        "Deleted purchase",
        "-999.00",
        "expense",
        None,
    )
    .await;
    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/transactions/{deleted_transaction_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;

    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/transfers"),
        json!({
            "amount": "50.00",
            "source": {
                "accountId": checking_id,
                "bookingDate": "2026-08-09",
                "label": "Card payment"
            },
            "destination": {
                "accountId": credit_card_id,
                "bookingDate": "2026-08-09",
                "label": "Payment received"
            }
        }),
        StatusCode::CREATED,
    )
    .await;

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/categories/{food_category_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;
    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/accounts/{credit_card_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;

    let other_household_id = create_household(&application, "Other household").await;
    let other_account_id = create_account(
        &application,
        &other_household_id,
        "Other account",
        "checking",
    )
    .await;
    create_transaction(
        &application,
        &other_household_id,
        &other_account_id,
        "2026-08-10",
        "Other household expense",
        "-1000.00",
        "expense",
        None,
    )
    .await;

    let report = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/monthly-flows/2026-08"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(report["month"], "2026-08");
    assert_eq!(report["dateFrom"], "2026-08-01");
    assert_eq!(report["dateTo"], "2026-08-31");
    assert_eq!(report["currency"], "EUR");
    assert_eq!(report["income"]["total"], "3100.0000");
    assert_eq!(report["income"]["transactionCount"], 2);
    assert_eq!(report["expenses"]["total"], "310.0000");
    assert_eq!(report["expenses"]["transactionCount"], 5);
    assert_eq!(report["netFlow"], "2790.0000");

    let expense_categories = report["expenses"]["categories"].as_array().unwrap();
    assert_eq!(expense_categories.len(), 2);
    assert_eq!(expense_categories[0]["categoryId"], food_category_id);
    assert_eq!(expense_categories[0]["categoryName"], "Alimentation");
    assert_eq!(expense_categories[0]["amount"], "260.0000");
    assert_eq!(expense_categories[0]["transactionCount"], 3);
    assert_eq!(expense_categories[1]["categoryId"], Value::Null);
    assert_eq!(expense_categories[1]["categoryName"], Value::Null);
    assert_eq!(expense_categories[1]["amount"], "50.0000");
    assert_eq!(expense_categories[1]["transactionCount"], 2);

    let uncategorized = send_json(
        &application,
        "GET",
        &format!(
            "/v1/households/{household_id}/transactions?dateFrom=2026-08-01&dateTo=2026-08-31&nature=expense&uncategorized=true"
        ),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(uncategorized.as_array().unwrap().len(), 2);

    let all_uncategorized = send_json(
        &application,
        "GET",
        &format!(
            "/v1/households/{household_id}/transactions?dateFrom=2026-08-01&dateTo=2026-08-31&uncategorized=true"
        ),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(all_uncategorized.as_array().unwrap().len(), 3);

    send_json(
        &application,
        "GET",
        &format!(
            "/v1/households/{household_id}/transactions?categoryId={food_category_id}&uncategorized=true"
        ),
        Value::Null,
        StatusCode::BAD_REQUEST,
    )
    .await;

    let empty_report = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/monthly-flows/2026-09"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(empty_report["income"]["total"], "0.0000");
    assert_eq!(empty_report["expenses"]["total"], "0.0000");
    assert_eq!(empty_report["netFlow"], "0.0000");
    assert!(
        empty_report["income"]["categories"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/monthly-flows/2026-8"),
        Value::Null,
        StatusCode::BAD_REQUEST,
    )
    .await;
    send_json(
        &application,
        "GET",
        "/v1/households/00000000-0000-0000-0000-000000000000/monthly-flows/2026-08",
        Value::Null,
        StatusCode::NOT_FOUND,
    )
    .await;

    for household_id in [household_id, other_household_id] {
        sqlx::query("DELETE FROM households WHERE id = $1::uuid")
            .bind(household_id)
            .execute(&db)
            .await
            .expect("test household cleanup should succeed");
    }
}

async fn create_household(application: &Router, name: &str) -> String {
    let household = send_json(
        application,
        "POST",
        "/v1/households",
        json!({
            "name": name,
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris"
        }),
        StatusCode::CREATED,
    )
    .await;

    household["id"].as_str().unwrap().to_owned()
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
            "initialBalance": "0.00",
            "balanceDate": "2026-07-01"
        }),
        StatusCode::CREATED,
    )
    .await;

    account["id"].as_str().unwrap().to_owned()
}

async fn category_id(application: &Router, household_id: &str, name: &str) -> String {
    let categories = send_json(
        application,
        "GET",
        &format!("/v1/households/{household_id}/categories"),
        Value::Null,
        StatusCode::OK,
    )
    .await;

    categories
        .as_array()
        .unwrap()
        .iter()
        .find(|category| category["name"] == name)
        .expect("the initial category should exist")["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[allow(clippy::too_many_arguments)]
async fn create_transaction(
    application: &Router,
    household_id: &str,
    account_id: &str,
    booking_date: &str,
    label: &str,
    amount: &str,
    nature: &str,
    category_id: Option<&str>,
) -> String {
    let transaction = send_json(
        application,
        "POST",
        &format!("/v1/households/{household_id}/transactions"),
        json!({
            "accountId": account_id,
            "bookingDate": booking_date,
            "label": label,
            "amount": amount,
            "nature": nature,
            "categoryId": category_id
        }),
        StatusCode::CREATED,
    )
    .await;

    transaction["id"].as_str().unwrap().to_owned()
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
