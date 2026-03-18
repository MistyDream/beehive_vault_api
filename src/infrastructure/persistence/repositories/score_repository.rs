use std::future::Future;
use std::pin::Pin;

use chrono::Utc;
use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::score_repository::ScoreRepository;
use crate::domain::scoring::ScoringResult;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::indicator_score::NewIndicatorScoreRow;
use crate::infrastructure::persistence::models::indicator_sub_score::NewIndicatorSubScoreRow;
use crate::infrastructure::persistence::models::score_detail::NewScoreDetailRow;
use crate::infrastructure::persistence::models::score_snapshot::NewScoreSnapshotRow;
use crate::schema::{indicator_scores, indicator_sub_scores, score_details, score_snapshots};

#[derive(Clone)]
pub struct PgScoreRepository {
    db: Db,
}

impl PgScoreRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl ScoreRepository for PgScoreRepository {
    fn save_scoring(
        &self,
        stock_id: i32,
        result: ScoringResult,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    conn.transaction(|conn| {
                        let snapshot = NewScoreSnapshotRow {
                            stock_id,
                            scored_at: Utc::now().naive_utc(),
                            global_score: result.global_score,
                        };
                        let snapshot_id: i32 = diesel::insert_into(score_snapshots::table)
                            .values(&snapshot)
                            .returning(score_snapshots::id)
                            .get_result(conn)?;

                        for cat in &result.categories {
                            let detail = NewScoreDetailRow {
                                snapshot_id,
                                category: cat.category.as_str().to_string(),
                                score: cat.score,
                                weight: cat.weight,
                            };
                            let detail_id: i32 = diesel::insert_into(score_details::table)
                                .values(&detail)
                                .returning(score_details::id)
                                .get_result(conn)?;

                            for indicator in &cat.indicators {
                                let ind = NewIndicatorScoreRow {
                                    detail_id,
                                    metric_key: indicator.metric_key.clone(),
                                    score: indicator.score,
                                };
                                let indicator_id: i32 = diesel::insert_into(indicator_scores::table)
                                    .values(&ind)
                                    .returning(indicator_scores::id)
                                    .get_result(conn)?;

                                let sub_scores = vec![
                                    NewIndicatorSubScoreRow {
                                        indicator_score_id: indicator_id,
                                        sub_score_type: "sector".to_string(),
                                        score: indicator.sector_score,
                                    },
                                    NewIndicatorSubScoreRow {
                                        indicator_score_id: indicator_id,
                                        sub_score_type: "historical".to_string(),
                                        score: indicator.historical_score,
                                    },
                                ];
                                diesel::insert_into(indicator_sub_scores::table)
                                    .values(&sub_scores)
                                    .execute(conn)?;
                            }
                        }

                        Ok(())
                    })
                })
                .await
                .map_err(AppError::from)
        })
    }
}
