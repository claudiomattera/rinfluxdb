// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use httpmock::MockServer;
use httpmock::Method::POST;

use anyhow::Result;

use url::Url;

use rinfluxdb::line_protocol::blocking::Client as InfluxLineClient;
use rinfluxdb::line_protocol::ClientError;
use rinfluxdb::InfluxLineBuilder;

use std::io::stderr;

use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt as subscriber_fmt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use std::sync::Once;

static INIT: Once = Once::new();

fn setup_logging() {
    INIT.call_once(|| {
        // Redirect all `log`'s events to our subscriber
        LogTracer::init().expect("Failed to set logger");

        let default_log_filter = "warn";
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_log_filter));

        let formatting_layer = subscriber_fmt::layer()
            .with_target(true)
            .without_time()
            .with_writer(stderr);

        let subscriber = Registry::default().with(env_filter).with(formatting_layer);

        set_global_default(subscriber).expect("Failed to set subscriber");
    });
}


#[test]
fn client_send() -> Result<()> {
    setup_logging();

    let server = MockServer::start();

    let hello_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/write")
            .query_param("db", "database");
        then.status(200)
            .body("");
    });

    let client = InfluxLineClient::new(Url::parse(&server.base_url())?, None::<(&str, &str)>)?;

    let lines = vec![
        InfluxLineBuilder::new("measurement")
            .insert_field("field", 42.0)
            .build(),
        InfluxLineBuilder::new("measurement")
            .insert_field("field", 43.0)
            .insert_tag("tag", "value")
            .build(),
    ];

    client.send("database", &lines)?;

    hello_mock.assert();

    Ok(())
}

#[test]
fn client_send_authenticated() -> Result<()> {
    setup_logging();

    let server = MockServer::start();

    let hello_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/write")
            .header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .query_param("db", "database");
        then.status(200)
            .body("");
    });

    let client = InfluxLineClient::new(Url::parse(&server.base_url())?, Some(("username", "password")))?;

    let lines = vec![
        InfluxLineBuilder::new("measurement")
            .insert_field("field", 42.0)
            .build(),
        InfluxLineBuilder::new("measurement")
            .insert_field("field", 43.0)
            .insert_tag("tag", "value")
            .build(),
    ];

    client.send("database", &lines)?;

    hello_mock.assert();

    Ok(())
}

#[test]
fn client_send_database_not_found() -> Result<()> {
    setup_logging();

    let server = MockServer::start();

    let hello_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/write")
            .query_param("db", "unknown");
        then.status(400)
            .body(r#"{"error": "database not found: \"unknown\""}"#);
    });

    let client = InfluxLineClient::new(Url::parse(&server.base_url())?, None::<(&str, &str)>)?;

    let lines = vec![
        InfluxLineBuilder::new("measurement")
            .insert_field("field", 42.0)
            .build(),
        InfluxLineBuilder::new("measurement")
            .insert_field("field", 43.0)
            .insert_tag("tag", "value")
            .build(),
    ];

    let result = client.send("unknown", &lines);

    hello_mock.assert();

    match result {
        Err(ClientError::DatabaseNotFound) => {},
        result => panic!("Did not receive expected error: {:?}", result),
    }


    Ok(())
}

#[test]
fn client_send_field_type_conflict() -> Result<()> {
    setup_logging();

    let server = MockServer::start();

    let hello_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/write")
            .query_param("db", "database");
        then.status(400)
            .body(r#"{"error": "field type conflict: input field \"temperature\" on measurement \"indoor_environment\" is type float, already exists as type boolean dropped=1"}"#);
    });

    let client = InfluxLineClient::new(Url::parse(&server.base_url())?, None::<(&str, &str)>)?;

    let lines = vec![
        InfluxLineBuilder::new("indoor_environment")
            .insert_field("temperature", 42.0)
            .build(),
    ];

    let result = client.send("database", &lines);

    hello_mock.assert();

    match result {
        Err(ClientError::FieldTypeConflict) => {},
        result => panic!("Did not receive expected error: {:?}", result),
    }


    Ok(())
}
