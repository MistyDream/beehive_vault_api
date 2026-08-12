use axum::Router;
use sqlx::PgPool;

use crate::{
    database::Database,
    features::{accounts, categories, health, households, institutions, net_worth},
};

pub fn build(pool: PgPool) -> Router {
    let database = Database::new(pool);
    let accounts = accounts::configure(database.clone());
    let categories = categories::configure(database.clone());
    let health = health::configure(database.clone());
    let households = households::configure(database.clone());
    let institutions = institutions::configure(database.clone());
    let net_worth = net_worth::configure(database);

    let api = Router::new()
        .merge(accounts::routes(accounts))
        .merge(categories::routes(categories))
        .merge(households::routes(households))
        .merge(institutions::routes(institutions))
        .merge(net_worth::routes(net_worth));

    Router::new().merge(health::routes(health)).nest("/v1", api)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn healthz_reports_a_live_process() {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/beehive_vault")
            .expect("test database URL should be valid");
        let response = build(db)
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
