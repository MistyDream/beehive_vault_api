use actix_web::{HttpRequest, HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::{Json, Query};

use crate::application::services::transaction_service::TransactionsQuery;
use crate::domain::wallet::transaction::TransactionFilter;
use crate::infrastructure::http::dto::request::transaction_request::{
    CreateTransactionRequest, TransactionQueryParams, UpdateTransactionRequest,
};
use crate::infrastructure::http::dto::response::paginated_response::{build_link_header, PaginatedResponse};
use crate::infrastructure::http::dto::response::transaction_response::TransactionResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

const CACHE_CONTROL: &str = "private, max-age=30";

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
    query: Query<TransactionQueryParams>,
    request: HttpRequest,
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

    let (page, stocks) = state.transaction_service.list_paginated(portfolio_id, tx_query).await?;
    let response: PaginatedResponse<TransactionResponse> = PaginatedResponse::from(page)
        .map(|t| TransactionResponse::from_transaction(t, &stocks));

    let link = build_link_header(
        request.path(),
        request.query_string(),
        response.page,
        response.per_page,
        response.total,
    );

    let mut builder = HttpResponse::Ok();
    builder.insert_header(("Cache-Control", CACHE_CONTROL));
    if let Some(link) = link {
        builder.insert_header(("Link", link));
    }
    Ok(builder.json(response))
}

#[get("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn get_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
) -> Result<HttpResponse, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let (transaction, stocks) = state.transaction_service.get(portfolio_id, tx_id).await?;
    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", CACHE_CONTROL))
        .json(TransactionResponse::from_transaction(transaction, &stocks)))
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
