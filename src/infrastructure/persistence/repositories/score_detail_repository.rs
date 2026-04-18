//! Repository for the `score_details` table.
//!
//! Each row is one category score within a [`ScoreSnapshot`]: the per-category
//! score (0-100) and its weight used to compute the global score.
//! Typically 6 rows per snapshot (one per [`MetricCategory`]).

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::domain::market::score_detail::ScoreDetail;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::score_detail::{
    NewScoreDetailRow, ScoreDetailRow,
};
use crate::schema::score_details;

#[derive(Clone)]
pub struct PgScoreDetailRepository {
    db: Db,
}

impl PgScoreDetailRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Fetch a single score detail by its primary key.
    pub async fn find_by_id(&self, detail_id: i32) -> Result<ScoreDetail, AppError> {
        self.db
            .exec(move |conn| {
                let row = score_details::table
                    .find(detail_id)
                    .select(ScoreDetailRow::as_select())
                    .first(conn)?;
                ScoreDetail::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch all category scores for a given snapshot, ordered by category name.
    pub async fn find_by_snapshot(
        &self,
        snapshot_id: i32,
    ) -> Result<Vec<ScoreDetail>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = score_details::table
                    .filter(score_details::snapshot_id.eq(snapshot_id))
                    .select(ScoreDetailRow::as_select())
                    .order(score_details::category.asc())
                    .load(conn)?;
                rows.into_iter().map(ScoreDetail::try_from).collect()
            })
            .await
            .map_err(AppError::from)
    }

    /// Insert a single category score.
    pub async fn insert(
        &self,
        new: NewScoreDetailRow<'static>,
    ) -> Result<ScoreDetail, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(score_details::table)
                    .values(&new)
                    .returning(ScoreDetailRow::as_returning())
                    .get_result(conn)?;
                ScoreDetail::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }

    /// Bulk insert all category scores for a snapshot (typically 6 rows at once).
    pub async fn insert_many(
        &self,
        rows: Vec<NewScoreDetailRow<'static>>,
    ) -> Result<Vec<ScoreDetail>, AppError> {
        self.db
            .exec(move |conn| {
                let inserted = diesel::insert_into(score_details::table)
                    .values(&rows)
                    .returning(ScoreDetailRow::as_returning())
                    .get_results(conn)?;
                inserted.into_iter().map(ScoreDetail::try_from).collect()
            })
            .await
            .map_err(AppError::from)
    }

    /// Delete all category scores for a snapshot. Returns the number of deleted rows.
    pub async fn delete_by_snapshot(&self, snapshot_id: i32) -> Result<usize, AppError> {
        self.db
            .exec(move |conn| {
                let count = diesel::delete(
                    score_details::table
                        .filter(score_details::snapshot_id.eq(snapshot_id)),
                )
                .execute(conn)?;
                Ok(count)
            })
            .await
            .map_err(AppError::from)
    }
}
