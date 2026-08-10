use axum::{Router, routing::get};

use crate::{AppState, features::health};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::liveness))
        .route("/readyz", get(health::readiness))
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
