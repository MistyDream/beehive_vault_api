use std::future::Future;
use std::pin::Pin;

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::application::ports::metric_value_repository::MetricValueRepository;
use crate::domain::market::metric_value::NewMetricValue;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::metric_value::NewMetricValueRow;
use crate::schema::metric_values;

#[derive(Clone)]
pub struct PgMetricValueRepository {
    db: Db,
}

impl PgMetricValueRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl MetricValueRepository for PgMetricValueRepository {
    fn bulk_insert_ignore(
        &self,
        values: Vec<NewMetricValue>,
    ) -> Pin<Box<dyn Future<Output = Result<usize, AppError>> + Send + '_>> {
        let rows: Vec<NewMetricValueRow> = values.into_iter().map(NewMetricValueRow::from).collect();
        Box::pin(async move {
            self.db
                .exec(move |conn| {
                    let count = diesel::insert_into(metric_values::table)
                        .values(&rows)
                        .on_conflict((
                            metric_values::stock_id,
                            metric_values::metric_key,
                            metric_values::period,
                            metric_values::period_end,
                        ))
                        .do_nothing()
                        .execute(conn)?;
                    Ok(count)
                })
                .await
                .map_err(AppError::from)
        })
    }
}
