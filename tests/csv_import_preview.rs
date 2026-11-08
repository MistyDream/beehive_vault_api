use std::env;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use beehive_vault_api::app;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

const UNKNOWN_ID: &str = "00000000-0000-0000-0000-000000000000";

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn csv_preview_reports_rows_without_changing_financial_state() {
    let application = test_application().await;
    let household = create_household(&application).await;
    let account = create_account(&application, &household, "Statement test account").await;
    let household_path = format!("/v1/households/{household}");
    let account_path = format!("{household_path}/accounts/{account}");
    let transactions_path = format!("{household_path}/transactions");

    send_json(
        &application,
        "POST",
        &transactions_path,
        Some(json!({
            "accountId": account,
            "bookingDate": "2026-08-11",
            "label": "Existing transaction",
            "amount": "12.00",
            "effect": "standard",
            "nature": "expense"
        })),
        StatusCode::CREATED,
    )
    .await;
    let state_paths = [
        transactions_path,
        format!("{account_path}/balances"),
        account_path.clone(),
        format!("{household_path}/summary"),
    ];
    let mut before = Vec::new();
    for path in &state_paths {
        before.push(send_json(&application, "GET", path, None, StatusCode::OK).await);
    }
    let csv = "Booking;Description;Amount\n\
        12/08/2026;Groceries;-12,3456\n\
        31/12/9999;;0\n\
        30/02/2026;Impossible day;13,00\n\
        13/08/2026;Missing amount\n\
        14/08/2026;Deposit;50,00";
    let request = preview_request(csv);
    let preview = send_json(
        &application,
        "POST",
        &format!("{account_path}/imports/csv/preview"),
        Some(request.clone()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        preview,
        json!({
            "accountId": account,
            "currency": "EUR",
            "rows": [
                {
                    "status": "valid",
                    "line": 2,
                    "bookingDate": "2026-08-12",
                    "label": "Groceries",
                    "signedAmount": "-12.3456"
                },
                {
                    "status": "rejected",
                    "line": 3,
                    "errors": [
                        {"code": "date_in_future", "field": "bookingDate"},
                        {"code": "invalid_length", "field": "label"},
                        {"code": "invalid_amount", "field": "amount"}
                    ]
                },
                {
                    "status": "rejected",
                    "line": 4,
                    "errors": [{"code": "invalid_date", "field": "bookingDate"}]
                },
                {
                    "status": "rejected",
                    "line": 5,
                    "errors": [{"code": "column_count_mismatch"}]
                },
                {
                    "status": "valid",
                    "line": 6,
                    "bookingDate": "2026-08-14",
                    "label": "Deposit",
                    "signedAmount": "50.00"
                }
            ]
        })
    );
    assert_eq!(
        send_json(
            &application,
            "POST",
            &format!("{account_path}/imports/csv/preview"),
            Some(request),
            StatusCode::OK,
        )
        .await,
        preview
    );
    for (path, before) in state_paths.iter().zip(before) {
        assert_eq!(
            send_json(&application, "GET", path, None, StatusCode::OK).await,
            before
        );
    }
}

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn csv_preview_returns_problems_for_bad_files_formats_and_targets() {
    let application = test_application().await;
    let household = create_household(&application).await;
    let account = create_account(&application, &household, "Statement format test account").await;
    let path = preview_path(&household, &account);

    let mut duplicate = preview_request("Booking;Description;Amount");
    duplicate["format"]["columns"]["amount"] = json!(" Description ");
    let mut empty = preview_request("Booking;Description;Amount");
    empty["format"]["columns"]["label"] = json!(" ");
    let mut invalid = preview_request("Booking;Description;Amount");
    invalid["format"]["delimiter"] = json!("unknown");
    let many_rows = format!(
        "Booking;Description;Amount\n{}",
        "13/08/2026;Deposit;1,00\n".repeat(5_001)
    );

    for (request, code, pointer) in [
        (
            duplicate,
            "duplicate_column_mapping",
            "#/format/columns/amount",
        ),
        (empty, "empty_column_mapping", "#/format/columns/label"),
        (invalid, "invalid_value", "#/format/delimiter"),
        (preview_request("\n\r\n"), "missing_csv_header", "#/csv"),
        (
            preview_request("Booking; Description;Description"),
            "invalid_csv_header",
            "#/csv",
        ),
        (
            preview_request("Booking;Description;Other"),
            "missing_csv_column",
            "#/format/columns/amount",
        ),
        (
            preview_request(
                "Booking;Description;Amount\n12/08/2026;Okay;1,00\n13/08/2026;\"Unclosed;2,00",
            ),
            "invalid_csv_syntax",
            "#/csv",
        ),
        (preview_request(&many_rows), "csv_row_limit", "#/csv"),
    ] {
        let error = send_json(
            &application,
            "POST",
            &path,
            Some(request),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
        assert_eq!(error["code"], "validation_error");
        assert_eq!(error["errors"][0]["code"], code);
        assert_eq!(error["errors"][0]["pointer"], pointer);
    }

    let too_large = preview_request(&"a".repeat(1024 * 1024 + 1));
    let error = send_json(
        &application,
        "POST",
        &path,
        Some(too_large),
        StatusCode::PAYLOAD_TOO_LARGE,
    )
    .await;
    assert_eq!(error["code"], "payload_too_large");

    let header_only = preview_request("Booking;Description;Amount");
    let result = send_json(
        &application,
        "POST",
        &path,
        Some(header_only.clone()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(result["rows"], json!([]));

    let other_household = create_household(&application).await;
    for (target, code) in [
        (preview_path(UNKNOWN_ID, &account), "household_not_found"),
        (
            preview_path(&other_household, &account),
            "account_not_found",
        ),
    ] {
        let error = send_json(
            &application,
            "POST",
            &target,
            Some(header_only.clone()),
            StatusCode::NOT_FOUND,
        )
        .await;
        assert_eq!(error["code"], code);
    }

    send_json(
        &application,
        "DELETE",
        &format!("/v1/households/{household}/accounts/{account}"),
        None,
        StatusCode::NO_CONTENT,
    )
    .await;
    let error = send_json(
        &application,
        "POST",
        &path,
        Some(header_only),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(error["code"], "account_not_found");
}

fn preview_path(household: &str, account: &str) -> String {
    format!("/v1/households/{household}/accounts/{account}/imports/csv/preview")
}

fn preview_request(csv: &str) -> Value {
    json!({
        "csv": csv,
        "format": {
            "delimiter": "semicolon",
            "dateFormat": "day_month_year",
            "decimalSeparator": "comma",
            "columns": {
                "bookingDate": "Booking",
                "label": "Description",
                "amount": "Amount"
            }
        }
    })
}

async fn test_application() -> Router {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must target an isolated test database");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("test database should be reachable");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("test migrations should succeed");
    app::build(pool)
}

async fn create_household(application: &Router) -> String {
    let household = send_json(
        application,
        "POST",
        "/v1/households",
        Some(json!({"name": "CSV test household", "baseCurrency": "EUR", "timezone": "Europe/Paris"})),
        StatusCode::CREATED,
    )
    .await;
    household["id"].as_str().unwrap().to_owned()
}

async fn create_account(application: &Router, household: &str, name: &str) -> String {
    let account = send_json(
        application,
        "POST",
        &format!("/v1/households/{household}/accounts"),
        Some(json!({
            "name": name,
            "kind": "checking",
            "currency": "EUR",
            "initialBalance": "0.00",
            "balanceDate": "2026-08-10"
        })),
        StatusCode::CREATED,
    )
    .await;
    account["id"].as_str().unwrap().to_owned()
}

async fn send_json(
    application: &Router,
    method: &str,
    path: &str,
    payload: Option<Value>,
    status: StatusCode,
) -> Value {
    let mut request = Request::builder().method(method).uri(path);
    let body = match payload {
        Some(payload) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(payload.to_string())
        }
        None => Body::empty(),
    };
    let response = application
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), status, "{method} {path}");
    if status.is_client_error() {
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
    }
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    if status == StatusCode::NO_CONTENT {
        assert!(bytes.is_empty());
        return Value::Null;
    }
    serde_json::from_slice(&bytes).unwrap()
}
