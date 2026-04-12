use actix_web::{HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::Json;

use crate::domain::wallet::transaction::TransactionFilter;
use crate::infrastructure::http::dto::request::transaction_request::{
    CreateTransactionRequest, TransactionQueryParams, UpdateTransactionRequest,
};
use crate::infrastructure::http::dto::response::transaction_response::TransactionResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[post("/portfolios/{id}/transactions")]
pub async fn create_transaction(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: Json<CreateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let new = body.into_inner().into_new_transaction(portfolio_id);
    let transaction = state.transaction_service.create(portfolio_id, new).await?;

    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/portfolios/{}/transactions/{}", portfolio_id, transaction.id)))
        .json(TransactionResponse::from(transaction)))
}

#[get("/portfolios/{id}/transactions")]
pub async fn list_transactions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: web::Query<TransactionQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();

    let has_filters = query.transaction_type.is_some()
        || query.stock_id.is_some()
        || query.from_date.is_some()
        || query.to_date.is_some();

    let filters = if has_filters {
        Some(TransactionFilter {
            transaction_type: query.transaction_type.clone(),
            stock_id: query.stock_id,
            from_date: query.from_date,
            to_date: query.to_date,
        })
    } else {
        None
    };

    let transactions = state.transaction_service.list(portfolio_id, filters).await?;
    let response: Vec<TransactionResponse> = transactions.into_iter().map(TransactionResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

#[get("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn get_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let transaction = state.transaction_service.get(portfolio_id, tx_id).await?;
    Ok(HttpResponse::Ok().json(TransactionResponse::from(transaction)))
}

#[put("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn update_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
    body: Json<UpdateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let data = body.into_inner().into_update_transaction(portfolio_id);
    let transaction = state.transaction_service.update(portfolio_id, tx_id, data).await?;
    Ok(HttpResponse::Ok().json(TransactionResponse::from(transaction)))
}

#[delete("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn delete_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    state.transaction_service.delete(portfolio_id, tx_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
