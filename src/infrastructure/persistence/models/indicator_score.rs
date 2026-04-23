use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::scoring::indicator_score::IndicatorScore;
use crate::schema::indicator_scores;

#[derive(Queryable, Selectable)]
#[diesel(table_name = indicator_scores)]
pub struct IndicatorScoreRow {
    pub id: i32,
    pub detail_id: i32,
    pub metric_key: String,
    pub score: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = indicator_scores)]
pub struct NewIndicatorScoreRow<'a> {
    pub detail_id: i32,
    pub metric_key: &'a str,
    pub score: f64,
}

impl From<IndicatorScoreRow> for IndicatorScore {
    fn from(row: IndicatorScoreRow) -> Self {
        IndicatorScore {
            id: row.id,
            detail_id: row.detail_id,
            metric_key: row.metric_key,
            score: row.score,
        }
    }
}
