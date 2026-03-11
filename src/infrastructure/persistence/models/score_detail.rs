use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::market::enums::MetricCategory;
use crate::domain::market::score_detail::ScoreDetail;
use crate::infrastructure::persistence::error::DbError;
use crate::schema::score_details;

#[derive(Queryable, Selectable)]
#[diesel(table_name = score_details)]
pub struct ScoreDetailRow {
    pub id: i32,
    pub snapshot_id: i32,
    pub category: String,
    pub score: f64,
    pub weight: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = score_details)]
pub struct NewScoreDetailRow<'a> {
    pub snapshot_id: i32,
    pub category: &'a str,
    pub score: f64,
    pub weight: f64,
}

impl TryFrom<ScoreDetailRow> for ScoreDetail {
    type Error = DbError;

    fn try_from(row: ScoreDetailRow) -> Result<Self, Self::Error> {
        Ok(ScoreDetail {
            id: row.id,
            snapshot_id: row.snapshot_id,
            category: MetricCategory::try_from(row.category.as_str())
                .map_err(DbError::Conversion)?,
            score: row.score,
            weight: row.weight,
        })
    }
}
