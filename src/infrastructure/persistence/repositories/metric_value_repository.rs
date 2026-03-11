//! Repository for the `metric_values` table.
//!
//! Stores raw financial metric observations (EAV pattern).
//! Each row is one data point: a metric value for a given stock, period type,
//! and period end date. Supports upsert for idempotent data ingestion from
//! external sources (e.g. GuruFocus).

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::upsert::excluded;

use crate::domain::market::metric_value::MetricValue;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::error::DbError;
use crate::infrastructure::persistence::models::metric_value::{
    MetricValueRow, NewMetricValueRow,
};
use crate::schema::metric_values;

/// Fetch a single metric value by its primary key.
pub async fn find_by_id(db: &Db, value_id: i64) -> Result<MetricValue, DbError> {
    db.exec(move |conn| {
        let row = metric_values::table
            .find(value_id)
            .select(MetricValueRow::as_select())
            .first(conn)?;
        MetricValue::try_from(row)
    })
    .await
}

/// Fetch all metric values for a given stock, most recent period first.
pub async fn find_by_stock(db: &Db, stock_id: i32) -> Result<Vec<MetricValue>, DbError> {
    db.exec(move |conn| {
        let rows = metric_values::table
            .filter(metric_values::stock_id.eq(stock_id))
            .select(MetricValueRow::as_select())
            .order(metric_values::period_end.desc())
            .load(conn)?;
        rows.into_iter().map(MetricValue::try_from).collect()
    })
    .await
}

/// Fetch all observations of a specific metric for a given stock, most recent first.
pub async fn find_by_stock_and_metric(
    db: &Db,
    stock_id: i32,
    metric_key: String,
) -> Result<Vec<MetricValue>, DbError> {
    db.exec(move |conn| {
        let rows = metric_values::table
            .filter(
                metric_values::stock_id
                    .eq(stock_id)
                    .and(metric_values::metric_key.eq(&metric_key)),
            )
            .select(MetricValueRow::as_select())
            .order(metric_values::period_end.desc())
            .load(conn)?;
        rows.into_iter().map(MetricValue::try_from).collect()
    })
    .await
}

/// Fetch the most recent observation of a specific metric for a given stock.
/// Returns `DbError::Diesel(NotFound)` if no data exists.
pub async fn find_latest_by_stock_and_metric(
    db: &Db,
    stock_id: i32,
    metric_key: String,
) -> Result<MetricValue, DbError> {
    db.exec(move |conn| {
        let row = metric_values::table
            .filter(
                metric_values::stock_id
                    .eq(stock_id)
                    .and(metric_values::metric_key.eq(&metric_key)),
            )
            .select(MetricValueRow::as_select())
            .order(metric_values::period_end.desc())
            .first(conn)?;
        MetricValue::try_from(row)
    })
    .await
}

/// Fetch all metric values for a stock whose `period_end` falls within `[from, to]`.
pub async fn find_by_stock_and_period_range(
    db: &Db,
    stock_id: i32,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<MetricValue>, DbError> {
    db.exec(move |conn| {
        let rows = metric_values::table
            .filter(
                metric_values::stock_id
                    .eq(stock_id)
                    .and(metric_values::period_end.between(from, to)),
            )
            .select(MetricValueRow::as_select())
            .order(metric_values::period_end.desc())
            .load(conn)?;
        rows.into_iter().map(MetricValue::try_from).collect()
    })
    .await
}

/// Insert a new metric value. Fails on duplicate `(stock_id, metric_key, period, period_end)`.
/// Prefer [`upsert`] for idempotent ingestion.
pub async fn insert(
    db: &Db,
    new: NewMetricValueRow<'static>,
) -> Result<MetricValue, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(metric_values::table)
            .values(&new)
            .returning(MetricValueRow::as_returning())
            .get_result(conn)?;
        MetricValue::try_from(row)
    })
    .await
}

/// Upsert a single metric value on the UNIQUE(stock_id, metric_key, period, period_end) constraint.
/// On conflict, updates value, unit, currency, source, and fetched_at.
pub async fn upsert(
    db: &Db,
    new: NewMetricValueRow<'static>,
) -> Result<MetricValue, DbError> {
    db.exec(move |conn| {
        let row = diesel::insert_into(metric_values::table)
            .values(&new)
            .on_conflict((
                metric_values::stock_id,
                metric_values::metric_key,
                metric_values::period,
                metric_values::period_end,
            ))
            .do_update()
            .set((
                metric_values::value.eq(excluded(metric_values::value)),
                metric_values::unit.eq(excluded(metric_values::unit)),
                metric_values::currency.eq(excluded(metric_values::currency)),
                metric_values::source.eq(excluded(metric_values::source)),
                metric_values::fetched_at.eq(excluded(metric_values::fetched_at)),
            ))
            .returning(MetricValueRow::as_returning())
            .get_result(conn)?;
        MetricValue::try_from(row)
    })
    .await
}

/// Bulk upsert metric values. Returns the number of rows affected.
pub async fn upsert_many(
    db: &Db,
    rows: Vec<NewMetricValueRow<'static>>,
) -> Result<Vec<MetricValue>, DbError> {
    db.exec(move |conn| {
        let inserted = diesel::insert_into(metric_values::table)
            .values(&rows)
            .on_conflict((
                metric_values::stock_id,
                metric_values::metric_key,
                metric_values::period,
                metric_values::period_end,
            ))
            .do_update()
            .set((
                metric_values::value.eq(excluded(metric_values::value)),
                metric_values::unit.eq(excluded(metric_values::unit)),
                metric_values::currency.eq(excluded(metric_values::currency)),
                metric_values::source.eq(excluded(metric_values::source)),
                metric_values::fetched_at.eq(excluded(metric_values::fetched_at)),
            ))
            .returning(MetricValueRow::as_returning())
            .get_results(conn)?;
        inserted.into_iter().map(MetricValue::try_from).collect()
    })
    .await
}

/// Delete a single metric value by ID. Returns `true` if a row was actually deleted.
pub async fn delete(db: &Db, value_id: i64) -> Result<bool, DbError> {
    db.exec(move |conn| {
        let count =
            diesel::delete(metric_values::table.find(value_id)).execute(conn)?;
        Ok(count > 0)
    })
    .await
}

/// Delete all metric values for a given stock. Returns the number of deleted rows.
pub async fn delete_by_stock(db: &Db, stock_id: i32) -> Result<usize, DbError> {
    db.exec(move |conn| {
        let count = diesel::delete(
            metric_values::table.filter(metric_values::stock_id.eq(stock_id)),
        )
        .execute(conn)?;
        Ok(count)
    })
    .await
}
