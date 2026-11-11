use std::{env, str::FromStr};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use beehive_vault_api::{
    app,
    database::Database,
    features::demo::{DEMO_CHECKING_ACCOUNT_ID, DEMO_HOUSEHOLD_ID, DEMO_STATEMENT, DemoRepository},
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires an isolated PostgreSQL test database"]
async fn demo_can_be_replayed_imported_and_reset_without_touching_other_households() {
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&env::var("TEST_DATABASE_URL").expect("isolated TEST_DATABASE_URL is required"))
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    let demo = DemoRepository::new(Database::new(pool.clone()));
    demo.reset().await.unwrap();
    let (left, right) = tokio::join!(demo.load(), demo.load());
    assert_ne!(left.unwrap(), right.unwrap());
    assert!(!demo.load().await.unwrap());
    let application = app::build(pool.clone());
    let household = format!("/v1/households/{DEMO_HOUSEHOLD_ID}");
    let summary = request(&application, "GET", &format!("{household}/summary"), None).await;
    assert_amount(&summary["assets"], "10745");
    assert_amount(&summary["liabilities"], "9700");
    assert_amount(&summary["netWorth"], "1045");
    for (month, income, expenses, net) in [
        ("2026-08", "2500", "990", "1510"),
        ("2026-09", "2600", "1025", "1575"),
        ("2026-10", "2700", "1040", "1660"),
    ] {
        let report = request(
            &application,
            "GET",
            &format!("{household}/monthly-flows/{month}"),
            None,
        )
        .await;
        assert_amount(&report["income"]["total"], income);
        assert_amount(&report["expenses"]["total"], expenses);
        assert_amount(&report["netFlow"], net);
        assert!(
            report["expenses"]["categories"]
                .as_array()
                .unwrap()
                .iter()
                .any(|category| category["categoryId"].is_null())
        );
    }
    let base = format!("{household}/accounts/{DEMO_CHECKING_ACCOUNT_ID}/imports/csv");
    let config = json!({"csv": DEMO_STATEMENT, "format": {"delimiter": "semicolon", "dateFormat": "day_month_year", "decimalSeparator": "comma", "columns": {"bookingDate": "Booking", "label": "Description", "amount": "Amount"}}});
    let preview = request(
        &application,
        "POST",
        &format!("{base}/preview"),
        Some(config.clone()),
    )
    .await;
    assert_eq!(preview["rows"].as_array().unwrap().len(), 3);
    assert!(
        preview["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == "valid")
    );
    let mut confirmation = config;
    confirmation["fileName"] = json!("statement.csv");
    confirmation["rows"] = json!([
        {"line":2,"nature":"expense","effect":"standard"},
        {"line":3,"nature":"expense","effect":"standard"},
        {"line":4,"nature":"income","effect":"standard"}
    ]);
    let imported = request(
        &application,
        "POST",
        &format!("{base}/confirm"),
        Some(confirmation.clone()),
    )
    .await;
    assert!(
        imported["rows"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["status"] == "imported")
    );
    let duplicate = request(
        &application,
        "POST",
        &format!("{base}/confirm"),
        Some(confirmation),
    )
    .await;
    for (new, old) in duplicate["rows"]
        .as_array()
        .unwrap()
        .iter()
        .zip(imported["rows"].as_array().unwrap())
    {
        assert_eq!(new["status"], "duplicate");
        assert_eq!(new["transactionId"], old["transactionId"]);
    }
    assert!(!demo.load().await.unwrap());
    let summary = request(&application, "GET", &format!("{household}/summary"), None).await;
    assert_amount(&summary["assets"], "10780");
    assert_amount(&summary["netWorth"], "1080");
    let report = request(
        &application,
        "GET",
        &format!("{household}/monthly-flows/2026-10"),
        None,
    )
    .await;
    assert_amount(&report["income"]["total"], "2775.25");
    assert_amount(&report["expenses"]["total"], "1080.25");
    assert_amount(&report["netFlow"], "1695");

    let other = Uuid::now_v7();
    let account = Uuid::now_v7();
    sqlx::query("INSERT INTO households (id,name,base_currency,timezone) VALUES ($1,'Other test household','EUR','UTC')")
        .bind(other).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO accounts (id,household_id,institution_id,name,kind,currency) VALUES ($1,$2,'019d9000-0000-7000-8000-00000000000b','Other account','checking','EUR')")
        .bind(account).bind(other).execute(&pool).await.unwrap();
    assert!(demo.reset().await.unwrap());
    assert!(!demo.reset().await.unwrap());
    let institution: String =
        sqlx::query_scalar("SELECT institution_id::text FROM accounts WHERE id=$1")
            .bind(account)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(institution, "019d9000-0000-7000-8000-00000000000b");
    assert!(demo.load().await.unwrap());
    assert!(demo.reset().await.unwrap());
    sqlx::query("DELETE FROM households WHERE id=$1")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM institutions WHERE id='019d9000-0000-7000-8000-00000000000b'")
        .execute(&pool)
        .await
        .unwrap();

    let conflicting_name = Uuid::now_v7();
    sqlx::query("INSERT INTO institutions (id,name) VALUES ($1,'Demo Investment Bank')")
        .bind(conflicting_name)
        .execute(&pool)
        .await
        .unwrap();
    assert!(demo.load().await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM households WHERE id=$1::uuid")
        .bind(DEMO_HOUSEHOLD_ID)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM institutions WHERE id='019d9000-0000-7000-8000-00000000000b'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 0);
    sqlx::query("DELETE FROM institutions WHERE id=$1")
        .bind(conflicting_name)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO institutions (id,name) VALUES ('019d9000-0000-7000-8000-00000000000c','Conflicting test bank')").execute(&pool).await.unwrap();
    assert!(demo.load().await.is_err());
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM households WHERE id=$1::uuid")
        .bind(DEMO_HOUSEHOLD_ID)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    sqlx::query("DELETE FROM institutions WHERE id='019d9000-0000-7000-8000-00000000000c'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO households (id,name,base_currency,timezone) VALUES ($1::uuid,'Conflicting test household','EUR','UTC')")
        .bind(DEMO_HOUSEHOLD_ID).execute(&pool).await.unwrap();
    assert!(demo.load().await.is_err());
    assert!(demo.reset().await.is_err());
    sqlx::query("DELETE FROM households WHERE id=$1::uuid")
        .bind(DEMO_HOUSEHOLD_ID)
        .execute(&pool)
        .await
        .unwrap();
}

fn assert_amount(value: &Value, expected: &str) {
    assert_eq!(
        Decimal::from_str(value.as_str().unwrap()).unwrap(),
        Decimal::from_str(expected).unwrap()
    );
}

async fn request(application: &Router, method: &str, path: &str, body: Option<Value>) -> Value {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
        .unwrap();
    let response = application.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        status,
        StatusCode::OK,
        "{}",
        String::from_utf8_lossy(&bytes)
    );
    serde_json::from_slice(&bytes).unwrap()
}
