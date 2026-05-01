pub mod error;
pub mod models;
pub mod pool;
pub mod repositories;

use std::future::Future;
use std::pin::Pin;

use tokio::task;
use diesel::pg::PgConnection;

use crate::application::error::AppError;
use crate::application::ports::health_checker::HealthChecker;
use crate::config::settings::DbConfig;

#[derive(Clone)]
pub struct Db {
    pool: pool::DbPool,
}

impl Db {
    pub fn new(pool: pool::DbPool) -> Self {
        Self { pool }
    }

    pub async fn exec<T, F>(&self, f: F) -> Result<T, error::DbError>
    where
        T: Send + 'static,
        F: FnOnce(&mut PgConnection) -> Result<T, error::DbError> + Send + 'static,
    {
        let pool = self.pool.clone();

        task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            f(&mut conn)
        })
        .await?
    }

    /// Cheap readiness probe: issues `SELECT 1` on a pooled connection.
    /// Used by `/readyz` to decide whether the container can receive traffic.
    pub async fn ping(&self) -> Result<(), error::DbError> {
        use diesel::prelude::*;
        self.exec(|conn| {
            diesel::sql_query("SELECT 1")
                .execute(conn)
                .map(|_| ())
                .map_err(Into::into)
        })
        .await
    }
}

/// `Db` is itself the concrete adapter for the `HealthChecker` port —
/// the DB is the only external dependency we want `/readyz` to probe
/// today. Keeping the impl on `Db` avoids a trivial wrapper struct.
impl HealthChecker for Db {
    fn readiness(&self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move { self.ping().await.map_err(Into::into) })
    }
}

pub fn connect(config: &DbConfig) -> Result<Db, error::DbError> {
    let pool = pool::create_pool(&config.database_url)?;
    Ok(Db::new(pool))
}