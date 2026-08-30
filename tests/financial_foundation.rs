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
async fn financial_foundation_calculates_net_worth() {
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
    let institution_id = Uuid::now_v7();
    let institution_name = format!("Example Bank {institution_id}");
    sqlx::query("INSERT INTO institutions (id, name) VALUES ($1, $2)")
        .bind(institution_id)
        .bind(&institution_name)
        .execute(&db)
        .await
        .expect("test institution should be created");
    let application = app::build(db.clone());

    let institutions = send_json(
        &application,
        "GET",
        "/v1/institutions",
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert!(institutions.as_array().unwrap().iter().any(|institution| {
        institution["id"] == institution_id.to_string()
            && institution["name"] == institution_name.as_str()
    }));

    let household = send_json(
        &application,
        "POST",
        "/v1/households",
        json!({
            "name": "Personal finances",
            "baseCurrency": "EUR",
            "timezone": "Europe/Paris"
        }),
        StatusCode::CREATED,
    )
    .await;
    let household_id = household["id"].as_str().unwrap();

    let invalid_institution = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "institutionId": Uuid::now_v7(),
            "name": "Invalid institution account",
            "kind": "checking",
            "currency": "EUR",
            "initialBalance": "0.00",
            "balanceDate": "2026-08-10"
        }),
        StatusCode::UNPROCESSABLE_ENTITY,
    )
    .await;
    assert_eq!(
        invalid_institution["errors"][0]["code"],
        "invalid_institution"
    );

    let asset = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "institutionId": institution_id,
            "name": "Checking account",
            "kind": "checking",
            "currency": "EUR",
            "initialBalance": "1200.00",
            "balanceDate": "2026-08-10"
        }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(asset["latestBalance"], "1200.0000");

    send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/accounts"),
        json!({
            "institutionId": institution_id,
            "name": "Personal loan",
            "kind": "loan",
            "currency": "EUR",
            "initialBalance": "300.00",
            "balanceDate": "2026-08-10"
        }),
        StatusCode::CREATED,
    )
    .await;

    let summary = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/summary"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(summary["assets"], "1200.0000");
    assert_eq!(summary["liabilities"], "300.0000");
    assert_eq!(summary["netWorth"], "900.0000");

    sqlx::query("DELETE FROM households WHERE id = $1::uuid")
        .bind(household_id)
        .execute(&db)
        .await
        .expect("test household cleanup should succeed");
    sqlx::query("DELETE FROM institutions WHERE id = $1")
        .bind(institution_id)
        .execute(&db)
        .await
        .expect("test institution cleanup should succeed");
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
