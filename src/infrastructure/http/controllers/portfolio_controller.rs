use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use serde::Deserialize;

use crate::application::error::AppError;
use crate::domain::wallet::enums::PortfolioKind;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::models::portfolio::NewPortfolioRow;
use crate::infrastructure::persistence::repositories::portfolio_repository;

#[derive(Debug, Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: String,
    pub kind: String,
    pub currency: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePortfolioRequest {
    pub name: String,
    pub kind: String,
    pub currency: Option<String>,
    pub description: Option<String>,
}

#[post("/portfolios")]
pub async fn create_portfolio(
    state: web::Data<AppState>,
    body: web::Json<CreatePortfolioRequest>,
) -> Result<impl Responder, ApiError> {
    PortfolioKind::try_from(body.kind.as_str())
        .map_err(|e| ApiError::from(AppError::BadRequest(e)))?;

    let currency = body.currency.clone().unwrap_or_else(|| "EUR".to_owned());

    let new = NewPortfolioRow {
        name: Box::leak(body.name.clone().into_boxed_str()),
        kind: Box::leak(body.kind.clone().into_boxed_str()),
        currency: Box::leak(currency.into_boxed_str()),
        description: body
            .description
            .as_deref()
            .map(|s| &*Box::leak(s.to_owned().into_boxed_str())),
    };

    let portfolio = portfolio_repository::insert(&state.db, new).await.map_err(AppError::from)?;
    Ok(HttpResponse::Created().json(portfolio))
}

#[get("/portfolios")]
pub async fn list_portfolios(
    state: web::Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let portfolios = portfolio_repository::list_all(&state.db).await.map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(portfolios))
}

#[get("/portfolios/{id}")]
pub async fn get_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();
    let portfolio = portfolio_repository::find_by_id(&state.db, portfolio_id).await.map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(portfolio))
}

#[put("/portfolios/{id}")]
pub async fn update_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: web::Json<UpdatePortfolioRequest>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();

    PortfolioKind::try_from(body.kind.as_str())
        .map_err(|e| ApiError::from(AppError::BadRequest(e)))?;

    let currency = body.currency.clone().unwrap_or_else(|| "EUR".to_owned());

    let new = NewPortfolioRow {
        name: Box::leak(body.name.clone().into_boxed_str()),
        kind: Box::leak(body.kind.clone().into_boxed_str()),
        currency: Box::leak(currency.into_boxed_str()),
        description: body
            .description
            .as_deref()
            .map(|s| &*Box::leak(s.to_owned().into_boxed_str())),
    };

    let portfolio = portfolio_repository::update(&state.db, portfolio_id, new).await.map_err(AppError::from)?;
    Ok(HttpResponse::Ok().json(portfolio))
}

#[delete("/portfolios/{id}")]
pub async fn delete_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<impl Responder, ApiError> {
    let portfolio_id = path.into_inner();
    let deleted = portfolio_repository::delete(&state.db, portfolio_id).await.map_err(AppError::from)?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::from(AppError::NotFound))
    }
}
