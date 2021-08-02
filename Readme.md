Rust InfluxDB Library
====

A library for querying and sending data to InfluxDB.

<https://gitlab.com/claudiomattera/rinfluxdb>


Features
----

* Serialize data to InfluxDB line protocol;
* Build InfluxQL queries in Rust;
* Build FLUX queries in Rust;
* Parse responses from InfluxDB;
* (Optional) client based on Reqwest to perform common queries.
* Parse dataframes from InfluxDB JSON returned for InfluxQL queries;
* Parse dataframes from InfluxDB annotated CSV returned for FLUX queries;
* (Optional) wrapper around Reqwest objects to construct requests and parse responses;


### Serialize Data to InfluxDB Line Protocol

The [line protocol] is used to send data to InfluxDB.

[line protocol]: https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/

~~~~rust
use rinfluxdb::line_protocol::LineBuilder;
use chrono::{TimeZone, Utc};

let line = LineBuilder::new("location")
    .insert_field("latitude", 55.383333)
    .insert_field("longitude", 10.383333)
    .insert_tag("city", "Odense")
    .set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
    .build();

assert_eq!(line.measurement(), &"location".into());
assert_eq!(line.field("latitude"), Some(&55.383333.into()));
assert_eq!(line.field("longitude"), Some(&10.383333.into()));
assert_eq!(line.tag("city"), Some(&"Odense".into()));
assert_eq!(line.timestamp(), Some(&Utc.ymd(2014, 7, 8).and_hms(9, 10, 11)));

assert_eq!(
    line.to_string(), 
    "location,city=Odense latitude=55.383333,longitude=10.383333 1404810611000000000"
);
~~~~


### Build InfluxQL Queries in Rust

InfluxQL queries can be built using `influxql::QueryBuilder`.

~~~~rust
use rinfluxdb::influxql::QueryBuilder;
use chrono::{TimeZone, Utc};

let query = QueryBuilder::from("indoor_environment")
    .field("temperature")
    .field("humidity")
    .start(Utc.ymd(2021, 3, 7).and_hms(21, 0, 0))
    .build();

assert_eq!(
    query.as_ref(),
    "SELECT temperature, humidity \
    FROM indoor_environment \
    WHERE time > '2021-03-07T21:00:00Z'",
);
~~~~


### Build FLUX Queries in Rust

FLUX queries can be built using `flux::QueryBuilder`.

~~~~rust
use rinfluxdb::types::Duration;
use rinfluxdb::flux::QueryBuilder;

let query = QueryBuilder::from("telegraf/autogen")
    .range_start(Duration::Minutes(-15))
    .filter(
        r#"r._measurement == "cpu" and
        r._field == "usage_system" and
        r.cpu == "cpu-total""#
    )
    .build();

assert_eq!(
    query.as_ref(),
    r#"from(bucket: "telegraf/autogen")
  |> range(start: -15m)
  |> filter(fn: (r) =>
    r._measurement == "cpu" and
    r._field == "usage_system" and
    r.cpu == "cpu-total"
  )
  |> yield()"#,
);
~~~~


### Parse Responses from InfluxDB

When sending a query to InfluxDB, it will reply with either JSON or annotated CSV content containing a list of dataframes.
This library allows to parse such replies to user-defined dataframe types.

A dataframe must be constructable from its name (a string), its index (a vector of instants) and its columns (a mapping of column names to vector of values).

A dataframe implementation only needs to implement this trait to be used with this crate.
I. e., as long as trait `TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>` is implemented for a given type `DF` (and type `E` implements `Into<ParseError>`), the parser can use it to construct the final objects.

A dummy implementation of a dataframe is available as `dataframe::DataFrame`, but the trait can be implemented for many other existing libraries.


#### Parse JSON Returned for InfluxQL Queries

JSON responses to InfluxQL queries can be parsed to dataframes.

~~~~no_run
use rinfluxdb::influxql::{ResponseError, StatementResult, from_str};
use rinfluxdb::dataframe::DataFrame;

let input: String = todo!();

let response: Result<Vec<StatementResult<DataFrame>>, ResponseError> =
    from_str(&input);
~~~~


#### Parse Annotated CSV Returned for FLUX Queries

Annotated CSV responses to FLUX queries can be parsed.

~~~~no_run
use rinfluxdb::flux::{ResponseError, from_str};
use rinfluxdb::dataframe::DataFrame;

let input: String = todo!();

let response: Result<DataFrame, ResponseError> = from_str(&input);
~~~~


### (Optional) Use a Client Based on Reqwest to Perform Common Queries

The functions shown above can be used to serialize and deserialize queries and data to raw text, and they can be integrated into existing applications.
In alternative, this library also implements an optional client API based on [Reqwest] to directly interact with an InfluxDB instance.
Both blocking and asynchronous clients are available, and they support common queries.

Clients are enabled using the `client` Cargo feature.


#### Query InfluxDB with InfluxQL

~~~~no_run
# use std::collections::HashMap;
# use url::Url;
#
use rinfluxdb::influxql::QueryBuilder;
use rinfluxdb::influxql::blocking::Client;
use rinfluxdb::dataframe::DataFrame;

let client = Client::new(
    Url::parse("https://example.com/")?,
    Some(("username", "password")),
)?;

let query = QueryBuilder::from("indoor_environment")
    .database("house")
    .field("temperature")
    .field("humidity")
    .build();
let dataframe: DataFrame = client.fetch_dataframe(query)?;
println!("{}", dataframe);

let query = QueryBuilder::from("indoor_environment")
    .database("house")
    .field("temperature")
    .field("humidity")
    .group_by("room")
    .build();
let tagged_dataframes: HashMap<String, DataFrame> = 
    client.fetch_dataframes_by_tag(query, "room")?;
for (tag, dataframe) in tagged_dataframes {
    println!("{}: {}", tag, dataframe);
}

# Ok::<(), rinfluxdb::influxql::ClientError>(())
~~~~


#### Query InfluxDB with FLUX

~~~~ignore
unimplemented!()
~~~~


#### Send Data to InfluxDB using the Line Protocol

~~~~no_run
# use url::Url;
#
use rinfluxdb::line_protocol::LineBuilder;
use rinfluxdb::line_protocol::blocking::Client;

let client = Client::new(
    Url::parse("https://example.com/")?,
    Some(("username", "password")),
)?;

let lines = vec![
    LineBuilder::new("measurement")
        .insert_field("field", 42.0)
        .build(),
    LineBuilder::new("measurement")
        .insert_field("field", 43.0)
        .insert_tag("tag", "value")
        .build(),
];

client.send("database", &lines)?;

# Ok::<(), rinfluxdb::line_protocol::ClientError>(())
~~~~


### (Optional) Wrapper around Reqwest Objects to Construct Requests and Parse Responses

This crate communicates with InfluxDB instances over HTTP(s).
The content of HTTP requests and responses must follow InfluxDB conventions and protocols, but they can otherwise be customized, e.g. by adding basic authentication.

In order to ensure maximal freedom, a tiny wrapper is constructed around Reqwest's `Client`, so that it can create a request builder already prepared to communicate with InfluxDB.
Such builder can be then converted to a regular request builder and executed.

~~~~no_run
# use url::Url;
#
use rinfluxdb::influxql::Query;

// Bring into scope the trait implementation
use rinfluxdb::influxql::blocking::InfluxqlClientWrapper;

// Create Reqwest client
let client = reqwest::blocking::Client::new();

// Create InfluxQL request
let base_url = Url::parse("https://example.com")?;
let mut builder = client
    // (this is a function added by the trait above)
    .influxql(&base_url)?
    // (this functions are defined on influxql::RequestBuilder)
    .database("house")
    .query(Query::new("SELECT temperature FROM indoor_temperature"))
    // (this function returns a regular Reqwest builder)
    .into_reqwest_builder();

// Now this is a regular Reqwest builder, and can be customized as usual
if let Some((username, password)) = Some(("username", "password")) {
    builder = builder.basic_auth(username, Some(password));
}

// Create a request from the builder
let request = builder.build()?;

// Execute the request through Reqwest and obtain a response
let response = client.execute(request)?;

# Ok::<(), rinfluxdb::influxql::ClientError>(())
~~~~


Similarly, a tiny wrapper is constructed around Reqwest's `Response`, so that a new function is added to parse dataframes from it.

~~~~no_run
# use std::collections::HashMap;
#
# use url::Url;
#
use rinfluxdb::influxql::Query;

use rinfluxdb::influxql::StatementResult;
use rinfluxdb::influxql::blocking::InfluxqlClientWrapper;
use rinfluxdb::dataframe::DataFrame;

// Bring into scope the trait implementation
use rinfluxdb::influxql::blocking::InfluxqlResponseWrapper;

// Create Reqwest client
let client = reqwest::blocking::Client::new();

// Create InfluxQL request
let base_url = Url::parse("https://example.com")?;
let mut request = client
    .influxql(&base_url)?
    .database("house")
    .query(Query::new("SELECT temperature FROM indoor_temperature"))
    .into_reqwest_builder()
    .build()?;

// Execute the request through Reqwest and obtain a response
let response = client.execute(request)?;

// Return an error if response status is not 200
// (this is a function from Reqwest's response)
let response = response.error_for_status()?;

// Parse the response from JSON to a list of dataframes
// (this is a function added by the trait above)
let results: Vec<StatementResult<DataFrame>> = response.dataframes()?;

# Ok::<(), rinfluxdb::influxql::ClientError>(())
~~~~


Wrappers are defined for both Reqwest's blocking API (`influxql::blocking::InfluxqlClientWrapper`, `influxql::blocking::InfluxqlResponseWrapper`) and Reqwest's asynchronous API (`influxql::r#async::InfluxqlClientWrapper`, `influxql::r#async::InfluxqlResponseWrapper`), and are enabled using the `client` Cargo feature.


[Reqwest]: https://lib.rs/crates/reqwest


Usage
----

This crate is a simple aggregator over smaller crates, each enabled by a Cargo feature and implementing a specific part of InfluxDB support.

~~~~plain
rinfluxdb
├── rinfluxdb-types
├── rinfluxdb-lineprotocol
├── rinfluxdb-influxql
├── rinfluxdb-flux
└── rinfluxdb-dataframe
~~~~

Clients can either depend on `rinfluxdb` enabling the necessary features, or they can depend explicitly on the `rinfluxdb-*` crates.

~~~~toml
[dependencies.rinfluxdb]
version = "0.2.0"
features = ["lineprotocol", "influxql", "client"]

# Or

[dependencies]
rinfluxdb-lineprotocol = { version = "0.2.0", features = ["client"] }
rinfluxdb-influxql = { version = "0.2.0", features = ["client"] }
~~~~


### Cargo Features

This crate supports the following Cargo features.

* `lineprotocol`: re-exports `rinfluxdb-lineprotocol` crate;
* `influxql`: re-exports `rinfluxdb-influxql` crate;
* `flux`: re-exports `rinfluxdb-flux` crate;
* `dataframe`: re-exports `rinfluxdb-dataframe` crate;
* `client`: enables feature `client` in all `rinfluxdb-*` crates.

When feature `client` is enabled, the crates define clients for line protocol, InfluxQL and Flux.
Clients are implemented using [Reqwest], and are available both for blocking and async mode.


License
----

Copyright Claudio Mattera 2021

You are free to copy, modify, and distribute this application with attribution under the terms of the [MIT license]. See the [`License.txt`](./License.txt) file for details.

This project is entirely original work, and it is not affiliated with nor endorsed in any way by InfluxData.

[MIT license]: https://opensource.org/licenses/MIT
