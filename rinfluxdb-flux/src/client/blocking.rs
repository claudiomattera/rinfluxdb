// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use std::collections::HashMap;
use std::convert::TryFrom;

use tracing::*;

use chrono::{DateTime, Utc};

use reqwest::blocking::Client as ReqwestClient;
use reqwest::blocking::ClientBuilder as ReqwestClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};

use url::Url;

use rinfluxdb_types::Value;

use super::ClientError;

use super::super::query::Query;
use super::super::response::{from_str, ResponseError};

/// A client for performing frequent Flux queries in a convenient way
#[derive(Debug)]
pub struct Client {
    client: ReqwestClient,
    base_url: Url,
    credentials: Option<(String, String)>,
}

impl Client {
    pub fn new(base_url: Url, credentials: Option<(String, String)>) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/csv"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/vnd.flux"));

        let client = ReqwestClientBuilder::new()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            client,
            base_url,
            credentials,
        })
    }

    #[instrument(
        name = "Fetching readings",
        skip(self),
    )]
    pub fn fetch_readings<DF, E>(&self, query: Query) -> Result<DF, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>,
    {
        let url = self.base_url.join("/api/v2/query")?;
        let mut request = self.client
            .post(url);

        if let Some((username, password)) = &self.credentials {
            request = request.basic_auth(username, Some(password));
        }

        request = request.body(query.as_ref().to_owned());

        debug!("Sending request to {}", self.base_url);
        trace!("Request: {:?}", request);

        let response = request.send()?;

        let response = response.error_for_status()?;

        let text = response.text()?;

        let dataframe = from_str(&text)?;

        Ok(dataframe)
    }
}
