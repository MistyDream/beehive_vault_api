//! Repository for the `score_snapshots` table.
//!
//! A score snapshot represents one scoring run for a stock at a given point in
//! time. It holds the `global_score` (0-100) and links to [`ScoreDetail`] rows
//! for per-category breakdowns.

use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::score_snapshot_repository::ScoreSnapshotRepository;
use crate::domain::market::score_snapshot::ScoreSnapshot;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::score_snapshot::{
    NewScoreSnapshotRow, ScoreSnapshotRow,
};
use crate::schema::score_snapshots;

#[derive(Clone)]
pub struct PgScoreSnapshotRepository {
    db: Db,
}

impl PgScoreSnapshotRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Insert a new score snapshot and return the created entity.
    pub async fn insert(&self, new: NewScoreSnapshotRow) -> Result<ScoreSnapshot, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(score_snapshots::table)
                    .values(&new)
                    .returning(ScoreSnapshotRow::as_returning())
                    .get_result(conn)?;
                Ok(ScoreSnapshot::from(row))
            })
            .await
            .map_err(AppError::from)
    }
}

impl ScoreSnapshotRepository for PgScoreSnapshotRepository {
    fn find_by_id(
        &self,
        snapshot_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<ScoreSnapshot, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = score_snapshots::table
                        .find(snapshot_id)
                        .select(ScoreSnapshotRow::as_select())
                        .first(conn)?;
                    Ok(ScoreSnapshot::from(row))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_by_stock(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ScoreSnapshot>, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let rows = score_snapshots::table
                        .filter(score_snapshots::stock_id.eq(stock_id))
                        .select(ScoreSnapshotRow::as_select())
                        .order(score_snapshots::scored_at.desc())
                        .load(conn)?;
                    Ok(rows.into_iter().map(ScoreSnapshot::from).collect())
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn find_latest_by_stock(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<ScoreSnapshot, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let row = score_snapshots::table
                        .filter(score_snapshots::stock_id.eq(stock_id))
                        .select(ScoreSnapshotRow::as_select())
                        .order(score_snapshots::scored_at.desc())
                        .first(conn)?;
                    Ok(ScoreSnapshot::from(row))
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn delete(
        &self,
        snapshot_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count =
                        diesel::delete(score_snapshots::table.find(snapshot_id)).execute(conn)?;
                    Ok(count > 0)
                })
                .await
                .map_err(AppError::from)
        })
    }

    fn delete_by_stock(
        &self,
        stock_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count = diesel::delete(
                        score_snapshots::table
                            .filter(score_snapshots::stock_id.eq(stock_id)),
                    )
                    .execute(conn)?;
                    Ok(count)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
