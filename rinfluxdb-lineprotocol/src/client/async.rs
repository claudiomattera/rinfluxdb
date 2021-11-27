// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use tracing::*;

use reqwest::Client as ReqwestClient;
use reqwest::ClientBuilder as ReqwestClientBuilder;
use reqwest::RequestBuilder as ReqwestRequestBuilder;
use reqwest::Response as ReqwestResponse;

use url::Url;

use async_trait::async_trait;

use super::super::Line;
use super::{parse_error, ClientError};

/// A client for sending data with Influx Line Protocol queries in a convenient
/// way
///
/// ```.no_run
/// use url::Url;
/// use rinfluxdb_lineprotocol::LineBuilder;
/// use rinfluxdb_lineprotocol::r#async::Client;
///
/// # async_std::task::block_on(async {
/// let client = Client::new(
///     Url::parse("https://example.com/")?,
///     Some(("username", "password")),
/// )?;
///
/// let lines = vec![
///     LineBuilder::new("measurement")
///         .insert_field("field", 42.0)
///         .build(),
///     LineBuilder::new("measurement")
///         .insert_field("field", 43.0)
///         .insert_tag("tag", "value")
///         .build(),
/// ];
///
/// client.send("database", &lines).await?;
/// # Ok::<(), rinfluxdb_lineprotocol::ClientError>(())
/// # })?;
/// # Ok::<(), rinfluxdb_lineprotocol::ClientError>(())
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
        let client = ReqwestClientBuilder::new()
            .build()?;

        let credentials = credentials
            .map(|(username, password)| (username.into(), password.into()));

        Ok(Self {
            client,
            base_url,
            credentials,
        })
    }

    /// Sends data using the Influx Line Protocol
    #[instrument(
        name = "Sending data using the Influx Line Protocol",
        skip(self, database, lines),
    )]
    pub async fn send(&self, database: &str, lines: &[Line]) -> Result<(), ClientError> {
        let mut request = self.client
                .line_protocol(&self.base_url, database, lines)?;

        if let Some((username, password)) = &self.credentials {
            request = request.basic_auth(username, Some(password));
        }

        debug!("Sending {} lines to {}", lines.len(), self.base_url);
        trace!("Request: {:?}", request);

        let response = request.send().await?;

        response.process_line_protocol_response().await?;

        Ok(())
    }
}

/// A trait to obtain a prepared Influx Line Protocol request builder from [Reqwest clients](reqwest::Client).
///
/// This trait is used to attach a `line_protocol()` function to [`reqwest::Client`](reqwest::Client).
///
/// ```no_run
/// # use url::Url;
/// # use rinfluxdb_lineprotocol::LineBuilder;
/// // Bring into scope the trait implementation
/// use rinfluxdb_lineprotocol::r#async::InfluxLineClientWrapper;
///
/// # async_std::task::block_on(async {
/// // Create Reqwest client
/// let client = reqwest::Client::new();
///
/// // Set database name
/// let database = "database";
///
/// // Create data
/// let lines = vec![
///     LineBuilder::new("measurement")
///         .insert_field("field", 42.0)
///         .build(),
///     LineBuilder::new("measurement")
///         .insert_field("field", 43.0)
///         .insert_tag("tag", "value")
///         .build(),
/// ];
///
/// // Create Influx Line Protocol request
/// let base_url = Url::parse("https://example.com")?;
/// let mut builder = client
///     // (this is a function added by the trait above)
///     .line_protocol(&base_url, &database, &lines)?;
///
/// // This is a regular Reqwest builder, and can be customized as usual
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
/// # Ok::<(), rinfluxdb_lineprotocol::ClientError>(())
/// # })?;
/// # Ok::<(), rinfluxdb_lineprotocol::ClientError>(())
/// ```
pub trait InfluxLineClientWrapper {
    /// Create an Influx Line Protocol request builder
    ///
    /// The request will point to the InfluxDB instance available at
    /// `base_url`.
    /// In particular, it will send a POST request to `base_url + "/query"`.
    fn line_protocol(
        &self,
        base_url: &Url,
        database: &str,
        lines: &[Line],
    ) -> Result<Self::RequestBuilderType, ClientError>;

    /// The type of the resulting request builder
    ///
    /// This type is a parameter so the trait can be implemented for
    /// `reqwest::Client` returning `reqwest::RequestBuilder`, and for
    /// `reqwest::Client` returning `reqwest::RequestBuilder`.
    type RequestBuilderType;
}

impl InfluxLineClientWrapper for ReqwestClient {
    type RequestBuilderType = ReqwestRequestBuilder;

    fn line_protocol(
        &self,
        base_url: &Url,
        database: &str,
        lines: &[Line],
    ) -> Result<ReqwestRequestBuilder, ClientError> {
        let mut url = base_url.join("/write")?;
        let query = "db=".to_string() + database;
        url.set_query(Some(&query));

        let strings: Vec<String> = lines.iter().map(|line| line.to_string()).collect();
        let payload: String = strings.join("\n");

        let builder = self
            .post(url)
            .body(payload);

        Ok(builder)
    }
}

/// A trait to parse a list of dataframes from [Reqwest responses](reqwest::Response).
///
/// This trait is used to attach a `dataframes()` function to [`reqwest::Response`](reqwest::Response).
///
/// ```no_run
/// # use url::Url;
/// # use rinfluxdb_lineprotocol::LineBuilder;
/// use rinfluxdb_lineprotocol::r#async::InfluxLineClientWrapper;
///
/// // Bring into scope the trait implementation
/// use rinfluxdb_lineprotocol::r#async::InfluxLineResponseWrapper;
///
/// # async_std::task::block_on(async {
/// // Create Reqwest client
/// let client = reqwest::Client::new();
///
/// // Set database name
/// let database = "database";
///
/// // Create data
/// let lines = vec![
///     LineBuilder::new("measurement")
///         .insert_field("field", 42.0)
///         .build(),
///     LineBuilder::new("measurement")
///         .insert_field("field", 43.0)
///         .insert_tag("tag", "value")
///         .build(),
/// ];
///
/// // Create Influx Line Protocol request
/// let base_url = Url::parse("https://example.com")?;
/// let mut builder = client
///     // (this is a function added by the trait above)
///     .line_protocol(&base_url, &database, &lines)?;
///
/// // This is a regular Reqwest builder, and can be customized as usual
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
/// // Process the response.
/// response.process_line_protocol_response().await?;
///
/// # Ok::<(), rinfluxdb_lineprotocol::ClientError>(())
/// # })?;
/// # Ok::<(), rinfluxdb_lineprotocol::ClientError>(())
/// ```
#[async_trait]
pub trait InfluxLineResponseWrapper {
    /// Process the response, parsing potential errors
    async fn process_line_protocol_response(self) -> Result<(), ClientError>;
}

#[async_trait]
impl InfluxLineResponseWrapper for ReqwestResponse {
    async fn process_line_protocol_response(self) -> Result<(), ClientError> {
        match self.error_for_status_ref() {
            Ok(_) => Ok(()),
            Err(_) => {
                let text = self.text().await?;
                debug!("Response: \"{}\"", text);
                let error = parse_error(&text);
                Err(error)
            }
        }
    }
}
