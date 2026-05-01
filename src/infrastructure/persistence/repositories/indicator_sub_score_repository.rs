use diesel::prelude::*;

use crate::application::error::AppError;
use crate::domain::scoring::indicator_sub_score::IndicatorSubScore;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::indicator_sub_score::{
    IndicatorSubScoreRow, NewIndicatorSubScoreRow,
};
use crate::schema::indicator_sub_scores;

#[derive(Clone)]
pub struct PgIndicatorSubScoreRepository {
    db: Db,
}

impl PgIndicatorSubScoreRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<IndicatorSubScore, AppError> {
        self.db
            .exec(move |conn| {
                let row = indicator_sub_scores::table
                    .find(id)
                    .select(IndicatorSubScoreRow::as_select())
                    .first(conn)?;
                Ok(IndicatorSubScore::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    pub async fn find_by_indicator_score(
        &self,
        indicator_score_id: i32,
    ) -> Result<Vec<IndicatorSubScore>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = indicator_sub_scores::table
                    .filter(indicator_sub_scores::indicator_score_id.eq(indicator_score_id))
                    .select(IndicatorSubScoreRow::as_select())
                    .order(indicator_sub_scores::sub_score_type.asc())
                    .load(conn)?;
                Ok(rows.into_iter().map(IndicatorSubScore::from).collect())
            })
            .await
            .map_err(AppError::from)
    }

    pub async fn insert(
        &self,
        new: NewIndicatorSubScoreRow<'static>,
    ) -> Result<IndicatorSubScore, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(indicator_sub_scores::table)
                    .values(&new)
                    .returning(IndicatorSubScoreRow::as_returning())
                    .get_result(conn)?;
                Ok(IndicatorSubScore::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    pub async fn insert_many(
        &self,
        rows: Vec<NewIndicatorSubScoreRow<'static>>,
    ) -> Result<Vec<IndicatorSubScore>, AppError> {
        self.db
            .exec(move |conn| {
                let inserted = diesel::insert_into(indicator_sub_scores::table)
                    .values(&rows)
                    .returning(IndicatorSubScoreRow::as_returning())
                    .get_results(conn)?;
                Ok(inserted.into_iter().map(IndicatorSubScore::from).collect())
            })
            .await
            .map_err(AppError::from)
    }

    pub async fn delete_by_indicator_score(
        &self,
        indicator_score_id: i32,
    ) -> Result<usize, AppError> {
        self.db
            .exec(move |conn| {
                let count = diesel::delete(
                    indicator_sub_scores::table
                        .filter(indicator_sub_scores::indicator_score_id.eq(indicator_score_id)),
                )
                .execute(conn)?;
                Ok(count)
            })
            .await
            .map_err(AppError::from)
    }
}
