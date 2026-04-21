use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::{Json, Query};

use crate::application::error::AppError;
use crate::application::services::pagination::SortDirection;
use crate::application::services::transaction_service::{TransactionSort, TransactionsQuery};
use crate::domain::wallet::enums::TransactionType;
use crate::domain::wallet::transaction::TransactionFilter;
use crate::infrastructure::http::dto::request::transaction_request::{
    CreateTransactionRequest, TransactionQueryParams, TransactionStatsQueryParams,
    UpdateTransactionRequest,
};
use crate::infrastructure::http::dto::response::paginated_response::PaginatedResponse;
use crate::infrastructure::http::dto::response::transaction_response::{
    TransactionResponse, TransactionStatsResponse,
};
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::etag::respond_with_etag;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::http::PRIVATE_SHORT_CACHE;

#[post("/portfolios/{portfolio_id}/transactions")]
pub async fn create_transaction(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: Json<CreateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let new = body.into_inner().into_new_transaction(portfolio_id)?;
    let (transaction, stocks) = state.transaction_service.create(portfolio_id, new).await?;

    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/v1/portfolios/{}/transactions/{}", portfolio_id, transaction.id)))
        .json(TransactionResponse::from_transaction(transaction, &stocks)))
}

#[get("/portfolios/{portfolio_id}/transactions")]
pub async fn list_transactions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: Query<TransactionQueryParams>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let q = query.into_inner();

    let transaction_types: Vec<TransactionType> = q
        .transaction_types
        .as_deref()
        .map(|csv| {
            csv.split(',')
                .filter(|s| !s.is_empty())
                .map(TransactionType::try_from)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()
        .map_err(|msg| AppError::BadRequest(format!("invalid transaction_types: {msg}")))?
        .unwrap_or_default();

    let tx_query = TransactionsQuery {
        filters: TransactionFilter {
            transaction_types,
            stock_id: q.stock_id,
            from_date: q.from_date,
            to_date: q.to_date,
        },
        sort_by: TransactionSort::parse(q.sort_by.as_deref()),
        sort_dir: SortDirection::parse(q.sort_dir.as_deref()),
        page: q.page,
        limit: q.limit,
    };

    let (page, stocks) = state.transaction_service.list_paginated(portfolio_id, tx_query).await?;
    let response: PaginatedResponse<TransactionResponse> = PaginatedResponse::from(page)
        .map(|t| TransactionResponse::from_transaction(t, &stocks));

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", PRIVATE_SHORT_CACHE))
        .json(response))
}

#[get("/portfolios/{portfolio_id}/transactions/stats")]
pub async fn get_transactions_stats(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: Query<TransactionStatsQueryParams>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let q = query.into_inner();
    let stats = state
        .transaction_service
        .stats(portfolio_id, q.stock_id, q.from_date, q.to_date)
        .await?;
    respond_with_etag(&request, &TransactionStatsResponse::from(stats), PRIVATE_SHORT_CACHE)
}

#[get("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn get_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
    request: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let (transaction, stocks) = state.transaction_service.get(portfolio_id, tx_id).await?;
    let response = TransactionResponse::from_transaction(transaction, &stocks);
    respond_with_etag(&request, &response, PRIVATE_SHORT_CACHE)
}

#[put("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn update_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
    body: Json<UpdateTransactionRequest>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let data = body.into_inner().into_update_transaction(portfolio_id)?;
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
