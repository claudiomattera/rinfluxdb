// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use std::collections::HashMap;
use std::convert::TryFrom;
use std::num::ParseFloatError;

use chrono::{DateTime, Utc};

use csv::ReaderBuilder as CsvReaderBuilder;

use itertools::izip;

use thiserror::Error;

use rinfluxdb_types::Value;

use super::ResponseResult;

/// An error occurred while parsing format
#[derive(Error, Debug)]
pub enum ResponseError {
    /// Error while parsing data types row
    #[error("Error while parsing data types row")]
    DataTypes,

    /// Error while parsing grouping row
    #[error("Error while parsing grouping row")]
    Grouping,

    /// Error while parsing default row
    #[error("Error while parsing default row")]
    Default,

    /// Error while parsing columns row
    #[error("Error while parsing columns row")]
    Columns,

    /// Error occurred while parsing CSV
    #[error("CSV parse error")]
    CsvError(#[from] csv::Error),

    /// Error occurred while parsing a datetime
    #[error("Chrono parse error")]
    ParseFloatError(#[from] ParseFloatError),

    /// Input is not a valid ISO8601 datetime
    #[error("could not parse datetime")]
    DatetimeError(#[from] chrono::ParseError),

    /// Error while creating dataframe
    #[error("could not create dataframe")]
    DataFrameError(#[from] rinfluxdb_types::DataFrameError),
}

/// Parse an annotated CSV response returned from InfluxDB to a list of tagged dataframes.
pub fn from_str<DF, E>(input: &str) -> ResponseResult<DF>
where
    DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
    E: Into<ResponseError>,
{
    let payloads: Vec<_> = input.split("\r\n\r\n").collect();

    for payload in payloads {
        if payload.is_empty() {
            break;
        }
        println!("{}", payload);
        println!("-------------");


        let mut csv = CsvReaderBuilder::new()
            .comment(None)
            .has_headers(false)
            .from_reader(payload.as_bytes());
        let mut rows = csv.records();
        let data_types = rows.next().ok_or(ResponseError::DataTypes)??;
        let grouping = rows.next().ok_or(ResponseError::Grouping)??;
        let default = rows.next().ok_or(ResponseError::Default)??;
        let columns = rows.next().ok_or(ResponseError::Columns)??;

        let columns: Vec<_> = izip!(
                columns.into_iter(),
                data_types.into_iter(),
                grouping.into_iter(),
                default.into_iter()
            )
            .skip(1)
            .collect();

        println!("Columns: {:?}", columns);

        let mut index: Vec<DateTime<Utc>> = Vec::new();
        let mut values: Vec<f64> = Vec::new();

        for result in rows {
            let record = result?;
            let pairs = columns.iter().zip(record.into_iter().skip(1));
            for (column, field) in pairs {
                println!("{}: {} (grouping? {})", column.0, field, column.2);
                if column.0 == "_time" {
                    let instant = field.parse()?;
                    index.push(instant);
                }

                if column.0 == "_value" {
                    let value = field.parse()?;
                    values.push(value);
                }

                if !column.0.starts_with('_') {}
            }
            println!();
        }
    }

    todo!()
}
