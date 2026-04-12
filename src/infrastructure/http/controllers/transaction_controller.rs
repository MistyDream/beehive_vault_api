use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use chrono::NaiveDate;
use serde::Deserialize;

use crate::application::error::AppError;
use crate::domain::wallet::enums::TransactionType;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::models::transaction::NewTransactionRow;
use crate::infrastructure::persistence::repositories::{
    portfolio_repository, transaction_repository,
};
use crate::infrastructure::persistence::repositories::transaction_repository::TransactionFilter;

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub stock_id: Option<i32>,
    pub transaction_type: String,
    pub executed_at: NaiveDate,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub amount: Option<f64>,
    pub fees: Option<f64>,
    pub tax: Option<f64>,
    pub split_from: Option<i32>,
    pub split_to: Option<i32>,
    pub currency: String,
    pub exchange_rate: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub stock_id: Option<i32>,
    pub transaction_type: String,
    pub executed_at: NaiveDate,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub amount: Option<f64>,
    pub fees: Option<f64>,
    pub tax: Option<f64>,
    pub split_from: Option<i32>,
    pub split_to: Option<i32>,
    pub currency: String,
    pub exchange_rate: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub transaction_type: Option<String>,
    pub stock_id: Option<i32>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

/// Validate required fields depending on transaction type.
fn validate_transaction(tx_type: &TransactionType, req: &CreateTransactionRequest) -> Result<(), String> {
    match tx_type {
        TransactionType::Buy | TransactionType::Sell => {
            if req.stock_id.is_none() {
                return Err(format!("{} requires stock_id", tx_type.as_str()));
            }
            if req.quantity.is_none() {
                return Err(format!("{} requires quantity", tx_type.as_str()));
            }
            if req.unit_price.is_none() {
                return Err(format!("{} requires unit_price", tx_type.as_str()));
            }
        }
        TransactionType::Dividend => {
            if req.stock_id.is_none() {
                return Err("dividend requires stock_id".to_owned());
            }
            if req.amount.is_none() {
                return Err("dividend requires amount".to_owned());
            }
        }
        TransactionType::Fee => {
            if req.amount.is_none() {
                return Err("fee requires amount".to_owned());
            }
        }
        TransactionType::Split => {
            if req.stock_id.is_none() {
                return Err("split requires stock_id".to_owned());
            }
            if req.split_from.is_none() || req.split_to.is_none() {
                return Err("split requires split_from and split_to".to_owned());
            }
        }
        TransactionType::Deposit | TransactionType::Withdrawal => {
            if req.amount.is_none() {
                return Err(format!("{} requires amount", tx_type.as_str()));
            }
        }
    }
    Ok(())
}

fn build_new_row(req: &CreateTransactionRequest) -> NewTransactionRow<'static> {
    NewTransactionRow {
        portfolio_id: 0, // overwritten by caller
        stock_id: req.stock_id,
        transaction_type: Box::leak(req.transaction_type.clone().into_boxed_str()),
        executed_at: req.executed_at,
        quantity: req.quantity,
        unit_price: req.unit_price,
        amount: req.amount,
        fees: req.fees.unwrap_or(0.0),
        tax: req.tax.unwrap_or(0.0),
        split_from: req.split_from,
        split_to: req.split_to,
        currency: Box::leak(req.currency.clone().into_boxed_str()),
        exchange_rate: req.exchange_rate.unwrap_or(1.0),
        notes: req.notes.as_deref().map(|s| &*Box::leak(s.to_owned().into_boxed_str())),
    }
}

fn build_new_row_from_update(req: &UpdateTransactionRequest) -> NewTransactionRow<'static> {
    NewTransactionRow {
        portfolio_id: 0, // overwritten by caller
        stock_id: req.stock_id,
        transaction_type: Box::leak(req.transaction_type.clone().into_boxed_str()),
        executed_at: req.executed_at,
        quantity: req.quantity,
        unit_price: req.unit_price,
        amount: req.amount,
        fees: req.fees.unwrap_or(0.0),
        tax: req.tax.unwrap_or(0.0),
        split_from: req.split_from,
        split_to: req.split_to,
        currency: Box::leak(req.currency.clone().into_boxed_str()),
        exchange_rate: req.exchange_rate.unwrap_or(1.0),
        notes: req.notes.as_deref().map(|s| &*Box::leak(s.to_owned().into_boxed_str())),
    }
}

#[post("/portfolios/{id}/transactions")]
pub async fn create_transaction(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<CreateTransactionRequest>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();

    // Verify portfolio exists
    portfolio_repository::find_by_id(&state.db, portfolio_id)
        .await
        .map_err(AppError::from)?;

    let tx_type = TransactionType::try_from(body.transaction_type.as_str())
        .map_err(|e| ApiError::from(AppError::BadRequest(e)))?;

    validate_transaction(&tx_type, &body)
        .map_err(|e| ApiError::from(AppError::BadRequest(e)))?;

    let mut new = build_new_row(&body);
    new.portfolio_id = portfolio_id;

    let transaction = transaction_repository::insert(&state.db, new)
        .await
        .map_err(AppError::from)?;
    Ok(HttpResponse::Created().json(transaction))
}

#[get("/portfolios/{id}/transactions")]
pub async fn list_transactions(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    query: web::Query<TransactionQueryParams>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();

    let has_filters = query.transaction_type.is_some()
        || query.stock_id.is_some()
        || query.from_date.is_some()
        || query.to_date.is_some();

    let transactions = if has_filters {
        let filters = TransactionFilter {
            transaction_type: query.transaction_type.clone(),
            stock_id: query.stock_id,
            from_date: query.from_date,
            to_date: query.to_date,
        };
        transaction_repository::list_by_portfolio_filtered(&state.db, portfolio_id, filters)
            .await
            .map_err(AppError::from)?
    } else {
        transaction_repository::list_by_portfolio(&state.db, portfolio_id)
            .await
            .map_err(AppError::from)?
    };

    Ok(HttpResponse::Ok().json(transactions))
}

#[get("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn get_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
) -> Result<impl Responder, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let transaction = transaction_repository::find_by_id(&state.db, portfolio_id, tx_id)
        .await
        .map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[put("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn update_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
    body: web::Json<UpdateTransactionRequest>,
) -> Result<impl Responder, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();

    let tx_type = TransactionType::try_from(body.transaction_type.as_str())
        .map_err(|e| ApiError::from(AppError::BadRequest(e)))?;

    // Reuse the same validation (fields are identical)
    let as_create = CreateTransactionRequest {
        stock_id: body.stock_id,
        transaction_type: body.transaction_type.clone(),
        executed_at: body.executed_at,
        quantity: body.quantity,
        unit_price: body.unit_price,
        amount: body.amount,
        fees: body.fees,
        tax: body.tax,
        split_from: body.split_from,
        split_to: body.split_to,
        currency: body.currency.clone(),
        exchange_rate: body.exchange_rate,
        notes: body.notes.clone(),
    };
    validate_transaction(&tx_type, &as_create)
        .map_err(|e| ApiError::from(AppError::BadRequest(e)))?;

    let mut new = build_new_row_from_update(&body);
    new.portfolio_id = portfolio_id;

    let transaction = transaction_repository::update(&state.db, portfolio_id, tx_id, new)
        .await
        .map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(transaction))
}

#[delete("/portfolios/{portfolio_id}/transactions/{tx_id}")]
pub async fn delete_transaction(
    state: web::Data<AppState>,
    path: web::Path<(i32, i64)>,
) -> Result<impl Responder, ApiError> {
    let (portfolio_id, tx_id) = path.into_inner();
    let deleted = transaction_repository::delete(&state.db, portfolio_id, tx_id)
        .await
        .map_err(AppError::from)?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::from(AppError::NotFound))
    }
}
