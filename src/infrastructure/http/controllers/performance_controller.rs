use actix_web::{HttpResponse, get, web};

use crate::infrastructure::http::dto::request::transaction_request::PerformanceQueryParams;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[get("/portfolios/{id}/performance")]
pub async fn get_performance(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: web::Query<PerformanceQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let report = state.position_service
        .get_performance(path.into_inner(), query.from_date, query.to_date)
        .await?;
    Ok(HttpResponse::Ok().json(report))
}
