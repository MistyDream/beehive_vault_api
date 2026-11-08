use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    features::{
        accounts::AccountKind,
        transactions::domain::{
            TransactionAmount, TransactionAmounts, TransactionEffect, TransactionLabel,
            TransactionNature, TransactionNominalAmount,
        },
    },
    types::TransactionId,
};

use super::reader::CsvReader;

pub(super) const MAX_CSV_BYTES: usize = 1024 * 1024;
pub(super) const MAX_CSV_ROWS: usize = 5_000;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Delimiter {
    Comma,
    Semicolon,
    Tab,
}

impl Delimiter {
    fn character(&self) -> char {
        match self {
            Self::Comma => ',',
            Self::Semicolon => ';',
            Self::Tab => '\t',
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DateFormat {
    YearMonthDay,
    DayMonthYear,
    MonthDayYear,
}

impl DateFormat {
    fn pattern(&self) -> &'static str {
        match self {
            Self::YearMonthDay => "%Y-%m-%d",
            Self::DayMonthYear => "%d/%m/%Y",
            Self::MonthDayYear => "%m/%d/%Y",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DecimalSeparator {
    Dot,
    Comma,
}

impl DecimalSeparator {
    fn character(&self) -> char {
        match self {
            Self::Dot => '.',
            Self::Comma => ',',
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CsvColumns {
    pub booking_date: String,
    pub label: String,
    pub amount: String,
}

impl CsvColumns {
    fn mappings(&self) -> [(Column, &str); 3] {
        [
            (Column::BookingDate, self.booking_date.trim()),
            (Column::Label, self.label.trim()),
            (Column::Amount, self.amount.trim()),
        ]
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CsvFormat {
    pub delimiter: Delimiter,
    pub date_format: DateFormat,
    pub decimal_separator: DecimalSeparator,
    pub columns: CsvColumns,
}

impl CsvFormat {
    pub fn validate(&self) -> Result<(), PreviewError> {
        let mut names = HashSet::new();
        for (column, name) in self.columns.mappings() {
            if name.is_empty() {
                return Err(PreviewError::EmptyColumnMapping(column));
            }
            if !names.insert(name) {
                return Err(PreviewError::DuplicateColumnMapping(column));
            }
        }
        Ok(())
    }

    fn parse_date(&self, value: &str, today: NaiveDate) -> Result<NaiveDate, RowError> {
        let value = value.trim();
        let pattern = self.date_format.pattern();
        let date = NaiveDate::parse_from_str(value, pattern).map_err(|_| RowError::InvalidDate)?;
        if !(1..=9999).contains(&date.year()) || date.format(pattern).to_string() != value {
            return Err(RowError::InvalidDate);
        }
        if date > today {
            return Err(RowError::FutureDate);
        }
        Ok(date)
    }

    fn parse_amount(&self, value: &str) -> Result<Decimal, RowError> {
        let value = value.trim();
        let unsigned = value.strip_prefix(['-', '+']).unwrap_or(value);
        let separator = self.decimal_separator.character();
        let valid_digits =
            |digits: &str| !digits.is_empty() && digits.bytes().all(|digit| digit.is_ascii_digit());
        let valid = match unsigned.split_once(separator) {
            Some((integer, fraction)) => {
                valid_digits(integer) && valid_digits(fraction) && fraction.len() <= 4
            }
            None => valid_digits(unsigned),
        };
        if !valid {
            return Err(RowError::InvalidAmount);
        }

        let canonical = value.replace(separator, ".");
        let amount = Decimal::from_str_exact(&canonical).map_err(|_| RowError::InvalidAmount)?;
        TransactionNominalAmount::new(amount.abs()).map_err(|_| RowError::InvalidAmount)?;
        Ok(amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Column {
    BookingDate,
    Label,
    Amount,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum PreviewError {
    EmptyColumnMapping(Column),
    DuplicateColumnMapping(Column),
    MissingHeader,
    InvalidHeader,
    MissingColumn(Column),
    InvalidSyntax(usize),
    TooManyRows,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum RowError {
    ColumnCountMismatch,
    InvalidDate,
    FutureDate,
    InvalidLabel,
    InvalidAmount,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PreviewValues {
    pub booking_date: NaiveDate,
    pub label: TransactionLabel,
    pub signed_amount: Decimal,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PreviewRow {
    pub line: usize,
    pub source_range: Range<usize>,
    pub result: Result<PreviewValues, Vec<RowError>>,
}

pub(super) struct RowSelection {
    pub line: i32,
    pub nature: ImportNature,
    pub effect: TransactionEffect,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ImportNature {
    Income,
    Expense,
}

impl From<ImportNature> for TransactionNature {
    fn from(nature: ImportNature) -> Self {
        match nature {
            ImportNature::Income => Self::Income,
            ImportNature::Expense => Self::Expense,
        }
    }
}

pub(super) struct SelectedRow<'a> {
    pub selection: RowSelection,
    pub values: PreviewValues,
    pub source_record: &'a str,
}

impl SelectedRow<'_> {
    pub fn account_amount(&self, account_kind: AccountKind) -> TransactionAmount {
        let nominal = TransactionNominalAmount::new(self.values.signed_amount.abs())
            .expect("the preview validated the nominal amount");
        TransactionAmounts::from_input(
            self.selection.nature.into(),
            self.selection.effect,
            nominal,
            account_kind,
        )
        .expect("CSV selections are restricted to income and expense")
        .account
    }
}

pub(super) struct ImportedRow {
    pub line: i32,
    pub status: ImportStatus,
    pub transaction_id: TransactionId,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ImportStatus {
    Imported,
    Duplicate,
}

pub(super) fn preview_rows(
    csv: &str,
    format: &CsvFormat,
    today: NaiveDate,
) -> Result<Vec<PreviewRow>, PreviewError> {
    let mut reader = CsvReader::new(csv, format.delimiter.character());
    let header = reader
        .next_record()
        .map_err(PreviewError::InvalidSyntax)?
        .ok_or(PreviewError::MissingHeader)?;
    let mut headers = HashMap::new();
    for (index, name) in header.fields.iter().map(|field| field.trim()).enumerate() {
        if name.is_empty() || headers.insert(name, index).is_some() {
            return Err(PreviewError::InvalidHeader);
        }
    }
    let lookup = |(column, name)| {
        headers
            .get(name)
            .copied()
            .ok_or(PreviewError::MissingColumn(column))
    };
    let [booking_date, label, amount] = format.columns.mappings();
    let (booking_date, label, amount) = (lookup(booking_date)?, lookup(label)?, lookup(amount)?);
    let mut rows = Vec::new();

    while let Some(record) = reader.next_record().map_err(PreviewError::InvalidSyntax)? {
        if rows.len() == MAX_CSV_ROWS {
            return Err(PreviewError::TooManyRows);
        }
        let result = if record.fields.len() == header.fields.len() {
            let date = format.parse_date(&record.fields[booking_date], today);
            let label =
                TransactionLabel::new(&record.fields[label]).map_err(|_| RowError::InvalidLabel);
            let amount = format.parse_amount(&record.fields[amount]);
            match (date, label, amount) {
                (Ok(booking_date), Ok(label), Ok(signed_amount)) => Ok(PreviewValues {
                    booking_date,
                    label,
                    signed_amount,
                }),
                (date, label, amount) => Err([date.err(), label.err(), amount.err()]
                    .into_iter()
                    .flatten()
                    .collect()),
            }
        } else {
            Err(vec![RowError::ColumnCountMismatch])
        };
        rows.push(PreviewRow {
            line: record.line,
            source_range: record.source_range,
            result,
        });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format() -> CsvFormat {
        CsvFormat {
            delimiter: Delimiter::Semicolon,
            date_format: DateFormat::DayMonthYear,
            decimal_separator: DecimalSeparator::Comma,
            columns: CsvColumns {
                booking_date: "Date".to_owned(),
                label: "Memo".to_owned(),
                amount: "Amount".to_owned(),
            },
        }
    }

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap()
    }

    #[test]
    fn preview_keeps_the_physical_start_line_and_normalizes_exact_values() {
        let csv = "\u{feff}Amount; Date ;Memo;Unused\r\n\r\n\
            -9999999999999999,9999;01/09/2026;\"Purchase; \"\"quoted\"\"\r\nreference\";ignored\r\n\
            +42,50;31/08/2026;Deposit;ignored";
        let rows = preview_rows(csv, &format(), today()).unwrap();
        let first = rows[0].result.as_ref().unwrap();
        assert_eq!(rows[0].line, 3);
        assert_eq!(first.booking_date, today());
        assert_eq!(first.label.as_str(), "Purchase; \"quoted\"\r\nreference");
        assert_eq!(first.signed_amount.to_string(), "-9999999999999999.9999");
        assert_eq!(
            &csv[rows[0].source_range.clone()],
            "-9999999999999999,9999;01/09/2026;\"Purchase; \"\"quoted\"\"\r\nreference\";ignored"
        );
        assert_eq!(
            &csv[rows[1].source_range.clone()],
            "+42,50;31/08/2026;Deposit;ignored"
        );
        assert_eq!(rows[1].line, 5);
        assert_eq!(
            rows[1].result.as_ref().unwrap().signed_amount.to_string(),
            "42.50"
        );
    }

    #[test]
    fn formats_are_explicit_and_amounts_cannot_be_rounded_or_zero() {
        let mut format = format();
        format.delimiter = Delimiter::Tab;
        format.date_format = DateFormat::MonthDayYear;
        format.decimal_separator = DecimalSeparator::Dot;
        let csv = "Date\tMemo\tAmount\n08/31/2026\tDeposit\t5.0001";
        let rows = preview_rows(csv, &format, today()).unwrap();
        let values = rows[0].result.as_ref().unwrap();
        assert_eq!(values.booking_date.to_string(), "2026-08-31");
        assert_eq!(values.signed_amount.to_string(), "5.0001");

        for invalid in [
            "0",
            "-0.0000",
            "1.00001",
            "10000000000000000",
            "1,25",
            "1 234.56",
            "1e3",
            ".5",
            "1.",
            "--1",
        ] {
            assert_eq!(
                format.parse_amount(invalid),
                Err(RowError::InvalidAmount),
                "{invalid}"
            );
        }
        for invalid in ["8/31/2026", "31/08/2026", "02/29/2025", "08/31/0000"] {
            assert_eq!(
                format.parse_date(invalid, today()),
                Err(RowError::InvalidDate),
                "{invalid}"
            );
        }
    }

    #[test]
    fn ambiguous_csv_structure_fails_instead_of_guessing() {
        for invalid in [
            "Date;Memo;Amount\n01/09/2026;\"Unclosed;3",
            "Date;Memo;Amount\n01/09/2026;\"Closed\" suffix;3",
            "Date;Memo;Amount\n01/09/2026;Embedded\"quote;3",
            "Date;Memo;Amount\n01/09/2026;Unexpected\0;3",
            "Date;Memo;Amount\n01/09/2026;Bare carriage return;3\r",
        ] {
            assert_eq!(
                preview_rows(invalid, &format(), today()),
                Err(PreviewError::InvalidSyntax(2))
            );
        }
    }
}
