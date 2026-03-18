use actix_web::{HttpResponse, post, web};

use crate::infrastructure::http::dto::request::import_gurufocus_request::ImportGurufocusRequest;
use crate::infrastructure::http::dto::response::import_gurufocus_response::ImportGurufocusResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[post("/stocks/{isin}/gurufocus")]
pub async fn import_gurufocus(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<ImportGurufocusRequest>,
) -> Result<HttpResponse, ApiError> {
    let isin = path.into_inner();

    let stock = state.stock_service.get_stock_by_isin(isin).await?;
    let request = body.into_inner();
    let values = request.into_metric_values(stock.id);
    let rank_data = request.extract_rank_data();
    let imported = state.gurufocus_service.import_and_score(stock.id, values, rank_data).await?;

    Ok(HttpResponse::Ok().json(ImportGurufocusResponse { imported }))
}
