mod catalog;
mod domain;
mod dto;
mod handlers;
mod repository;
mod service;

use axum::{
    Router,
    routing::{patch, post},
};

use crate::database::Database;

use service::CategoryService;

pub(crate) use catalog::INITIAL_CATEGORIES;
pub(crate) use domain::CategoryKind;
pub(crate) use repository::CategoryRepository;

#[derive(Clone)]
pub(crate) struct CategoriesModule {
    service: CategoryService,
}

pub(crate) fn configure(database: Database) -> CategoriesModule {
    let repository = CategoryRepository::new(database);
    let service = CategoryService::new(repository);

    CategoriesModule { service }
}

pub(crate) fn routes(module: CategoriesModule) -> Router {
    Router::new()
        .route(
            "/households/{household_id}/categories",
            post(handlers::create).get(handlers::list),
        )
        .route(
            "/households/{household_id}/categories/{category_id}",
            patch(handlers::update).delete(handlers::archive),
        )
        .with_state(module)
}
