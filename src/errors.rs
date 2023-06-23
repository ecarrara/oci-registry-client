//! Error representation.

use reqwest;
use std::fmt;

/// A list of errors.
#[derive(serde::Deserialize, Debug)]
pub struct ErrorList {
    errors: Vec<Error>,
}

/// An error.
///
/// Represents an error returned by Image Registry API.
#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
pub struct Error {
    code: String,
    message: String,
    detail: serde_json::Value,
}

/// Details about an error.
#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorDetail {
    r#type: String,
    class: String,
    name: String,
    action: String,
}

/// Error response
///
/// `APIError` is returned when Image Registry API returns an error, otherwise
/// `RequestError` is returned
#[derive(Debug)]
pub enum ErrorResponse {
    APIError(ErrorList),
    RequestError(reqwest::Error),
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::APIError(err) => {
                write!(f, "API error:")?;
                for e in err.errors.iter() {
                    write!(f, "\n  {}: {}", e.code, e.message)?;
                }
                Ok(())
            }
            Self::RequestError(err) => write!(f, "Request error: {}", err),
        }
    }
}

impl std::error::Error for ErrorResponse {}

impl From<reqwest::Error> for ErrorResponse {
    fn from(error: reqwest::Error) -> Self {
        ErrorResponse::RequestError(error)
    }
}
