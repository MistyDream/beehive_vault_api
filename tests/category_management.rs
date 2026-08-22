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
async fn household_categories_can_be_managed() {
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

    let household = create_household(&application, "Primary household").await;
    let household_id = household["id"].as_str().unwrap();

    let initial_categories = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/categories"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(initial_categories.as_array().unwrap().len(), 19);

    let income_categories = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household_id}/categories?kind=income"),
        Value::Null,
        StatusCode::OK,
    )
    .await;
    assert_eq!(income_categories.as_array().unwrap().len(), 6);

    let category = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/categories"),
        json!({ "name": "Pet care", "kind": "expense" }),
        StatusCode::CREATED,
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    let duplicate_problem = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/categories"),
        json!({ "name": "  PET CARE  ", "kind": "expense" }),
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(duplicate_problem["code"], "duplicate_category_name");
    assert_eq!(
        duplicate_problem["type"],
        "urn:beehive-vault:problem:duplicate-category-name"
    );
    assert!(duplicate_problem.get("detail").is_none());

    let renamed_category = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{household_id}/categories/{category_id}"),
        json!({ "name": "Animal care" }),
        StatusCode::OK,
    )
    .await;
    assert_eq!(renamed_category["name"], "Animal care");
    assert_eq!(renamed_category["kind"], "expense");

    let other_household = create_household(&application, "Other household").await;
    let other_household_id = other_household["id"].as_str().unwrap();
    let not_found_problem = send_json(
        &application,
        "PATCH",
        &format!("/v1/households/{other_household_id}/categories/{category_id}"),
        json!({ "name": "Unavailable category" }),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(not_found_problem["code"], "category_not_found");

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household_id}/categories/{category_id}"),
        Value::Null,
        StatusCode::NO_CONTENT,
    )
    .await;

    let replacement = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household_id}/categories"),
        json!({ "name": "ANIMAL CARE", "kind": "expense" }),
        StatusCode::CREATED,
    )
    .await;
    assert_eq!(replacement["name"], "ANIMAL CARE");

    for id in [household_id, other_household_id] {
        sqlx::query("DELETE FROM households WHERE id = $1::uuid")
            .bind(id)
            .execute(&db)
            .await
            .expect("test household cleanup should succeed");
    }
}

async fn create_household(application: &Router, name: &str) -> Value {
    send_json(
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
    assert_eq!(response.status(), expected_status);
    if expected_status.is_client_error() || expected_status.is_server_error() {
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/problem+json"
        );
    }
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}
