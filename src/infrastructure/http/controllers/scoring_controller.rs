use actix_web::{HttpResponse, get, web};
use uuid::Uuid;

use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[get("/portfolios/{id}/scoring")]
pub async fn get_portfolio_scoring(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let scoring = state.portfolio_scoring_service
        .get_scoring(path.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(scoring))
}
