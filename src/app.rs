use axum::Router;
use sqlx::PgPool;

use crate::{
    database::Database,
    error::{ApiError, ProblemKind},
    features::{
        accounts, categories, health, households, institutions, monthly_flows, net_worth,
        transactions, transfers,
    },
};

pub fn build(pool: PgPool) -> Router {
    let database = Database::new(pool);
    let accounts = accounts::configure(database.clone());
    let categories = categories::configure(database.clone());
    let health = health::configure(database.clone());
    let households = households::configure(database.clone());
    let institutions = institutions::configure(database.clone());
    let monthly_flows = monthly_flows::configure(database.clone());
    let net_worth = net_worth::configure(database.clone());
    let transactions = transactions::configure(database.clone());
    let transfers = transfers::configure(database);

    let api = Router::new()
        .merge(accounts::routes(accounts))
        .merge(categories::routes(categories))
        .merge(households::routes(households))
        .merge(institutions::routes(institutions))
        .merge(monthly_flows::routes(monthly_flows))
        .merge(net_worth::routes(net_worth))
        .merge(transactions::routes(transactions))
        .merge(transfers::routes(transfers));

    Router::new()
        .merge(health::routes(health))
        .nest("/v1", api)
        .fallback(route_not_found)
        .method_not_allowed_fallback(method_not_allowed)
}

async fn route_not_found() -> ApiError {
    ApiError::new(ProblemKind::RouteNotFound).with_detail("The requested route does not exist.")
}

async fn method_not_allowed() -> ApiError {
    ApiError::new(ProblemKind::MethodNotAllowed)
        .with_detail("The requested method is not supported for this route.")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::Value;
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn healthz_reports_a_live_process() {
        let response = test_application()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn malformed_json_uses_problem_details() {
        let (status, content_type, problem) =
            send_request("POST", "/v1/households", Some("application/json"), "{").await;

        assert_problem(
            status,
            &content_type,
            &problem,
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "urn:beehive-vault:problem:invalid-request",
        );
    }

    #[tokio::test]
    async fn invalid_json_data_identifies_the_body_field() {
        let (status, content_type, problem) = send_request(
            "POST",
            "/v1/households",
            Some("application/json"),
            r#"{"name":"Home","baseCurrency":"EU","timezone":"Europe/Paris"}"#,
        )
        .await;

        assert_problem(
            status,
            &content_type,
            &problem,
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "urn:beehive-vault:problem:validation-error",
        );
        assert_eq!(problem["errors"][0]["location"], "body");
        assert_eq!(problem["errors"][0]["pointer"], "#/baseCurrency");
        assert_eq!(problem["errors"][0]["code"], "invalid_value");
    }

    #[tokio::test]
    async fn missing_json_content_type_uses_problem_details() {
        let (status, content_type, problem) = send_request(
            "POST",
            "/v1/households",
            None,
            r#"{"name":"Home","baseCurrency":"EUR","timezone":"Europe/Paris"}"#,
        )
        .await;

        assert_problem(
            status,
            &content_type,
            &problem,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "urn:beehive-vault:problem:unsupported-media-type",
        );
    }

    #[tokio::test]
    async fn invalid_path_and_query_parameters_use_problem_details() {
        let (path_status, path_content_type, path_problem) =
            send_request("GET", "/v1/households/not-a-uuid", None, "").await;
        assert_problem(
            path_status,
            &path_content_type,
            &path_problem,
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "urn:beehive-vault:problem:invalid-request",
        );
        assert_eq!(path_problem["errors"][0]["location"], "path");

        let (query_status, query_content_type, query_problem) = send_request(
            "GET",
            "/v1/households/00000000-0000-0000-0000-000000000000/categories?kind=unknown",
            None,
            "",
        )
        .await;
        assert_problem(
            query_status,
            &query_content_type,
            &query_problem,
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
            "urn:beehive-vault:problem:validation-error",
        );
        assert_eq!(query_problem["errors"][0]["location"], "query");
        assert_eq!(query_problem["errors"][0]["pointer"], "#/kind");
    }

    #[tokio::test]
    async fn router_fallbacks_use_problem_details() {
        let (not_found_status, not_found_content_type, not_found_problem) =
            send_request("GET", "/v1/unknown", None, "").await;
        assert_problem(
            not_found_status,
            &not_found_content_type,
            &not_found_problem,
            StatusCode::NOT_FOUND,
            "route_not_found",
            "urn:beehive-vault:problem:route-not-found",
        );

        let (legacy_status, legacy_content_type, legacy_problem) = send_request(
            "GET",
            "/v1/households/00000000-0000-0000-0000-000000000000/institutions",
            None,
            "",
        )
        .await;
        assert_problem(
            legacy_status,
            &legacy_content_type,
            &legacy_problem,
            StatusCode::NOT_FOUND,
            "route_not_found",
            "urn:beehive-vault:problem:route-not-found",
        );

        let (method_status, method_content_type, method_problem) =
            send_request("DELETE", "/v1/households", None, "").await;
        assert_problem(
            method_status,
            &method_content_type,
            &method_problem,
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "urn:beehive-vault:problem:method-not-allowed",
        );
    }

    fn test_application() -> Router {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/beehive_vault")
            .expect("test database URL should be valid");
        build(db)
    }

    async fn send_request(
        method: &str,
        uri: &str,
        content_type: Option<&str>,
        body: &str,
    ) -> (StatusCode, String, Value) {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(content_type) = content_type {
            request = request.header(header::CONTENT_TYPE, content_type);
        }
        let response = test_application()
            .oneshot(request.body(Body::from(body.to_owned())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let body = serde_json::from_slice(&bytes).unwrap();
        (status, content_type, body)
    }

    fn assert_problem(
        status: StatusCode,
        content_type: &str,
        problem: &Value,
        expected_status: StatusCode,
        expected_code: &str,
        expected_type: &str,
    ) {
        assert_eq!(status, expected_status);
        assert_eq!(content_type, "application/problem+json");
        assert_eq!(problem["status"], expected_status.as_u16());
        assert_eq!(problem["code"], expected_code);
        assert_eq!(problem["type"], expected_type);
        assert!(problem["title"].is_string());
    }
}
