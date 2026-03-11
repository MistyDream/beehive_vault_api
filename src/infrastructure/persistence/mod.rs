pub mod error;
pub mod models;
pub mod pool;
pub mod repositories;

use tokio::task;
use diesel::pg::PgConnection;

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
}

pub fn connect(config: &DbConfig) -> Result<Db, error::DbError> {
    let pool = pool::create_pool(&config.database_url)?;
    Ok(Db::new(pool))
}