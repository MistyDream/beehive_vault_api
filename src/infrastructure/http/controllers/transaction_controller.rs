use actix_web::{HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::Json;

use crate::application::services::pagination::Paginated;
use crate::application::services::transaction_service::TransactionsQuery;
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
    let (transaction, stocks) = state.transaction_service.create(portfolio_id, new).await?;

    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/portfolios/{}/transactions/{}", portfolio_id, transaction.id)))
        .json(TransactionResponse::from_transaction(transaction, &stocks)))
}

#[get("/portfolios/{id}/transactions")]
pub async fn list_transactions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: web::Query<TransactionQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let q = query.into_inner();

    let tx_query = TransactionsQuery {
        filters: TransactionFilter {
            transaction_type: q.transaction_type,
            stock_id: q.stock_id,
            from_date: q.from_date,
            to_date: q.to_date,
        },
        sort_by: q.sort_by,
        sort_dir: q.sort_dir,
        page: q.page,
        limit: q.limit,
    };

    let (paginated, stocks) = state.transaction_service.list_paginated(portfolio_id, tx_query).await?;
    let items: Vec<TransactionResponse> = paginated
        .items
        .into_iter()
        .map(|t| TransactionResponse::from_transaction(t, &stocks))
        .collect();

    Ok(HttpResponse::Ok().json(Paginated { items, total: paginated.total }))
}

#[get("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn get_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let (transaction, stocks) = state.transaction_service.get(portfolio_id, tx_id).await?;
    Ok(HttpResponse::Ok().json(TransactionResponse::from_transaction(transaction, &stocks)))
}

#[put("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn update_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
    body: Json<UpdateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let data = body.into_inner().into_update_transaction(portfolio_id);
    let (transaction, stocks) = state.transaction_service.update(portfolio_id, tx_id, data).await?;
    Ok(HttpResponse::Ok().json(TransactionResponse::from_transaction(transaction, &stocks)))
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
