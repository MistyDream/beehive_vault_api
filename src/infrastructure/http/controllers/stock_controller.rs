use actix_web::{HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::Json;

use crate::domain::market::stock::{NewStock, StockFilter, UpdateStock};
use crate::infrastructure::http::dto::request::list_stocks_query::ListStocksQuery;
use crate::infrastructure::http::dto::request::stock_request::{CreateStockRequest, UpdateStockRequest};
use crate::infrastructure::http::dto::response::paginated_response::PaginatedResponse;
use crate::infrastructure::http::dto::response::stock_response::StockResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[post("/stocks")]
pub async fn create_stock(
    state: web::Data<AppState>,
    body: Json<CreateStockRequest>,
) -> Result<HttpResponse, ApiError> {
    let new_stock = NewStock::from(body.into_inner());

    let stock = state.stock_service.create_stock(new_stock).await?;

    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/stocks/{}", stock.isin)))
        .json(StockResponse::from(stock)))
}

#[get("/stocks")]
pub async fn list_stocks(
    state: web::Data<AppState>,
    query: web::Query<ListStocksQuery>,
) -> Result<HttpResponse, ApiError> {
    let filter = StockFilter::from(query.into_inner());

    let result = state.stock_service.search_stocks(filter).await?;

    Ok(HttpResponse::Ok().json(PaginatedResponse::<StockResponse>::from(result)))
}

#[put("/stocks/{isin}")]
pub async fn update_stock(
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: Json<UpdateStockRequest>,
) -> Result<HttpResponse, ApiError> {
    let isin = path.into_inner();
    let data = UpdateStock::from(body.into_inner());

    let stock = state.stock_service.update_stock(isin, data).await?;

    Ok(HttpResponse::Ok().json(StockResponse::from(stock)))
}

#[delete("/stocks/{isin}")]
pub async fn delete_stock(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let isin = path.into_inner();

    state.stock_service.delete_stock(isin).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/stocks/{isin}")]
pub async fn get_stock(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let isin = path.into_inner();

    let stock = state.stock_service.get_stock_by_isin(isin).await?;

    Ok(HttpResponse::Ok().json(StockResponse::from(stock)))
}
