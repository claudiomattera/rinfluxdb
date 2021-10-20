// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use std::io::stderr;

use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt as subscriber_fmt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// use anyhow::Result;

use structopt::clap::{crate_authors, crate_description, crate_name};
use structopt::StructOpt;

use url::Url;

use chrono::Duration;

use rinfluxdb_dataframe::DataFrame;
use rinfluxdb_flux::blocking::Client;
use rinfluxdb_flux::{ClientError, QueryBuilder};

fn main() -> Result<(), ClientError> {
    let arguments = Arguments::from_args();
    setup_logging(arguments.verbosity);

    let client = Client::new(
        arguments.host,
        Some((arguments.username, arguments.password)),
    )?;

    let query = QueryBuilder::from(arguments.bucket)
        .range_start(Duration::hours(-2))
        .filter(
            r#"
            r._measurement == "indoor_environment" and
            r._field == "temperature""#,
        )
        .build();

    let _dataframe: DataFrame = client.fetch_readings(query)?;

    // let response = client
    //     .post(url)
    //     .form(&params)
    //     .basic_auth(arguments.username, Some(arguments.password))
    //     .send()?;

    // let response = response.error_for_status()?;

    // let text = response.text()?;

    // println!("{}", text);

    // let results: Vec<Result<Vec<(DataFrame, Option<HashMap<String, String>>)>, ParseError>> = from_str(&text)?;
    // println!("Fetched {} statement results", results.len());

    // for (i, result) in results.into_iter().enumerate() {
    //     let dataframes_and_tags = result?;
    //     println!("Statement {} returned {} data-frames", i + 1, dataframes_and_tags.len());
    //     for (j, (dataframe, tags)) in dataframes_and_tags.into_iter().enumerate() {
    //         println!("Data-frame {}:", j + 1);

    //         println!("  {:?}", dataframe);

    //         if let Some(tags) = tags {
    //             println!("  Tags: {}", tags.into_iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", "));
    //         }

    //         println!();
    //     }
    //     println!();
    // }

    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = crate_name!(), about = crate_description!(), author = crate_authors!())]
pub struct Arguments {
    /// Verbosity
    #[structopt(short, long = "verbose", parse(from_occurrences))]
    pub verbosity: u8,

    /// InfluxDB host
    #[structopt(long, parse(try_from_str = Url::parse))]
    pub host: Url,

    /// InfluxDB bucket
    #[structopt(short, long)]
    pub bucket: String,

    /// Influxdb username
    #[structopt(short, long)]
    pub username: String,

    /// Influxdb password
    #[structopt(short, long)]
    pub password: String,

    /// Listen port
    #[structopt()]
    pub query: String,
}

fn setup_logging(verbosity: u8) {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init().expect("Failed to set logger");

    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        3 => "debug",
        _ => "trace",
    };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_log_filter));

    let formatting_layer = subscriber_fmt::layer()
        .with_target(true)
        .without_time()
        .with_writer(stderr);

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");
}
