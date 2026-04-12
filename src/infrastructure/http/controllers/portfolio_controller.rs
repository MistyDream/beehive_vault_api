use actix_web::{HttpResponse, delete, get, post, put, web};
use garde_actix_web::web::Json;

use crate::domain::wallet::portfolio::NewPortfolio;
use crate::infrastructure::http::dto::request::portfolio_request::{
    CreatePortfolioRequest, UpdatePortfolioRequest,
};
use crate::infrastructure::http::dto::response::portfolio_response::PortfolioResponse;
use crate::infrastructure::http::error::ApiError;
use crate::infrastructure::http::state::AppState;

#[post("/portfolios")]
pub async fn create_portfolio(
    state: web::Data<AppState>,
    body: Json<CreatePortfolioRequest>,
) -> Result<HttpResponse, ApiError> {
    let new = NewPortfolio::from(body.into_inner());
    let portfolio = state.portfolio_service.create(new).await?;

    Ok(HttpResponse::Created()
        .insert_header(("Location", format!("/portfolios/{}", portfolio.id)))
        .json(PortfolioResponse::from(portfolio)))
}

#[get("/portfolios")]
pub async fn list_portfolios(
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let portfolios = state.portfolio_service.list().await?;
    let response: Vec<PortfolioResponse> = portfolios.into_iter().map(PortfolioResponse::from).collect();
    Ok(HttpResponse::Ok().json(response))
}

#[get("/portfolios/{id}")]
pub async fn get_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let portfolio = state.portfolio_service.get(path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(PortfolioResponse::from(portfolio)))
}

#[put("/portfolios/{id}")]
pub async fn update_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
    body: Json<UpdatePortfolioRequest>,
) -> Result<HttpResponse, ApiError> {
    let portfolio = state.portfolio_service.update(path.into_inner(), body.into_inner().into()).await?;
    Ok(HttpResponse::Ok().json(PortfolioResponse::from(portfolio)))
}

#[delete("/portfolios/{id}")]
pub async fn delete_portfolio(
    state: web::Data<AppState>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    state.portfolio_service.delete(path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
