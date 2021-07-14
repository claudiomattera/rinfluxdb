// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

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
