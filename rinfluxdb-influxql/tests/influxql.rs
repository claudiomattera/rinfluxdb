// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::collections::HashMap;

use httpmock::Method::POST;
use httpmock::MockServer;

use anyhow::Result;

use url::Url;

use rinfluxdb_dataframe::DataFrame;
use rinfluxdb_influxql::blocking::Client as InfluxqlClient;
use rinfluxdb_influxql::QueryBuilder as InfluxqlQueryBuilder;

use std::io::stderr;

use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt as subscriber_fmt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use chrono::{TimeZone, Utc};

use std::sync::Once;

static INIT: Once = Once::new();

fn setup_logging() {
    INIT.call_once(|| {
        // Redirect all `log`'s events to our subscriber
        LogTracer::init().expect("Failed to set logger");

        let default_log_filter = "warn";
        let env_filter =
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(default_log_filter));

        let formatting_layer = subscriber_fmt::layer()
            .with_target(true)
            .without_time()
            .with_writer(stderr);

        let subscriber = Registry::default().with(env_filter).with(formatting_layer);

        set_global_default(subscriber).expect("Failed to set subscriber");
    });
}


#[test]
fn influxql_client_simple_query() -> Result<()> {
    setup_logging();

    let server = MockServer::start();

    let result = r#"{
        "results": [
            {
                "statement_id": 0,
                "series": [
                    {
                        "name": "indoor_environment",
                        "columns": ["time","temperature"],
                        "values":[
                            ["2021-03-04T17:00:00Z",28.4],
                            ["2021-03-04T18:00:00Z",29.2]
                        ]
                    }
                ]
            }
        ]
    }"#;

    let hello_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/query")
            .header("Accept", "application/json");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(result);
    });

    let client = InfluxqlClient::new(Url::parse(&server.base_url())?, None::<(&str, &str)>)?;

    let query = InfluxqlQueryBuilder::from("indoor_environment")
        .field("temperature")
        .database("house")
        .start(Utc.ymd(2021, 3, 7).and_hms(21, 0, 0))
        .build();

    let _dataframe: DataFrame = client.fetch_dataframe(query)?;

    hello_mock.assert();

    Ok(())
}

#[test]
fn influxql_client_tagged_query() -> Result<()> {
    setup_logging();

    let server = MockServer::start();

    let result = r#"{
        "results": [
            {
                "statement_id": 0,
                "series": [
                    {
                        "name": "indoor_environment",
                        "columns": ["time","temperature"],
                        "values":[
                            ["2021-03-04T17:00:00Z",28.4],
                            ["2021-03-04T18:00:00Z",29.2]
                        ],
                        "tags": {
                            "room": "bedroom"
                        }
                    },
                    {
                        "name": "indoor_environment",
                        "columns": ["time","temperature"],
                        "values":[
                            ["2021-03-04T17:00:00Z",21.1],
                            ["2021-03-04T18:00:00Z",18.6]
                        ],
                        "tags": {
                            "room": "entrance"
                        }
                    }
                ]
            }
        ]
    }"#;

    let hello_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/query")
            .header("Accept", "application/json");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(result);
    });

    let client = InfluxqlClient::new(Url::parse(&server.base_url())?, None::<(&str, &str)>)?;

    let query = InfluxqlQueryBuilder::from("indoor_environment")
        .field("temperature")
        .database("house")
        .start(Utc.ymd(2021, 3, 7).and_hms(21, 0, 0))
        .group_by("room")
        .build();

    let tagged_dataframes: HashMap<String, DataFrame> =
        client.fetch_dataframes_by_tag(query, "room")?;

    hello_mock.assert();

    assert!(tagged_dataframes.contains_key("bedroom"));
    assert!(tagged_dataframes.contains_key("entrance"));

    Ok(())
}
