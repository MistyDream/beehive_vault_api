use std::fmt;

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::pagination::PaginationError;

const PROBLEM_TYPE_PREFIX: &str = "urn:beehive-vault:problem:";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemKind {
    InvalidRequest,
    ValidationError,
    UnsupportedMediaType,
    PayloadTooLarge,
    RouteNotFound,
    MethodNotAllowed,
    HouseholdNotFound,
    InstitutionNotFound,
    AccountNotFound,
    CategoryNotFound,
    TransactionNotFound,
    TransferNotFound,
    DuplicateInstitutionName,
    DuplicateCategoryName,
    DuplicateBalanceDate,
    AccountKindChangeForbidden,
    TransferMovementUpdateForbidden,
    TransferMovementDeleteForbidden,
    ImportedTransactionFieldsImmutable,
    InternalError,
}

impl ProblemKind {
    pub const fn type_name(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid-request",
            Self::ValidationError => "validation-error",
            Self::UnsupportedMediaType => "unsupported-media-type",
            Self::PayloadTooLarge => "payload-too-large",
            Self::RouteNotFound => "route-not-found",
            Self::MethodNotAllowed => "method-not-allowed",
            Self::HouseholdNotFound => "household-not-found",
            Self::InstitutionNotFound => "institution-not-found",
            Self::AccountNotFound => "account-not-found",
            Self::CategoryNotFound => "category-not-found",
            Self::TransactionNotFound => "transaction-not-found",
            Self::TransferNotFound => "transfer-not-found",
            Self::DuplicateInstitutionName => "duplicate-institution-name",
            Self::DuplicateCategoryName => "duplicate-category-name",
            Self::DuplicateBalanceDate => "duplicate-balance-date",
            Self::AccountKindChangeForbidden => "account-kind-change-forbidden",
            Self::TransferMovementUpdateForbidden => "transfer-movement-update-forbidden",
            Self::TransferMovementDeleteForbidden => "transfer-movement-delete-forbidden",
            Self::ImportedTransactionFieldsImmutable => "imported-transaction-fields-immutable",
            Self::InternalError => "internal-error",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::ValidationError => "validation_error",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::PayloadTooLarge => "payload_too_large",
            Self::RouteNotFound => "route_not_found",
            Self::MethodNotAllowed => "method_not_allowed",
            Self::HouseholdNotFound => "household_not_found",
            Self::InstitutionNotFound => "institution_not_found",
            Self::AccountNotFound => "account_not_found",
            Self::CategoryNotFound => "category_not_found",
            Self::TransactionNotFound => "transaction_not_found",
            Self::TransferNotFound => "transfer_not_found",
            Self::DuplicateInstitutionName => "duplicate_institution_name",
            Self::DuplicateCategoryName => "duplicate_category_name",
            Self::DuplicateBalanceDate => "duplicate_balance_date",
            Self::AccountKindChangeForbidden => "account_kind_change_forbidden",
            Self::TransferMovementUpdateForbidden => "transfer_movement_update_forbidden",
            Self::TransferMovementDeleteForbidden => "transfer_movement_delete_forbidden",
            Self::ImportedTransactionFieldsImmutable => "imported_transaction_fields_immutable",
            Self::InternalError => "internal_error",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::InvalidRequest => "Invalid request",
            Self::ValidationError => "Request validation failed",
            Self::UnsupportedMediaType => "Unsupported media type",
            Self::PayloadTooLarge => "Request payload too large",
            Self::RouteNotFound => "Route not found",
            Self::MethodNotAllowed => "Method not allowed",
            Self::HouseholdNotFound => "Household not found",
            Self::InstitutionNotFound => "Institution not found",
            Self::AccountNotFound => "Account not found",
            Self::CategoryNotFound => "Category not found",
            Self::TransactionNotFound => "Transaction not found",
            Self::TransferNotFound => "Transfer not found",
            Self::DuplicateInstitutionName => "Institution name already exists",
            Self::DuplicateCategoryName => "Category name already exists",
            Self::DuplicateBalanceDate => "Balance date already exists",
            Self::AccountKindChangeForbidden => "Account kind change forbidden",
            Self::TransferMovementUpdateForbidden => "Transfer movement update forbidden",
            Self::TransferMovementDeleteForbidden => "Transfer movement deletion forbidden",
            Self::ImportedTransactionFieldsImmutable => "Imported transaction fields are immutable",
            Self::InternalError => "Internal server error",
        }
    }

    pub const fn status(self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::ValidationError => StatusCode::UNPROCESSABLE_ENTITY,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::RouteNotFound
            | Self::HouseholdNotFound
            | Self::InstitutionNotFound
            | Self::AccountNotFound
            | Self::CategoryNotFound
            | Self::TransactionNotFound
            | Self::TransferNotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            Self::DuplicateInstitutionName
            | Self::DuplicateCategoryName
            | Self::DuplicateBalanceDate
            | Self::AccountKindChangeForbidden
            | Self::TransferMovementUpdateForbidden
            | Self::TransferMovementDeleteForbidden
            | Self::ImportedTransactionFieldsImmutable => StatusCode::CONFLICT,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvalidParameterLocation {
    Body,
    Path,
    Query,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidParameter {
    location: InvalidParameterLocation,
    pointer: String,
    code: &'static str,
    detail: String,
}

impl InvalidParameter {
    pub fn new(
        location: InvalidParameterLocation,
        pointer: impl Into<String>,
        code: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            location,
            pointer: pointer.into(),
            code,
            detail: detail.into(),
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    kind: ProblemKind,
    detail: Option<String>,
    errors: Vec<InvalidParameter>,
}

impl ApiError {
    pub fn new(kind: ProblemKind) -> Self {
        Self {
            kind,
            detail: None,
            errors: Vec::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_errors(mut self, errors: Vec<InvalidParameter>) -> Self {
        self.errors = errors;
        self
    }

    pub fn validation(error: InvalidParameter) -> Self {
        Self::new(ProblemKind::ValidationError)
            .with_detail("One or more request values are invalid.")
            .with_errors(vec![error])
    }

    pub fn body_validation(
        pointer: impl Into<String>,
        code: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self::validation(InvalidParameter::new(
            InvalidParameterLocation::Body,
            pointer,
            code,
            detail,
        ))
    }

    pub fn query_validation(
        pointer: impl Into<String>,
        code: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self::validation(InvalidParameter::new(
            InvalidParameterLocation::Query,
            pointer,
            code,
            detail,
        ))
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.kind.title())
    }
}

impl std::error::Error for ApiError {}

impl From<PaginationError> for ApiError {
    fn from(error: PaginationError) -> Self {
        let (pointer, code) = match error {
            PaginationError::InvalidLimit => ("#/limit", "invalid_range"),
            PaginationError::InvalidPage | PaginationError::PageTooLarge => {
                ("#/page", "invalid_range")
            }
        };
        Self::query_validation(pointer, code, error.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        let database_error = error.as_database_error();
        let constraint = database_error.and_then(|database_error| database_error.constraint());
        let known_problem = match constraint {
            Some("institutions_active_name_unique") => Some(ProblemKind::DuplicateInstitutionName),
            Some("categories_active_name_unique") => Some(ProblemKind::DuplicateCategoryName),
            Some("account_balance_snapshots_account_id_balance_date_key") => {
                Some(ProblemKind::DuplicateBalanceDate)
            }
            _ => None,
        };

        if let Some(kind) = known_problem {
            return Self::new(kind);
        }

        let database_code = database_error
            .and_then(|database_error| database_error.code())
            .map(|code| code.into_owned());
        if database_code.as_deref() == Some("23503") {
            return Self::validation(InvalidParameter::new(
                InvalidParameterLocation::Body,
                "#/",
                "invalid_reference",
                "A referenced resource does not exist.",
            ));
        }

        tracing::error!(%error, "database operation failed");
        Self::new(ProblemKind::InternalError)
            .with_detail("An unexpected error occurred while processing the request.")
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDetails {
    #[serde(rename = "type")]
    type_uri: String,
    title: &'static str,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    code: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<InvalidParameter>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.kind.status();
        let problem = ProblemDetails {
            type_uri: format!("{PROBLEM_TYPE_PREFIX}{}", self.kind.type_name()),
            title: self.kind.title(),
            status: status.as_u16(),
            detail: self.detail,
            code: self.kind.code(),
            errors: self.errors,
        };
        let mut response = (status, Json(problem)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

pub fn required_text(value: String, field: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return Err(ApiError::body_validation(
            format!("#/{field}"),
            "invalid_length",
            format!("{field} must contain between 1 and 100 characters"),
        ));
    }
    Ok(trimmed.to_owned())
}
