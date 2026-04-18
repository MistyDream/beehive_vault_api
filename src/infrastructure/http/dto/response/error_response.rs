use serde::Serialize;

/// RFC 9457 — Problem Details for HTTP APIs
#[derive(Serialize)]
pub struct ProblemDetail {
    /// URI reference identifying the problem type (use "about:blank" when no specific type)
    #[serde(rename = "type")]
    pub problem_type: String,
    /// Short human-readable summary
    pub title: String,
    /// HTTP status code (must match actual response status)
    pub status: u16,
    /// Human-readable explanation specific to this occurrence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// URI identifying this specific occurrence (e.g. request path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Validation field errors (extension)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<FieldError>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}
