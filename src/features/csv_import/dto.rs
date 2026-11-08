use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    types::{AccountId, CurrencyCode},
};

use super::domain::{Column, CsvFormat, PreviewError, PreviewRow, PreviewValues, RowError};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PreviewRequest {
    pub csv: String,
    pub format: CsvFormat,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PreviewResponse {
    pub account_id: AccountId,
    pub currency: CurrencyCode,
    pub rows: Vec<PreviewRowResponse>,
}

#[derive(Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub(super) enum PreviewRowResponse {
    Valid {
        line: usize,
        booking_date: NaiveDate,
        label: String,
        #[serde(with = "rust_decimal::serde::str")]
        signed_amount: Decimal,
    },
    Rejected {
        line: usize,
        errors: Vec<RowErrorResponse>,
    },
}

impl From<PreviewRow> for PreviewRowResponse {
    fn from(row: PreviewRow) -> Self {
        match row.result {
            Ok(PreviewValues {
                booking_date,
                label,
                signed_amount,
            }) => Self::Valid {
                line: row.line,
                booking_date,
                label: label.into_string(),
                signed_amount,
            },
            Err(errors) => Self::Rejected {
                line: row.line,
                errors: errors.into_iter().map(Into::into).collect(),
            },
        }
    }
}

#[derive(Serialize)]
pub(super) struct RowErrorResponse {
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<&'static str>,
}

impl From<RowError> for RowErrorResponse {
    fn from(error: RowError) -> Self {
        let (code, field) = match error {
            RowError::ColumnCountMismatch => ("column_count_mismatch", None),
            RowError::InvalidDate => ("invalid_date", Some("bookingDate")),
            RowError::FutureDate => ("date_in_future", Some("bookingDate")),
            RowError::InvalidLabel => ("invalid_length", Some("label")),
            RowError::InvalidAmount => ("invalid_amount", Some("amount")),
        };
        Self { code, field }
    }
}

impl From<PreviewError> for ApiError {
    fn from(error: PreviewError) -> Self {
        match error {
            PreviewError::EmptyColumnMapping(column) => Self::body_validation(
                column_pointer(column),
                "empty_column_mapping",
                "A column mapping must name a non-empty header.",
            ),
            PreviewError::DuplicateColumnMapping(column) => Self::body_validation(
                column_pointer(column),
                "duplicate_column_mapping",
                "Each field must map to a different header.",
            ),
            PreviewError::MissingHeader => Self::body_validation(
                "#/csv",
                "missing_csv_header",
                "The CSV must contain a header record.",
            ),
            PreviewError::InvalidHeader => Self::body_validation(
                "#/csv",
                "invalid_csv_header",
                "The CSV header names must be non-empty and unique.",
            ),
            PreviewError::MissingColumn(column) => Self::body_validation(
                column_pointer(column),
                "missing_csv_column",
                "The mapped column is not present in the CSV header.",
            ),
            PreviewError::InvalidSyntax(line) => Self::body_validation(
                "#/csv",
                "invalid_csv_syntax",
                format!("The CSV structure is invalid at or near physical line {line}."),
            ),
            PreviewError::TooManyRows => Self::body_validation(
                "#/csv",
                "csv_row_limit",
                "The CSV exceeds the maximum of 5000 data records.",
            ),
        }
    }
}

fn column_pointer(column: Column) -> &'static str {
    match column {
        Column::BookingDate => "#/format/columns/bookingDate",
        Column::Label => "#/format/columns/label",
        Column::Amount => "#/format/columns/amount",
    }
}
