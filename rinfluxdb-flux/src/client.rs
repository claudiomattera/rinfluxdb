// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use thiserror::Error;

use super::response::ResponseError;

pub mod r#async;
pub mod blocking;

/// An error occurred during interfacing with an InfluxDB server
#[derive(Error, Debug)]
pub enum ClientError {
    /// Error occurred inside Request library
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    /// Error occurred while parsing a URL
    #[error("URL parse error")]
    UrlError(#[from] url::ParseError),

    /// Error occurred while parsing a datetime
    #[error("Chrono parse error")]
    ParseDatetimeError(#[from] chrono::ParseError),

    /// Error occurred while parsing format
    #[error("Format parse error")]
    ResponseError(#[from] ResponseError),
}
