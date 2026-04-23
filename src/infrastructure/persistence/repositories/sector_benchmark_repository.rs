//! Repository for the `sector_benchmarks` table.
//!
//! Stores sector/industry median values for financial metrics (e.g. median PER
//! for the "Technology" sector). These benchmarks serve as reference points for
//! relative scoring. Supports upsert for idempotent ingestion from GuruFocus.

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::upsert::excluded;

use crate::application::error::AppError;
use crate::domain::scoring::sector_benchmark::SectorBenchmark;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::sector_benchmark::{
    NewSectorBenchmarkRow, SectorBenchmarkRow,
};
use crate::schema::sector_benchmarks;

#[derive(Clone)]
pub struct PgSectorBenchmarkRepository {
    db: Db,
}

impl PgSectorBenchmarkRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Fetch a single benchmark by its primary key.
    pub async fn find_by_id(&self, benchmark_id: i32) -> Result<SectorBenchmark, AppError> {
        self.db
            .exec(move |conn| {
                let row = sector_benchmarks::table
                    .find(benchmark_id)
                    .select(SectorBenchmarkRow::as_select())
                    .first(conn)?;
                Ok(SectorBenchmark::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch the latest benchmark for a given sector + metric (sector-level, no industry filter).
    /// Returns `AppError::NotFound` if no benchmark exists.
    pub async fn find_latest_by_sector_and_metric(
        &self,
        sector: String,
        metric_key: String,
    ) -> Result<SectorBenchmark, AppError> {
        self.db
            .exec(move |conn| {
                let row = sector_benchmarks::table
                    .filter(
                        sector_benchmarks::sector
                            .eq(&sector)
                            .and(sector_benchmarks::industry.is_null())
                            .and(sector_benchmarks::metric_key.eq(&metric_key)),
                    )
                    .select(SectorBenchmarkRow::as_select())
                    .order(sector_benchmarks::period_end.desc())
                    .first(conn)?;
                Ok(SectorBenchmark::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch the latest benchmark for a given sector + industry + metric.
    /// Returns `AppError::NotFound` if no benchmark exists.
    pub async fn find_latest_by_industry_and_metric(
        &self,
        sector: String,
        industry: String,
        metric_key: String,
    ) -> Result<SectorBenchmark, AppError> {
        self.db
            .exec(move |conn| {
                let row = sector_benchmarks::table
                    .filter(
                        sector_benchmarks::sector
                            .eq(&sector)
                            .and(sector_benchmarks::industry.eq(&industry))
                            .and(sector_benchmarks::metric_key.eq(&metric_key)),
                    )
                    .select(SectorBenchmarkRow::as_select())
                    .order(sector_benchmarks::period_end.desc())
                    .first(conn)?;
                Ok(SectorBenchmark::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch all benchmarks for a sector (both sector-level and industry-level), most recent first.
    pub async fn find_by_sector(
        &self,
        sector: String,
    ) -> Result<Vec<SectorBenchmark>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = sector_benchmarks::table
                    .filter(sector_benchmarks::sector.eq(&sector))
                    .select(SectorBenchmarkRow::as_select())
                    .order(sector_benchmarks::period_end.desc())
                    .load(conn)?;
                Ok(rows.into_iter().map(SectorBenchmark::from).collect())
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch all benchmarks for a specific metric across all sectors, most recent first.
    pub async fn find_by_metric(
        &self,
        metric_key: String,
    ) -> Result<Vec<SectorBenchmark>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = sector_benchmarks::table
                    .filter(sector_benchmarks::metric_key.eq(&metric_key))
                    .select(SectorBenchmarkRow::as_select())
                    .order(sector_benchmarks::period_end.desc())
                    .load(conn)?;
                Ok(rows.into_iter().map(SectorBenchmark::from).collect())
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch benchmarks for a sector + metric within a date range.
    pub async fn find_by_sector_metric_and_period_range(
        &self,
        sector: String,
        metric_key: String,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<SectorBenchmark>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = sector_benchmarks::table
                    .filter(
                        sector_benchmarks::sector
                            .eq(&sector)
                            .and(sector_benchmarks::metric_key.eq(&metric_key))
                            .and(sector_benchmarks::period_end.between(from, to)),
                    )
                    .select(SectorBenchmarkRow::as_select())
                    .order(sector_benchmarks::period_end.desc())
                    .load(conn)?;
                Ok(rows.into_iter().map(SectorBenchmark::from).collect())
            })
            .await
            .map_err(AppError::from)
    }

    /// Insert a new benchmark. Fails on duplicate `(sector, industry, metric_key, period_end)`.
    /// Prefer [`upsert`] for idempotent ingestion.
    pub async fn insert(
        &self,
        new: NewSectorBenchmarkRow<'static>,
    ) -> Result<SectorBenchmark, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(sector_benchmarks::table)
                    .values(&new)
                    .returning(SectorBenchmarkRow::as_returning())
                    .get_result(conn)?;
                Ok(SectorBenchmark::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    /// Upsert a single benchmark on the UNIQUE(sector, industry, metric_key, period_end) constraint.
    /// On conflict, updates value, source, and fetched_at.
    pub async fn upsert(
        &self,
        new: NewSectorBenchmarkRow<'static>,
    ) -> Result<SectorBenchmark, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(sector_benchmarks::table)
                    .values(&new)
                    .on_conflict((
                        sector_benchmarks::sector,
                        sector_benchmarks::industry,
                        sector_benchmarks::metric_key,
                        sector_benchmarks::period_end,
                    ))
                    .do_update()
                    .set((
                        sector_benchmarks::value.eq(excluded(sector_benchmarks::value)),
                        sector_benchmarks::source.eq(excluded(sector_benchmarks::source)),
                        sector_benchmarks::fetched_at
                            .eq(excluded(sector_benchmarks::fetched_at)),
                    ))
                    .returning(SectorBenchmarkRow::as_returning())
                    .get_result(conn)?;
                Ok(SectorBenchmark::from(row))
            })
            .await
            .map_err(AppError::from)
    }

    /// Bulk upsert benchmarks. Returns all upserted rows.
    pub async fn upsert_many(
        &self,
        rows: Vec<NewSectorBenchmarkRow<'static>>,
    ) -> Result<Vec<SectorBenchmark>, AppError> {
        self.db
            .exec(move |conn| {
                let inserted = diesel::insert_into(sector_benchmarks::table)
                    .values(&rows)
                    .on_conflict((
                        sector_benchmarks::sector,
                        sector_benchmarks::industry,
                        sector_benchmarks::metric_key,
                        sector_benchmarks::period_end,
                    ))
                    .do_update()
                    .set((
                        sector_benchmarks::value.eq(excluded(sector_benchmarks::value)),
                        sector_benchmarks::source.eq(excluded(sector_benchmarks::source)),
                        sector_benchmarks::fetched_at
                            .eq(excluded(sector_benchmarks::fetched_at)),
                    ))
                    .returning(SectorBenchmarkRow::as_returning())
                    .get_results(conn)?;
                Ok(inserted.into_iter().map(SectorBenchmark::from).collect())
            })
            .await
            .map_err(AppError::from)
    }

    /// Delete a single benchmark by ID. Returns `true` if a row was actually deleted.
    pub async fn delete(&self, benchmark_id: i32) -> Result<bool, AppError> {
        self.db
            .exec(move |conn| {
                let count =
                    diesel::delete(sector_benchmarks::table.find(benchmark_id)).execute(conn)?;
                Ok(count > 0)
            })
            .await
            .map_err(AppError::from)
    }

    /// Delete all benchmarks for a given sector. Returns the number of deleted rows.
    pub async fn delete_by_sector(&self, sector: String) -> Result<usize, AppError> {
        self.db
            .exec(move |conn| {
                let count = diesel::delete(
                    sector_benchmarks::table
                        .filter(sector_benchmarks::sector.eq(&sector)),
                )
                .execute(conn)?;
                Ok(count)
            })
            .await
            .map_err(AppError::from)
    }
}
