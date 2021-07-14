// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::collections::HashMap;

use std::io::stderr;

use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::fmt as subscriber_fmt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use structopt::clap::{crate_authors, crate_description, crate_name};
use structopt::StructOpt;

use url::Url;

use rinfluxdb::influxql::{ClientError, ResponseError, Query};
use rinfluxdb::influxql::blocking::Client;
use rinfluxdb::dataframe::DataFrame;

fn main() -> Result<(), ClientError> {
    let arguments = Arguments::from_args();
    setup_logging(arguments.verbosity);

    let client = Client::new(
        arguments.host,
        Some((arguments.username, arguments.password)),
    )?;

    let query = Query::new(arguments.query);

    type TaggedDataFrames = Vec<(DataFrame, Option<HashMap<String, String>>)>;
    let results: Vec<Result<TaggedDataFrames, ResponseError>>
        = client.fetch_readings_from_database(query, Some(arguments.database))?;

    for (i, result) in results.into_iter().enumerate() {
        let dataframes_and_tags = result?;
        println!("Statement {} returned {} data-frames", i + 1, dataframes_and_tags.len());
        for (j, (dataframe, tags)) in dataframes_and_tags.into_iter().enumerate() {
            print!("Data-frame {}", j + 1);

            if let Some(tags) = tags {
                println!(
                    " (tags: {}):\n",
                    tags
                        .into_iter()
                        .map(|(k, v)| format!("{} = {}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            } else {
                println!(":\n");
            }

            println!("{}", dataframe);
        }
        println!();
    }

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

    /// InfluxDB database
    #[structopt(short, long)]
    pub database: String,

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
