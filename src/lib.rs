// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

//! Rust InfluxDB Library
//! ====
//!
//! A library for querying and sending data to InfluxDB.
//!
//! <https://gitlab.com/claudiomattera/rinfluxdb>


pub mod flux;
pub mod influxql;
pub mod line_protocol;
pub mod types;


pub mod dataframe;

#[cfg(feature = "client")]
/// A client for performing frequent Flux queries in a convenient way
pub type FluxClient = flux::blocking::Client;

#[cfg(feature = "client")]
/// A client for performing frequent asynchronous Flux queries in a convenient way
pub type FluxAsyncClient = flux::r#async::Client;

/// A Flux query
pub type FluxQuery = flux::Query;

/// A builder for Flux queries
pub type FluxQueryBuilder = flux::QueryBuilder;

#[cfg(feature = "client")]
/// A client for performing frequent InfluxQL queries in a convenient way
pub type InfluxqlClient = influxql::blocking::Client;

#[cfg(feature = "client")]
/// A client for performing frequent asynchronous InfluxQL queries in a convenient way
pub type InfluxqlAsyncClient = influxql::r#async::Client;

/// An InfluxQL query
pub type InfluxqlQuery = influxql::Query;

/// A builder for InfluxQL queries
pub type InfluxqlQueryBuilder = influxql::QueryBuilder;

#[cfg(feature = "client")]
/// A client for sending data with Influx Line Protocol queries in a convenient
/// way
pub type InfluxLineClient = line_protocol::blocking::Client;

#[cfg(feature = "client")]
/// A client for asynchronously sending data with Influx Line Protocol queries in a convenient
/// way
pub type InfluxAsyncLineClient = line_protocol::r#async::Client;

/// Represent a line in Influx Line Protocol
pub type InfluxLine = line_protocol::Line;

/// A builder for Influx Line Protocol lines
pub type InfluxLineBuilder = line_protocol::LineBuilder;
