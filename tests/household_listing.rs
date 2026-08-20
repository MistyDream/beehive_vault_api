use std::env;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use beehive_vault_api::app;
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn households_can_be_listed_for_active_household_resolution() {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must target an isolated test database");
    let schema = format!("household_listing_{}", Uuid::now_v7().simple());
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("test database should be reachable");
    // The identifier is generated exclusively from a fixed prefix and UUID hexadecimal digits.
    sqlx::query(AssertSqlSafe(format!("CREATE SCHEMA \"{schema}\"")))
        .execute(&admin)
        .await
        .expect("isolated test schema should be created");

    let connection_schema = schema.clone();
    let db = PgPoolOptions::new()
        .max_connections(2)
        .after_connect(move |connection, _metadata| {
            let search_path = format!("SET search_path TO \"{connection_schema}\"");
            Box::pin(async move {
                // The identifier is generated exclusively from a fixed prefix and UUID hexadecimal digits.
                sqlx::query(AssertSqlSafe(search_path))
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .expect("isolated test schema should be reachable");
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("test migrations should succeed");
    let application = app::build(db.clone());

    let empty_households = get_json(&application, "/v1/households", StatusCode::OK).await;
    assert_eq!(empty_households, json!([]));

    let second_alphabetically = create_household(&application, "zebra household").await;
    let first_alphabetically = create_household(&application, "Alpha household").await;

    let households = get_json(&application, "/v1/households", StatusCode::OK).await;
    assert_eq!(
        households,
        json!([first_alphabetically, second_alphabetically])
    );

    db.close().await;
    // The identifier is generated exclusively from a fixed prefix and UUID hexadecimal digits.
    sqlx::query(AssertSqlSafe(format!("DROP SCHEMA \"{schema}\" CASCADE")))
        .execute(&admin)
        .await
        .expect("isolated test schema should be removed");
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

async fn get_json(application: &Router, uri: &str, expected_status: StatusCode) -> Value {
    send_json(application, "GET", uri, Value::Null, expected_status).await
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
