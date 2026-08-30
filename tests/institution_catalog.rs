use std::env;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use beehive_vault_api::{
    app,
    database::Database,
    features::institutions::admin::{AdminError, InstitutionAdminService, InstitutionCatalog},
};
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires the local PostgreSQL test database"]
async fn household_institutions_migrate_to_the_global_catalog() {
    dotenvy::dotenv().ok();
    let database_url = env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must target an isolated test database");
    let schema = format!("institution_catalog_{}", Uuid::now_v7().simple());
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

    sqlx::raw_sql(include_str!(
        "../migrations/202608100001_create_financial_foundation.sql"
    ))
    .execute(&db)
    .await
    .expect("financial foundation migration should succeed");

    let first_household_id = Uuid::now_v7();
    let second_household_id = Uuid::now_v7();
    for (household_id, name) in [
        (first_household_id, "First household"),
        (second_household_id, "Second household"),
    ] {
        sqlx::query(
            "INSERT INTO households (id, name, base_currency, timezone) \
             VALUES ($1, $2, 'EUR', 'Europe/Paris')",
        )
        .bind(household_id)
        .bind(name)
        .execute(&db)
        .await
        .expect("test household should be created");
    }

    let canonical_institution_id = Uuid::now_v7();
    let duplicate_institution_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO institutions (id, household_id, name, created_at) \
         VALUES ($1, $2, ' Example Bank ', '2026-01-01T00:00:00Z')",
    )
    .bind(canonical_institution_id)
    .bind(first_household_id)
    .execute(&db)
    .await
    .expect("first household institution should be created");
    sqlx::query(
        "INSERT INTO institutions (id, household_id, name, created_at) \
         VALUES ($1, $2, 'example bank', '2026-01-02T00:00:00Z')",
    )
    .bind(duplicate_institution_id)
    .bind(second_household_id)
    .execute(&db)
    .await
    .expect("duplicate household institution should be created");

    for (household_id, institution_id, name) in [
        (
            first_household_id,
            canonical_institution_id,
            "First account",
        ),
        (
            second_household_id,
            duplicate_institution_id,
            "Second account",
        ),
    ] {
        sqlx::query(
            "INSERT INTO accounts (id, household_id, institution_id, name, kind, currency) \
             VALUES ($1, $2, $3, $4, 'checking', 'EUR')",
        )
        .bind(Uuid::now_v7())
        .bind(household_id)
        .bind(institution_id)
        .bind(name)
        .execute(&db)
        .await
        .expect("test account should be created");
    }

    sqlx::raw_sql(include_str!(
        "../migrations/202608120001_create_financial_transactions.sql"
    ))
    .execute(&db)
    .await
    .expect("financial transactions migration should succeed");
    sqlx::raw_sql(include_str!(
        "../migrations/202608230001_global_financial_institutions.sql"
    ))
    .execute(&db)
    .await
    .expect("global institution migration should succeed");

    let migrated_institution_ids =
        sqlx::query_scalar::<_, Uuid>("SELECT institution_id FROM accounts ORDER BY name")
            .fetch_all(&db)
            .await
            .expect("migrated account references should be readable");
    assert_eq!(
        migrated_institution_ids,
        vec![canonical_institution_id, canonical_institution_id]
    );

    let response = app::build(db.clone())
        .oneshot(
            Request::get("/v1/institutions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let institutions: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        institutions,
        json!([{ "id": canonical_institution_id, "name": "Example Bank" }])
    );

    let admin_service = InstitutionAdminService::new(Database::new(db.clone()));
    let renamed = admin_service
        .rename(
            canonical_institution_id.to_string().parse().unwrap(),
            " Canonical Bank ".into(),
        )
        .await
        .expect("institution should be renamed");
    assert_eq!(renamed.id.to_string(), canonical_institution_id.to_string());
    assert_eq!(renamed.name, "Canonical Bank");

    let account_institution_ids =
        sqlx::query_scalar::<_, Uuid>("SELECT institution_id FROM accounts ORDER BY name")
            .fetch_all(&db)
            .await
            .expect("account references should survive an institution rename");
    assert_eq!(
        account_institution_ids,
        vec![canonical_institution_id, canonical_institution_id]
    );

    let duplicate = admin_service.add("canonical bank".into()).await;
    assert!(matches!(
        duplicate,
        Err(AdminError::DuplicateInstitutionName)
    ));

    let catalog = InstitutionCatalog::from_json(
        r#"{"institutions":[{"name":"Canonical Bank"},{"name":"Another Bank"}]}"#,
    )
    .unwrap();
    let first_import = admin_service
        .import_catalog(catalog)
        .await
        .expect("catalog should be imported");
    assert_eq!(first_import.added, 1);
    assert_eq!(first_import.unchanged, 1);

    let second_import = admin_service
        .import_catalog(
            InstitutionCatalog::from_json(
                r#"{"institutions":[{"name":"Canonical Bank"},{"name":"Another Bank"}]}"#,
            )
            .unwrap(),
        )
        .await
        .expect("catalog import should be idempotent");
    assert_eq!(second_import.added, 0);
    assert_eq!(second_import.unchanged, 2);

    let listed_names = admin_service
        .list()
        .await
        .expect("institutions should be listed")
        .into_iter()
        .map(|institution| institution.name)
        .collect::<Vec<_>>();
    assert_eq!(listed_names, vec!["Another Bank", "Canonical Bank"]);

    db.close().await;
    // The identifier is generated exclusively from a fixed prefix and UUID hexadecimal digits.
    sqlx::query(AssertSqlSafe(format!("DROP SCHEMA \"{schema}\" CASCADE")))
        .execute(&admin)
        .await
        .expect("isolated test schema should be removed");
}
