// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

#![doc = include_str!("../../Readme.md")]

pub use rinfluxdb_types as types;

#[cfg(feature = "lineprotocol")]
pub use rinfluxdb_lineprotocol as line_protocol;

#[cfg(feature = "influxql")]
pub use rinfluxdb_influxql as influxql;

#[cfg(feature = "flux")]
pub use rinfluxdb_flux as flux;

#[cfg(feature = "dataframe")]
pub use rinfluxdb_dataframe as dataframe;

#[cfg(feature = "polars")]
pub use rinfluxdb_polars as polars;

#[cfg(all(feature = "client", feature = "flux"))]
/// A client for performing frequent Flux queries in a convenient way
pub type FluxClient = flux::blocking::Client;

#[cfg(all(feature = "client", feature = "flux"))]
/// A client for performing frequent asynchronous Flux queries in a convenient way
pub type FluxAsyncClient = flux::r#async::Client;

/// A Flux query
#[cfg(feature = "flux")]
pub type FluxQuery = flux::Query;

/// A builder for Flux queries
#[cfg(feature = "flux")]
pub type FluxQueryBuilder = flux::QueryBuilder;

#[cfg(all(feature = "client", feature = "influxql"))]
/// A client for performing frequent InfluxQL queries in a convenient way
pub type InfluxqlClient = influxql::blocking::Client;

#[cfg(all(feature = "client", feature = "influxql"))]
/// A client for performing frequent asynchronous InfluxQL queries in a convenient way
pub type InfluxqlAsyncClient = influxql::r#async::Client;

/// An InfluxQL query
#[cfg(feature = "influxql")]
pub type InfluxqlQuery = influxql::Query;

/// A builder for InfluxQL queries
#[cfg(feature = "influxql")]
pub type InfluxqlQueryBuilder = influxql::QueryBuilder;

#[cfg(all(feature = "client", feature = "lineprotocol"))]
/// A client for sending data with Influx Line Protocol queries in a convenient
/// way
pub type InfluxLineClient = line_protocol::blocking::Client;

#[cfg(all(feature = "client", feature = "lineprotocol"))]
/// A client for asynchronously sending data with Influx Line Protocol queries in a convenient
/// way
pub type InfluxAsyncLineClient = line_protocol::r#async::Client;

/// A line in the Influx Line Protocol
#[cfg(feature = "lineprotocol")]
pub type InfluxLine = line_protocol::Line;

/// A builder for Influx Line Protocol lines
#[cfg(feature = "lineprotocol")]
pub type InfluxLineBuilder = line_protocol::LineBuilder;
