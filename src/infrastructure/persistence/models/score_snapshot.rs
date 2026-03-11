use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::market::score_snapshot::ScoreSnapshot;
use crate::schema::score_snapshots;

#[derive(Queryable, Selectable)]
#[diesel(table_name = score_snapshots)]
pub struct ScoreSnapshotRow {
    pub id: i32,
    pub stock_id: i32,
    pub scored_at: NaiveDateTime,
    pub global_score: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = score_snapshots)]
pub struct NewScoreSnapshotRow {
    pub stock_id: i32,
    pub scored_at: NaiveDateTime,
    pub global_score: f64,
}

impl From<ScoreSnapshotRow> for ScoreSnapshot {
    fn from(row: ScoreSnapshotRow) -> Self {
        ScoreSnapshot {
            id: row.id,
            stock_id: row.stock_id,
            scored_at: row.scored_at,
            global_score: row.global_score,
        }
    }
}
