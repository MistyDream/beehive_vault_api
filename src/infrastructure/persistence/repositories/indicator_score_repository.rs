use diesel::prelude::*;

use crate::domain::market::indicator_score::IndicatorScore;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::indicator_score::{
    IndicatorScoreRow, NewIndicatorScoreRow,
};
use crate::schema::indicator_scores;

pub async fn find_by_id(db: &Db, id: i32) -> Result<IndicatorScore, DbError> {
    db.exec(move |conn| {
        let row = indicator_scores::table
            .find(id)
            .select(IndicatorScoreRow::as_select())
            .first(conn)?;
        Ok(IndicatorScore::from(row))
    })
    .await
}

pub async fn find_by_detail(db: &Db, detail_id: i32) -> Result<Vec<IndicatorScore>, DbError> {
    db.exec(move |conn| {
        let rows = indicator_scores::table
            .filter(indicator_scores::detail_id.eq(detail_id))
            .select(IndicatorScoreRow::as_select())
            .order(indicator_scores::metric_key.asc())
            .load(conn)?;
        Ok(rows.into_iter().map(IndicatorScore::from).collect())
    })
    .await
}

pub async fn insert(
    db: &Db,
    new: NewIndicatorScoreRow<'static>,
) -> Result<IndicatorScore, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(indicator_scores::table)
            .values(&new)
            .returning(IndicatorScoreRow::as_returning())
            .get_result(conn)?;
        Ok(IndicatorScore::from(row))
    })
    .await
}

pub async fn insert_many(
    db: &Db,
    rows: Vec<NewIndicatorScoreRow<'static>>,
) -> Result<Vec<IndicatorScore>, DbError> {
    db.exec(move |conn| {
        let inserted = diesel::insert_into(indicator_scores::table)
            .values(&rows)
            .returning(IndicatorScoreRow::as_returning())
            .get_results(conn)?;
        Ok(inserted.into_iter().map(IndicatorScore::from).collect())
    })
    .await
}

pub async fn delete_by_detail(db: &Db, detail_id: i32) -> Result<usize, DbError> {
    db.exec(move |conn| {
        let count = diesel::delete(
            indicator_scores::table.filter(indicator_scores::detail_id.eq(detail_id)),
        )
        .execute(conn)?;
        Ok(count)
    })
    .await
}
