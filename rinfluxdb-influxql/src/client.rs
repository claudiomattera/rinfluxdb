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
    /// Error occurred within the Reqwest library
    #[error("Reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    /// Error occurred while parsing a URL
    #[error("URL parse error")]
    UrlError(#[from] url::ParseError),

    /// Error occurred while parsing format
    #[error("Format parse error")]
    FormatError(#[from] ResponseError),

    /// The server returned an empty statement
    #[error("Empty statement")]
    EmptyError,

    /// The server returned an result without tags when tags were expected
    #[error("Missing tags")]
    ExpectedTagsError,

    /// An expected tag was missing
    #[error("Missing tag \"{0}\"")]
    ExpectedTagError(String),
}
