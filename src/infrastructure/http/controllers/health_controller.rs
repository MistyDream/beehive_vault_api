use actix_web::{HttpResponse, get, web};

use crate::infrastructure::http::state::AppState;

/// Probes must reflect real-time state, never a cached 200 from an
/// intermediary (RFC 9111 §5.2).
const NO_STORE: (&str, &str) = ("Cache-Control", "no-store");

/// Liveness probe: the process is up and responding. No external dependency.
#[get("/healthz")]
pub async fn healthz() -> HttpResponse {
    HttpResponse::Ok().insert_header(NO_STORE).finish()
}

/// Readiness probe: the container can receive traffic. Delegates to the
/// `HealthChecker` port (DB ping in prod). Returns 503 if the dependency
/// is unreachable so orchestrators stop sending traffic until recovery.
#[get("/readyz")]
pub async fn readyz(state: web::Data<AppState>) -> HttpResponse {
    match state.health_checker.readiness().await {
        Ok(()) => HttpResponse::Ok().insert_header(NO_STORE).finish(),
        Err(err) => {
            tracing::error!(error = %err, "readiness probe failed");
            HttpResponse::ServiceUnavailable().insert_header(NO_STORE).finish()
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz);
    cfg.service(readyz);
}
