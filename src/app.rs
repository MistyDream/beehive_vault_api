use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::{
    AppState,
    features::{accounts, health, households, institutions, net_worth},
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::liveness))
        .route("/readyz", get(health::readiness))
        .route("/v1/households", post(households::create))
        .route("/v1/households/{household_id}", get(households::get))
        .route(
            "/v1/households/{household_id}/institutions",
            post(institutions::create).get(institutions::list),
        )
        .route(
            "/v1/households/{household_id}/institutions/{institution_id}",
            patch(institutions::update).delete(institutions::archive),
        )
        .route(
            "/v1/households/{household_id}/accounts",
            post(accounts::create).get(accounts::list),
        )
        .route(
            "/v1/households/{household_id}/accounts/{account_id}",
            get(accounts::get)
                .patch(accounts::update)
                .delete(accounts::archive),
        )
        .route(
            "/v1/households/{household_id}/accounts/{account_id}/balances",
            post(accounts::create_balance).get(accounts::list_balances),
        )
        .route(
            "/v1/households/{household_id}/summary",
            get(net_worth::summary),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn healthz_reports_a_live_process() {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/beehive_vault")
            .expect("test database URL should be valid");
        let response = router(AppState { db })
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
