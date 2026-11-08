use rust_decimal::Decimal;
use sqlx::AssertSqlSafe;
use uuid::Uuid;

use super::*;

const STATEMENT: &str = "Booking;Description;Amount\n\
    12/08/2026;Repeated purchase;12,3456\n\
    12/08/2026;Repeated purchase;12,3456\n\
    13/08/2026;Invalid amount;0\n\
    14/08/2026;Unselected deposit;42,00";

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn imports_selected_rows_preserves_provenance_and_keeps_bank_fields_immutable() {
    let pool = test_pool().await;
    let application = app::build(pool.clone());
    let household = create_household(&application).await;
    let account = create_account(&application, &household, "CSV import checking account").await;
    let path = confirm_path(&household, &account);
    let payload = confirmation(STATEMENT, json!([selection(3), selection(2)]));

    let imported = send_json(
        &application,
        "POST",
        &path,
        Some(payload.clone()),
        StatusCode::OK,
    )
    .await;
    assert_eq!(imported["accountId"], account);
    let rows = imported["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["line"], 3);
    assert_eq!(rows[1]["line"], 2);
    assert_eq!(rows[0]["status"], "imported");
    assert_eq!(rows[1]["status"], "imported");
    let transaction = rows[0]["transactionId"].as_str().unwrap();
    assert_ne!(rows[1]["transactionId"], transaction);

    let persisted: Vec<(i32, String, Decimal, Decimal, i32)> = sqlx::query_as(
        "SELECT csv_source_line, csv_file_name, csv_signed_amount, amount,          octet_length(csv_statement_digest) FROM transactions WHERE account_id = $1          ORDER BY csv_source_line",
    )
    .bind(Uuid::parse_str(&account).unwrap())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(persisted.len(), 2);
    for (index, (line, file, signed, amount, digest_length)) in persisted.into_iter().enumerate() {
        assert_eq!(line, (index + 2) as i32);
        assert_eq!(file, "statement.csv");
        assert_eq!(signed.to_string(), "12.3456");
        assert_eq!(amount.to_string(), "-12.3456");
        assert_eq!(digest_length, 32);
    }

    let household_path = format!("/v1/households/{household}");
    let transaction_path = format!("{household_path}/transactions/{transaction}");
    let current = send_json(&application, "GET", &transaction_path, None, StatusCode::OK).await;
    assert_eq!(current["origin"], "import");
    assert_eq!(current["category"], Value::Null);
    assert_eq!(current["effect"], "standard");
    assert_eq!(current["amount"], "12.3456");
    let categories = send_json(
        &application,
        "GET",
        &format!("{household_path}/categories?kind=income"),
        None,
        StatusCode::OK,
    )
    .await;
    let category = &categories[0]["id"];
    let recategorized = send_json(
        &application,
        "PATCH",
        &transaction_path,
        Some(json!({"nature": "income", "categoryId": category, "note": "Reviewed"})),
        StatusCode::OK,
    )
    .await;
    assert_eq!(&recategorized["category"]["id"], category);
    assert_eq!(recategorized["effect"], "reversal");
    assert_eq!(recategorized["accountAmount"], current["accountAmount"]);
    let problem = send_json(
        &application,
        "PATCH",
        &transaction_path,
        Some(json!({"effect": "standard"})),
        StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(problem["code"], "imported_transaction_fields_immutable");

    send_json(
        &application,
        "DELETE",
        &transaction_path,
        None,
        StatusCode::NO_CONTENT,
    )
    .await;
    let mut retry = payload;
    retry["fileName"] = json!("renamed.csv");
    retry["rows"] = json!([selection(2), {"line": 3, "nature": "income", "effect": "standard"}]);
    let duplicates = send_json(&application, "POST", &path, Some(retry), StatusCode::OK).await;
    assert_eq!(
        duplicates["rows"],
        json!([
            {"line": 2, "status": "duplicate", "transactionId": rows[1]["transactionId"]},
            {"line": 3, "status": "duplicate", "transactionId": transaction}
        ])
    );
    send_json(
        &application,
        "GET",
        &transaction_path,
        None,
        StatusCode::NOT_FOUND,
    )
    .await;

    let raw_record = "14/08/2026;\"First line; note\r\nSecond line \"\"quoted\"\"\";+3,0001";
    let overlapping = format!(
        "Booking;Description;Amount\r\n12/08/2026;Repeated purchase;12,3456\r\n{raw_record}\r\n"
    );
    let response = send_json(
        &application,
        "POST",
        &path,
        Some(confirmation(
            &overlapping,
            json!([selection(2), selection(3)]),
        )),
        StatusCode::OK,
    )
    .await;
    assert_eq!(response["rows"][0]["status"], "imported");
    assert_eq!(response["rows"][1]["status"], "imported");
    let id = response["rows"][1]["transactionId"].as_str().unwrap();
    let raw: String =
        sqlx::query_scalar("SELECT csv_source_record FROM transactions WHERE id = $1")
            .bind(Uuid::parse_str(id).unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(raw, raw_record);
}

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn retries_concurrently_and_keeps_deduplication_scoped_to_the_account() {
    let application = test_application().await;
    let household = create_household(&application).await;
    let account = create_account(&application, &household, "First import account").await;
    let other_account = send_json(
        &application,
        "POST",
        &format!("/v1/households/{household}/accounts"),
        Some(json!({
            "name": "Second import account",
            "kind": "credit_card",
            "currency": "EUR",
            "initialBalance": "0.00",
            "balanceDate": "2026-08-10"
        })),
        StatusCode::CREATED,
    )
    .await;
    let other_account = other_account["id"].as_str().unwrap();
    let path = confirm_path(&household, &account);
    let request = confirmation(STATEMENT, json!([selection(2)]));
    let (left, right) = tokio::join!(
        send_json(
            &application,
            "POST",
            &path,
            Some(request.clone()),
            StatusCode::OK
        ),
        send_json(
            &application,
            "POST",
            &path,
            Some(request.clone()),
            StatusCode::OK
        )
    );
    let left = &left["rows"][0];
    let right = &right["rows"][0];
    let mut statuses = [
        left["status"].as_str().unwrap(),
        right["status"].as_str().unwrap(),
    ];
    statuses.sort_unstable();
    assert_eq!(statuses, ["duplicate", "imported"]);
    assert_eq!(left["transactionId"], right["transactionId"]);

    let other = send_json(
        &application,
        "POST",
        &confirm_path(&household, other_account),
        Some(request),
        StatusCode::OK,
    )
    .await;
    let other = &other["rows"][0];
    assert_eq!(other["status"], "imported");
    assert_ne!(other["transactionId"], left["transactionId"]);
    let id = other["transactionId"].as_str().unwrap();
    let transaction = send_json(
        &application,
        "GET",
        &format!("/v1/households/{household}/transactions/{id}"),
        None,
        StatusCode::OK,
    )
    .await;
    assert_eq!(transaction["economicAmount"], "-12.3456");
    assert_eq!(transaction["accountAmount"], "12.3456");
}

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn validates_all_selected_rows_and_rolls_back_a_failed_write() {
    let pool = test_pool().await;
    let application = app::build(pool.clone());
    let household = create_household(&application).await;
    let account = create_account(&application, &household, "CSV rollback test account").await;
    let path = confirm_path(&household, &account);
    let transactions_path = format!("/v1/households/{household}/transactions");

    let mut invalid_name = confirmation(STATEMENT, json!([selection(2)]));
    invalid_name["fileName"] = json!(" ");
    for (request, code, pointer) in [
        (invalid_name, "invalid_file_name", "#/fileName"),
        (
            confirmation(STATEMENT, json!([])),
            "invalid_selection_size",
            "#/rows",
        ),
        (
            confirmation(STATEMENT, json!([selection(2), selection(2)])),
            "duplicate_csv_selection",
            "#/rows/1/line",
        ),
        (
            confirmation(STATEMENT, json!([selection(2), selection(9)])),
            "csv_line_not_found",
            "#/rows/1/line",
        ),
        (
            confirmation(STATEMENT, json!([selection(2), selection(4)])),
            "rejected_csv_line",
            "#/rows/1/line",
        ),
        (
            confirmation(
                STATEMENT,
                json!([{"line": 2, "nature": "transfer", "effect": "standard"}]),
            ),
            "invalid_value",
            "#/rows/0/nature",
        ),
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
    let request = confirmation(STATEMENT, json!([selection(2)]));
    let other_household = create_household(&application).await;
    for (target, code) in [
        (confirm_path(UNKNOWN_ID, &account), "household_not_found"),
        (
            confirm_path(&other_household, &account),
            "account_not_found",
        ),
        (confirm_path(&household, UNKNOWN_ID), "account_not_found"),
    ] {
        let error = send_json(
            &application,
            "POST",
            &target,
            Some(request.clone()),
            StatusCode::NOT_FOUND,
        )
        .await;
        assert_eq!(error["code"], code);
    }
    let rows = send_json(
        &application,
        "GET",
        &transactions_path,
        None,
        StatusCode::OK,
    )
    .await;
    assert_eq!(rows["total"], 0);

    // Both DDL substitutions are a fixed prefix followed by UUID hex digits.
    let constraint = format!(
        "test_reject_{}",
        Uuid::parse_str(&account).unwrap().simple()
    );
    sqlx::query(AssertSqlSafe(format!(
        "ALTER TABLE transactions ADD CONSTRAINT {constraint} CHECK (label <> '{constraint}')"
    )))
    .execute(&pool)
    .await
    .unwrap();
    let statement = format!(
        "Booking;Description;Amount\n12/08/2026;Valid row;12,00\n13/08/2026;{constraint};13,00"
    );
    let payload = confirmation(&statement, json!([selection(2), selection(3)]));
    let result = application
        .clone()
        .oneshot(
            Request::post(&path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    sqlx::query(AssertSqlSafe(format!(
        "ALTER TABLE transactions DROP CONSTRAINT {constraint}"
    )))
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(result.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let rows = send_json(
        &application,
        "GET",
        &transactions_path,
        None,
        StatusCode::OK,
    )
    .await;
    assert_eq!(rows["total"], 0);

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
        Some(payload),
        StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(error["code"], "account_not_found");
}

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn confirms_the_full_selection_after_previewing_a_maximum_size_csv() {
    const CSV_BYTES: usize = 1024 * 1024;
    const ROWS: usize = 5_000;
    let header = "Booking;Description;Amount;Ignored\n";
    let record = "12/08/2026;Groceries;-12,3456;";
    // Tabs in an ignored column are valid CSV and require escaping in the JSON body.
    let padding = CSV_BYTES - header.len() - ROWS * (record.len() + 1);
    let tabs = "\t".repeat(padding / ROWS);
    let final_tabs = "\t".repeat(padding % ROWS);
    let mut csv = String::with_capacity(CSV_BYTES);
    csv.push_str(header);
    for index in 0..ROWS {
        csv.push_str(record);
        csv.push_str(&tabs);
        if index + 1 == ROWS {
            csv.push_str(&final_tabs);
        }
        csv.push('\n');
    }
    assert_eq!(csv.len(), CSV_BYTES);

    let pool = test_pool().await;
    let application = app::build(pool.clone());
    let household = create_household(&application).await;
    let account = create_account(&application, &household, "Full statement account").await;
    let preview = preview_request(&csv);
    assert!(preview.to_string().len() < 2 * CSV_BYTES);
    let preview = send_json(
        &application,
        "POST",
        &preview_path(&household, &account),
        Some(preview),
        StatusCode::OK,
    )
    .await;
    let preview_rows = preview["rows"].as_array().unwrap();
    assert_eq!(preview_rows.len(), ROWS);
    assert!(preview_rows.iter().all(|row| row["status"] == "valid"));

    let rows = preview_rows
        .iter()
        .map(|row| selection(i32::try_from(row["line"].as_u64().unwrap()).unwrap()))
        .collect();
    let confirmation = confirmation(&csv, Value::Array(rows));
    assert!(confirmation.to_string().len() > 2 * CSV_BYTES);
    let imported = send_json(
        &application,
        "POST",
        &confirm_path(&household, &account),
        Some(confirmation),
        StatusCode::OK,
    )
    .await;
    let imported_rows = imported["rows"].as_array().unwrap();
    assert_eq!(imported_rows.len(), ROWS);
    assert!(imported_rows.iter().all(|row| row["status"] == "imported"));

    let stored: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM transactions WHERE account_id = $1 AND source = 'import'",
    )
    .bind(Uuid::parse_str(&account).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored, ROWS as i64);
}

#[tokio::test]
async fn keeps_payload_limits_local_and_reports_problem_details() {
    const MIB: usize = 1024 * 1024;
    let application = app::build(
        PgPoolOptions::new()
            .connect_lazy("postgres://unused@127.0.0.1/unused")
            .unwrap(),
    );
    for (path, limit) in [
        (preview_path(UNKNOWN_ID, UNKNOWN_ID), 2 * MIB),
        (confirm_path(UNKNOWN_ID, UNKNOWN_ID), 2 * MIB + MIB / 2),
        ("/v1/households".to_owned(), 2 * MIB),
    ] {
        let problem = send_json(
            &application,
            "POST",
            &path,
            Some(json!({"padding": "x".repeat(limit)})),
            StatusCode::PAYLOAD_TOO_LARGE,
        )
        .await;
        assert_eq!(problem["code"], "payload_too_large");
        assert_eq!(
            problem["detail"],
            "The request body exceeds the supported size."
        );
    }

    let problem = send_json(
        &application,
        "POST",
        &confirm_path(UNKNOWN_ID, UNKNOWN_ID),
        Some(confirmation(&"x".repeat(MIB + 1), json!([selection(2)]))),
        StatusCode::PAYLOAD_TOO_LARGE,
    )
    .await;
    assert_eq!(problem["code"], "payload_too_large");
    assert_eq!(
        problem["detail"],
        "The CSV text exceeds the supported size of 1 MiB."
    );
}

fn confirm_path(household: &str, account: &str) -> String {
    format!("/v1/households/{household}/accounts/{account}/imports/csv/confirm")
}

fn selection(line: i32) -> Value {
    json!({"line": line, "nature": "expense", "effect": "standard"})
}

fn confirmation(csv: &str, rows: Value) -> Value {
    let mut request = preview_request(csv);
    request["fileName"] = json!("statement.csv");
    request["rows"] = rows;
    request
}
