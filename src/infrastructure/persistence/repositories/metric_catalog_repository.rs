//! Repository for the `metrics_catalog` table.
//!
//! Manages metric definitions (name, category, data type, plausibility bounds, etc.).
//! Each metric is uniquely identified by its `key` (e.g. `"pe_ratio"`, `"roe"`).

use diesel::prelude::*;

use crate::application::error::AppError;
use crate::domain::scoring::metric_catalog::MetricCatalog;
use crate::infrastructure::persistence::Db;
use crate::infrastructure::persistence::models::metric_catalog::{
    MetricCatalogRow, NewMetricCatalogRow,
};
use crate::schema::metrics_catalog;

#[derive(Clone)]
pub struct PgMetricCatalogRepository {
    db: Db,
}

impl PgMetricCatalogRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Fetch a metric definition by its primary key.
    pub async fn find_by_id(&self, catalog_id: i32) -> Result<MetricCatalog, AppError> {
        self.db
            .exec(move |conn| {
                let row = metrics_catalog::table
                    .find(catalog_id)
                    .select(MetricCatalogRow::as_select())
                    .first(conn)?;
                MetricCatalog::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }

    /// Fetch a metric definition by its unique key (e.g. `"pe_ratio"`).
    pub async fn find_by_key(&self, key: String) -> Result<MetricCatalog, AppError> {
        self.db
            .exec(move |conn| {
                let row = metrics_catalog::table
                    .filter(metrics_catalog::key.eq(&key))
                    .select(MetricCatalogRow::as_select())
                    .first(conn)?;
                MetricCatalog::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }

    /// List all metric definitions belonging to a given category (e.g. `"valuation"`).
    pub async fn find_by_category(
        &self,
        category: String,
    ) -> Result<Vec<MetricCatalog>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = metrics_catalog::table
                    .filter(metrics_catalog::category.eq(&category))
                    .select(MetricCatalogRow::as_select())
                    .order(metrics_catalog::key.asc())
                    .load(conn)?;
                rows.into_iter().map(MetricCatalog::try_from).collect()
            })
            .await
            .map_err(AppError::from)
    }

    /// List all metric definitions, ordered alphabetically by key.
    pub async fn list_all(&self) -> Result<Vec<MetricCatalog>, AppError> {
        self.db
            .exec(move |conn| {
                let rows = metrics_catalog::table
                    .select(MetricCatalogRow::as_select())
                    .order(metrics_catalog::key.asc())
                    .load(conn)?;
                rows.into_iter().map(MetricCatalog::try_from).collect()
            })
            .await
            .map_err(AppError::from)
    }

    /// Insert a new metric definition and return the created entity.
    pub async fn insert(
        &self,
        new: NewMetricCatalogRow<'static>,
    ) -> Result<MetricCatalog, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::insert_into(metrics_catalog::table)
                    .values(&new)
                    .returning(MetricCatalogRow::as_returning())
                    .get_result(conn)?;
                MetricCatalog::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }

    /// Replace all mutable fields of an existing metric definition.
    /// The `updated_at` timestamp is managed automatically by the DB trigger.
    pub async fn update(
        &self,
        catalog_id: i32,
        new: NewMetricCatalogRow<'static>,
    ) -> Result<MetricCatalog, AppError> {
        self.db
            .exec(move |conn| {
                let row = diesel::update(metrics_catalog::table.find(catalog_id))
                    .set((
                        metrics_catalog::key.eq(new.key),
                        metrics_catalog::name.eq(new.name),
                        metrics_catalog::category.eq(new.category),
                        metrics_catalog::data_type.eq(new.data_type),
                        metrics_catalog::unit.eq(new.unit),
                        metrics_catalog::frequency.eq(new.frequency),
                        metrics_catalog::higher_is_better.eq(new.higher_is_better),
                        metrics_catalog::min_plausible.eq(new.min_plausible),
                        metrics_catalog::max_plausible.eq(new.max_plausible),
                        metrics_catalog::notes.eq(new.notes),
                    ))
                    .returning(MetricCatalogRow::as_returning())
                    .get_result(conn)?;
                MetricCatalog::try_from(row)
            })
            .await
            .map_err(AppError::from)
    }

    /// Delete a metric definition by ID. Returns `true` if a row was actually deleted.
    /// Will fail if `metric_values` rows still reference this metric (FK RESTRICT).
    pub async fn delete(&self, catalog_id: i32) -> Result<bool, AppError> {
        self.db
            .exec(move |conn| {
                let count =
                    diesel::delete(metrics_catalog::table.find(catalog_id)).execute(conn)?;
                Ok(count > 0)
            })
            .await
            .map_err(AppError::from)
    }
}
