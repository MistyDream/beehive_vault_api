use actix_web::{HttpResponse, get, web};

use crate::infrastructure::http::state::AppState;

/// Liveness probe: the process is up and responding. No external dependency.
#[get("/healthz")]
pub async fn healthz() -> HttpResponse {
    HttpResponse::Ok().finish()
}

/// Readiness probe: the container can receive traffic. Delegates to the
/// `HealthChecker` port (DB ping in prod). Returns 503 if the dependency
/// is unreachable so orchestrators stop sending traffic until recovery.
#[get("/readyz")]
pub async fn readyz(state: web::Data<AppState>) -> HttpResponse {
    match state.health_checker.readiness().await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(err) => {
            tracing::error!(error = %err, "readiness probe failed");
            HttpResponse::ServiceUnavailable().finish()
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz);
    cfg.service(readyz);
}
