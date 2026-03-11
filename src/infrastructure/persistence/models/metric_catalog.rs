use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::market::enums::{MetricCategory, MetricDataType};
use crate::domain::market::metric_catalog::MetricCatalog;
use crate::infrastructure::persistence::error::DbError;
use crate::schema::metrics_catalog;

#[derive(Queryable, Selectable)]
#[diesel(table_name = metrics_catalog)]
pub struct MetricCatalogRow {
    pub id: i32,
    pub key: String,
    pub name: String,
    pub category: String,
    pub data_type: String,
    pub unit: Option<String>,
    pub frequency: Option<String>,
    pub higher_is_better: bool,
    pub min_plausible: Option<f64>,
    pub max_plausible: Option<f64>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = metrics_catalog)]
pub struct NewMetricCatalogRow<'a> {
    pub key: &'a str,
    pub name: &'a str,
    pub category: &'a str,
    pub data_type: &'a str,
    pub unit: Option<&'a str>,
    pub frequency: Option<&'a str>,
    pub higher_is_better: bool,
    pub min_plausible: Option<f64>,
    pub max_plausible: Option<f64>,
    pub notes: Option<&'a str>,
}

impl TryFrom<MetricCatalogRow> for MetricCatalog {
    type Error = DbError;

    fn try_from(row: MetricCatalogRow) -> Result<Self, Self::Error> {
        Ok(MetricCatalog {
            id: row.id,
            key: row.key,
            name: row.name,
            category: MetricCategory::try_from(row.category.as_str())
                .map_err(DbError::Conversion)?,
            data_type: MetricDataType::try_from(row.data_type.as_str())
                .map_err(DbError::Conversion)?,
            unit: row.unit,
            frequency: row.frequency,
            higher_is_better: row.higher_is_better,
            min_plausible: row.min_plausible,
            max_plausible: row.max_plausible,
            notes: row.notes,
            updated_at: row.updated_at,
        })
    }
}
