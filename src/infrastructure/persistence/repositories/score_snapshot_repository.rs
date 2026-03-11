//! Repository for the `score_snapshots` table.
//!
//! A score snapshot represents one scoring run for a stock at a given point in
//! time. It holds the `global_score` (0–100) and links to [`ScoreDetail`] rows
//! for per-category breakdowns.

use diesel::prelude::*;

use crate::domain::market::score_snapshot::ScoreSnapshot;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::score_snapshot::{
    NewScoreSnapshotRow, ScoreSnapshotRow,
};
use crate::schema::score_snapshots;

/// Fetch a single score snapshot by its primary key.
pub async fn find_by_id(db: &Db, snapshot_id: i32) -> Result<ScoreSnapshot, DbError> {
    db.exec(move |conn| {
        let row = score_snapshots::table
            .find(snapshot_id)
            .select(ScoreSnapshotRow::as_select())
            .first(conn)?;
        Ok(ScoreSnapshot::from(row))
    })
    .await
}

/// Fetch all score snapshots for a stock, most recent first (full history).
pub async fn find_by_stock(
    db: &Db,
    stock_id: i32,
) -> Result<Vec<ScoreSnapshot>, DbError> {
    db.exec(move |conn| {
        let rows = score_snapshots::table
            .filter(score_snapshots::stock_id.eq(stock_id))
            .select(ScoreSnapshotRow::as_select())
            .order(score_snapshots::scored_at.desc())
            .load(conn)?;
        Ok(rows.into_iter().map(ScoreSnapshot::from).collect())
    })
    .await
}

/// Fetch the most recent score snapshot for a stock.
/// Returns `DbError::Diesel(NotFound)` if the stock has never been scored.
pub async fn find_latest_by_stock(
    db: &Db,
    stock_id: i32,
) -> Result<ScoreSnapshot, DbError> {
    db.exec(move |conn| {
        let row = score_snapshots::table
            .filter(score_snapshots::stock_id.eq(stock_id))
            .select(ScoreSnapshotRow::as_select())
            .order(score_snapshots::scored_at.desc())
            .first(conn)?;
        Ok(ScoreSnapshot::from(row))
    })
    .await
}

/// Insert a new score snapshot and return the created entity.
pub async fn insert(
    db: &Db,
    new: NewScoreSnapshotRow,
) -> Result<ScoreSnapshot, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(score_snapshots::table)
            .values(&new)
            .returning(ScoreSnapshotRow::as_returning())
            .get_result(conn)?;
        Ok(ScoreSnapshot::from(row))
    })
    .await
}

/// Delete a snapshot by ID (cascades to its score_details). Returns `true` if deleted.
pub async fn delete(db: &Db, snapshot_id: i32) -> Result<bool, DbError> {
    db.exec(move |conn| {
        let count =
            diesel::delete(score_snapshots::table.find(snapshot_id)).execute(conn)?;
        Ok(count > 0)
    })
    .await
}

/// Delete all snapshots for a stock (cascades to their score_details). Returns deleted count.
pub async fn delete_by_stock(db: &Db, stock_id: i32) -> Result<usize, DbError> {
    db.exec(move |conn| {
        let count = diesel::delete(
            score_snapshots::table
                .filter(score_snapshots::stock_id.eq(stock_id)),
        )
        .execute(conn)?;
        Ok(count)
    })
    .await
}
