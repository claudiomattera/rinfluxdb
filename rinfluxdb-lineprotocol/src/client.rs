// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use serde::Deserialize;

use serde_json::from_str;

use thiserror::Error;

pub mod r#async;
pub mod blocking;

/// An error occurred during interfacing with an InfluxDB server
#[derive(Error, Debug)]
pub enum ClientError {
    /// Error occurred within the Reqwest library
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    /// Error occurred while parsing a URL
    #[error("URL parse error")]
    UrlError(#[from] url::ParseError),

    /// Specified a field with conflicting type
    #[error("Field type conflict")]
    FieldTypeConflict,

    /// Database was not found
    #[error("Database not found")]
    DatabaseNotFound,

    /// Unknown error
    #[error("Unknown error")]
    Unknown,
}

fn parse_error(text: &str) -> ClientError {
    let response: Result<Response, _> = from_str(text);
    match response {
        Ok(response) => {
            if response.error.starts_with("field type conflict") {
                ClientError::FieldTypeConflict
            } else if response.error.starts_with("database not found") {
                ClientError::DatabaseNotFound
            } else {
                ClientError::Unknown
            }
        }
        Err(_) => ClientError::Unknown,
    }

}

#[derive(Debug, Deserialize)]
struct Response {
    error: String,
}
