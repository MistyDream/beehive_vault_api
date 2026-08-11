use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use super::HealthModule;

#[derive(Serialize)]
pub(super) struct HealthResponse {
    status: &'static str,
}

pub(super) async fn liveness() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub(super) async fn readiness(State(module): State<HealthModule>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(module.database.pool())
        .await
    {
        Ok(_) => (StatusCode::OK, Json(HealthResponse { status: "ready" })),
        Err(error) => {
            tracing::error!(%error, "database readiness check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                }),
            )
        }
    }
}
