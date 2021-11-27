// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use std::collections::HashMap;
use std::convert::TryFrom;

use tracing::*;

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use reqwest::Client as ReqwestClient;
use reqwest::ClientBuilder as ReqwestClientBuilder;
use reqwest::RequestBuilder as ReqwestRequestBuilder;
use reqwest::Response as ReqwestResponse;

use url::Url;

use chrono::{DateTime, Utc};

use async_trait::async_trait;

use rinfluxdb_types::Value;

use super::ClientError;

use super::super::query::Query;
use super::super::response::{from_str, ResponseError};
use super::super::StatementResult;

/// A client for performing frequent InfluxQL queries in a convenient way
///
/// ```.no_run
/// use std::collections::HashMap;
/// use url::Url;
/// use rinfluxdb_influxql::QueryBuilder;
/// use rinfluxdb_influxql::r#async::Client;
/// use rinfluxdb_dataframe::DataFrame;
///
/// async_std::task::block_on(async {
/// let client = Client::new(
///     Url::parse("https://example.com/")?,
///     Some(("username", "password")),
/// )?;
///
/// let query = QueryBuilder::from("indoor_environment")
///     .database("house")
///     .field("temperature")
///     .field("humidity")
///     .build();
/// let dataframe: DataFrame = client.fetch_dataframe(query).await?;
/// println!("{}", dataframe);
///
/// let query = QueryBuilder::from("indoor_environment")
///     .database("house")
///     .field("temperature")
///     .field("humidity")
///     .group_by("room")
///     .build();
/// let tagged_dataframes: HashMap<String, DataFrame> = client.fetch_dataframes_by_tag(query, "room").await?;
/// for (tag, dataframe) in tagged_dataframes {
///     println!("{}: {}", tag, dataframe);
/// }
/// # Ok::<(), rinfluxdb_influxql::ClientError>(())
/// # })?;
/// # Ok::<(), rinfluxdb_influxql::ClientError>(())
/// ```
#[derive(Debug)]
pub struct Client {
    client: ReqwestClient,
    base_url: Url,
    credentials: Option<(String, String)>,
}

impl Client {
    /// Create a new client to an InfluxDB server
    ///
    /// Parameter `credentials` can be used to provide username and password if
    /// the server requires authentication.
    pub fn new<T, S>(
        base_url: Url,
        credentials: Option<(T, S)>,
    ) -> Result<Self, ClientError>
    where
        T: Into<String>,
        S: Into<String>,
    {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let client = ReqwestClientBuilder::new()
            .default_headers(headers)
            .build()?;

        let credentials = credentials
            .map(|(username, password)| (username.into(), password.into()));

        Ok(Self {
            client,
            base_url,
            credentials,
        })
    }

    /// Query the server for a single dataframe
    ///
    /// This function assumes a single statement is returned, and that such
    /// statement contains a single dataframe. Everything else is ignored.
    ///
    /// [`ClientError::EmptyError`](ClientError::EmptyError) is returned if the
    /// response does not contain
    /// dataframes.
    #[instrument(
        name = "Fetching dataframe",
        skip(self),
    )]
    pub async fn fetch_dataframe<DF, E>(
        &self,
        query: Query,
    ) -> Result<DF, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>,
    {
        let statement_results = self.fetch_readings_from_database(query, None::<String>).await?;
        let statement_result = statement_results
            .into_iter()
            .next()
            .ok_or(ClientError::EmptyError)?;
        let dataframes = statement_result?;
        let (dataframe, _tags) = dataframes
            .into_iter()
            .next()
            .ok_or(ClientError::EmptyError)?;
        Ok(dataframe)
    }

    /// Query the server for dataframes grouped by a single tag
    ///
    /// This function assumes a single statement is returned, and that such
    /// statement contains multiple dataframe with the specified tag.
    /// Everything else is ignored.
    ///
    /// [`ClientError::EmptyError`](ClientError::EmptyError) is returned if the
    /// response does not contain dataframes.
    /// [`ClientError::ExpectedTagsError`](ClientError::ExpectedTagsError) is
    /// returned if the response does not contain tagged dataframes.
    /// [`ClientError::ExpectedTagError`](ClientError::ExpectedTagError) is
    /// returned if the response contains tagged dataframes, but the specified
    /// tag is missing.
    #[instrument(
        name = "Fetching dataframe by tag",
        skip(self),
    )]
    pub async fn fetch_dataframes_by_tag<DF, E>(
        &self,
        query: Query,
        tag: &str,
    ) -> Result<HashMap<String, DF>, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>,
    {
        let statement_results = self.fetch_readings_from_database(query, None::<String>).await?;
        let statement_result = statement_results
            .into_iter()
            .next()
            .ok_or(ClientError::EmptyError)?;
        let dataframes = statement_result?;
        dataframes
            .into_iter()
            .map(|(dataframe, tags)| {
                let tags = tags.ok_or(ClientError::ExpectedTagsError)?;
                let tag_value = tags
                    .get(tag)
                    .ok_or_else(|| ClientError::ExpectedTagError(tag.to_owned()))?;
                Ok((tag_value.to_owned(), dataframe))
            })
            .collect()
    }

    pub async fn fetch_readings<DF, E>(
        &self,
        query: Query,
    ) -> Result<Vec<StatementResult<DF>>, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>,
    {
        self.fetch_readings_from_database(query, None::<String>).await
    }

    pub async fn fetch_readings_from_database<DF, E, T>(
        &self,
        query: Query,
        database: Option<T>,
    ) -> Result<Vec<StatementResult<DF>>, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>,
        T: Into<String>,
    {
        let mut influxql_request = self.client
            .influxql(&self.base_url)?
            .query(query);
        if let Some(database) = database {
            influxql_request = influxql_request.database(database);
        }
        let mut request = influxql_request.into_reqwest_builder();

        if let Some((username, password)) = &self.credentials {
            request = request.basic_auth(username, Some(password));
        }

        let request = request.build()?;

        debug!("Sending request to {}", self.base_url);
        trace!("Request: {:?}", request);

        let response = self.client.execute(request).await?;

        let response = response.error_for_status()?;

        type TaggedDataFrames<DF> = Vec<(DF, Option<HashMap<String, String>>)>;
        let results: Vec<Result<TaggedDataFrames<DF>, ResponseError>> = response.dataframes().await?;
        debug!("Fetched {} statement results", results.len());

        Ok(results)
    }
}

/// A trait to obtain a prepared InfluxQL request builder from [Reqwest clients](reqwest::Client).
///
/// This trait is used to attach an `influxql()` function to [`reqwest::Client`](reqwest::Client).
///
/// ```no_run
/// # use url::Url;
/// # use rinfluxdb_influxql::Query;
/// // Bring into scope the trait implementation
/// use rinfluxdb_influxql::r#async::InfluxqlClientWrapper;
///
/// async_std::task::block_on(async {
/// // Create Reqwest client
/// let client = reqwest::Client::new();
///
/// // Create InfluxQL request
/// let base_url = Url::parse("https://example.com")?;
/// let mut builder = client
///     // (this is a function added by the trait above)
///     .influxql(&base_url)?
///     // (this functions are defined on influxql::RequestBuilder)
///     .database("house")
///     .query(Query::new("SELECT temperature FROM indoor_temperature"))
///     // (this function returns a regular Reqwest builder)
///     .into_reqwest_builder();
///
/// // Now this is a regular Reqwest builder, and can be customized as usual
/// if let Some((username, password)) = Some(("username", "password")) {
///     builder = builder.basic_auth(username, Some(password));
/// }
///
/// // Create a request from the builder
/// let request = builder.build()?;
///
/// // Execute the request through Reqwest and obtain a response
/// let response = client.execute(request).await?;
///
/// # Ok::<(), rinfluxdb_influxql::ClientError>(())
/// # })?;
/// # Ok::<(), rinfluxdb_influxql::ClientError>(())
/// ```
pub trait InfluxqlClientWrapper {
    /// Create an InfluxQL request builder
    ///
    /// The request will point to the InfluxDB instance available at
    /// `base_url`.
    /// In particular, it will send a POST request to `base_url + "/query"`.
    fn influxql(&self, base_url: &Url) -> Result<RequestBuilder, ClientError>;
}

impl InfluxqlClientWrapper for ReqwestClient {
    fn influxql(&self, base_url: &Url) -> Result<RequestBuilder, ClientError> {
        let url = base_url.join("/query")?;

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        let builder = self
            .post(url)
            .headers(headers);

        Ok(RequestBuilder::new(builder))
    }
}

/// An extension of [`reqwest::RequestBuilder`](reqwest::RequestBuilder)
/// to build requests to InfluxDB using InfluxQL
///
/// See traits [`InfluxqlClientWrapper`](InfluxqlClientWrapper) and
/// [`InfluxqlResponseWrapper`](InfluxqlResponseWrapper) for an example.
#[derive(Debug)]
pub struct RequestBuilder {
    builder: ReqwestRequestBuilder,
    database: Option<String>,
    query: Option<Query>,
}

impl RequestBuilder {
    fn new(builder: ReqwestRequestBuilder) -> Self {
        Self {
            builder,
            database: None,
            query: None,
        }
    }

    /// Set a database for the request
    pub fn database<T>(mut self, database: T) -> Self
    where
        T: Into<String>,
    {
        self.database = Some(database.into());
        self
    }

    /// Set the query for the request
    pub fn query(mut self, query: Query) -> Self {
        self.query = Some(query);
        self
    }

    /// Convert to a [`reqwest::RequestBuilder`](reqwest::RequestBuilder)
    /// prepared to build requests to InfluxDB using InfluxQL
    pub fn into_reqwest_builder(self) -> ReqwestRequestBuilder {
        let mut params = HashMap::new();
        if let Some(query) = self.query.as_ref() {
            params.insert("q", query.as_ref());
        }
        if let Some(database) = self.database.as_ref() {
            params.insert("db", database.as_ref());
        }

        self.builder
            .form(&params)
    }
}

#[async_trait]
impl InfluxqlResponseWrapper for ReqwestResponse {
    async fn dataframes<DF, E>(self) -> Result<Vec<StatementResult<DF>>, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>,
    {
        let text = self.text().await?;
        let dataframes = from_str(&text)?;
        Ok(dataframes)
    }
}

/// A trait to parse a list of dataframes from [Reqwest responses](reqwest::Response).
///
/// This trait is used to attach a `dataframes()` function to [`reqwest::Response`](reqwest::Response).
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use url::Url;
/// # use rinfluxdb_influxql::{Query, ResponseError};
/// use rinfluxdb_influxql::r#async::InfluxqlClientWrapper;
/// use rinfluxdb_dataframe::DataFrame;
///
/// // Bring into scope the trait implementation
/// use rinfluxdb_influxql::r#async::InfluxqlResponseWrapper;
///
/// async_std::task::block_on(async {
/// // Create Reqwest client
/// let client = reqwest::Client::new();
///
/// // Create InfluxQL request
/// let base_url = Url::parse("https://example.com")?;
/// let mut request = client
///     .influxql(&base_url)?
///     .database("house")
///     .query(Query::new("SELECT temperature FROM indoor_temperature"))
///     .into_reqwest_builder()
///     .build()?;
///
/// // Execute the request through Reqwest and obtain a response
/// let response = client.execute(request).await?;
///
/// // Return an error if response status is not 200
/// // (this is a function from Reqwest's response)
/// let response = response.error_for_status()?;
///
/// // Parse the response from JSON to a list of dataframes
/// // (this is a function added by the trait above)
/// let results: Vec<Result<Vec<(DataFrame, Option<HashMap<String, String>>)>, ResponseError>>
///     = response.dataframes().await?;
///
/// # Ok::<(), rinfluxdb_influxql::ClientError>(())
/// # })?;
/// # Ok::<(), rinfluxdb_influxql::ClientError>(())
/// ```
#[async_trait]
pub trait InfluxqlResponseWrapper {
    /// Return the response body as a list of tagged dataframes
    async fn dataframes<DF, E>(self) -> Result<Vec<StatementResult<DF>>, ClientError>
    where
        DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
        E: Into<ResponseError>;
}
