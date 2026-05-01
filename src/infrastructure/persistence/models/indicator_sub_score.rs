use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::scoring::indicator_sub_score::IndicatorSubScore;
use crate::schema::indicator_sub_scores;

#[derive(Queryable, Selectable)]
#[diesel(table_name = indicator_sub_scores)]
pub struct IndicatorSubScoreRow {
    pub id: i32,
    pub indicator_score_id: i32,
    pub sub_score_type: String,
    pub score: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = indicator_sub_scores)]
pub struct NewIndicatorSubScoreRow<'a> {
    pub indicator_score_id: i32,
    pub sub_score_type: &'a str,
    pub score: f64,
}

impl From<IndicatorSubScoreRow> for IndicatorSubScore {
    fn from(row: IndicatorSubScoreRow) -> Self {
        IndicatorSubScore {
            id: row.id,
            indicator_score_id: row.indicator_score_id,
            sub_score_type: row.sub_score_type,
            score: row.score,
        }
    }
}
