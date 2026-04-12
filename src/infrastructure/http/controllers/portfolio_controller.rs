use actix_web::{HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::Json;

use crate::application::error::AppError;
use crate::infrastructure::http::dto::request::portfolio_request::{
    CreatePortfolioRequest, UpdatePortfolioRequest,
};
use crate::infrastructure::http::dto::response::portfolio_response::PortfolioResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;
use crate::infrastructure::persistence::models::portfolio::NewPortfolioRow;
use crate::infrastructure::persistence::repositories::portfolio_repository;

#[post("/portfolios")]
pub async fn create_portfolio(
    state: web::Data<AppState>,
    body: Json<CreatePortfolioRequest>,
) -> Result<HttpResponse, ApiError> {
    let body = body.into_inner();
    let currency = body.currency.unwrap_or_else(|| "EUR".to_owned());

    let new = NewPortfolioRow {
        name: Box::leak(body.name.into_boxed_str()),
        kind: Box::leak(body.kind.into_boxed_str()),
        currency: Box::leak(currency.into_boxed_str()),
        description: body
            .description
            .map(|s| &*Box::leak(s.into_boxed_str())),
    };

    let portfolio = portfolio_repository::insert(&state.db, new)
        .await
        .map_err(AppError::from)?;

    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/portfolios/{}", portfolio.id)))
        .json(PortfolioResponse::from(portfolio)))
}

#[get("/portfolios")]
pub async fn list_portfolios(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let portfolios = portfolio_repository::list_all(&state.db)
        .await
        .map_err(AppError::from)?;

    let response: Vec<PortfolioResponse> = portfolios.into_iter().map(PortfolioResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

#[get("/portfolios/{id}")]
pub async fn get_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let portfolio = portfolio_repository::find_by_id(&state.db, portfolio_id)
        .await
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(PortfolioResponse::from(portfolio)))
}

#[put("/portfolios/{id}")]
pub async fn update_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: Json<UpdatePortfolioRequest>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let body = body.into_inner();
    let currency = body.currency.unwrap_or_else(|| "EUR".to_owned());

    let new = NewPortfolioRow {
        name: Box::leak(body.name.into_boxed_str()),
        kind: Box::leak(body.kind.into_boxed_str()),
        currency: Box::leak(currency.into_boxed_str()),
        description: body
            .description
            .map(|s| &*Box::leak(s.into_boxed_str())),
    };

    let portfolio = portfolio_repository::update(&state.db, portfolio_id, new)
        .await
        .map_err(AppError::from)?;

    Ok(HttpResponse::Ok().json(PortfolioResponse::from(portfolio)))
}

#[delete("/portfolios/{id}")]
pub async fn delete_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let portfolio_id = path.into_inner();
    let deleted = portfolio_repository::delete(&state.db, portfolio_id)
        .await
        .map_err(AppError::from)?;

    if deleted {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::from(AppError::NotFound))
    }
}
