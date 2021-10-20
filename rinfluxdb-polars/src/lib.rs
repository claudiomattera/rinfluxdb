// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

//! Polars dataframe implementation

use std::collections::HashMap;
use std::convert::TryFrom;

use chrono::{DateTime, Utc};

use rinfluxdb_types::Value;

use polars::chunked_array::ChunkedArray;
use polars::frame::DataFrame;
use polars::series::Series;
use polars::datatypes::Date64Type;
use polars::chunked_array::temporal::FromNaiveDateTime;
use polars::error::PolarsError;

/// Wrapper around [Polars](https://lib.rs/crates/polars) dataframe
///
/// It is not possible to implement
/// `TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>`
/// directly for Polars dataframes, so the newtype pattern is used with a unit
/// struct.
///
/// Note that Polars dataframe cannot be indexed by datetimes, so the index is
/// stored in a regular column named `index`.
pub struct DataFrameWrapper(pub DataFrame);

impl TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>)> for DataFrameWrapper {
    type Error = PolarsError;

    fn try_from(
        (_name, index, columns): (String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>),
    ) -> Result<Self, Self::Error> {
        let columns: HashMap<String, Result<Series, Self::Error>> = columns
            .into_iter()
            .map(|(name, column)| {
                let column = match column.first() {
                    Some(Value::Float(_)) => Ok(
                        column
                            .into_iter()
                            .map(|element| element.into_f64())
                            .collect(),
                    ),
                    Some(Value::Integer(_)) => Ok(
                        column
                            .into_iter()
                            .map(|element| element.into_i64())
                            .collect(),
                    ),
                    Some(Value::UnsignedInteger(_)) => Ok(
                        column
                            .into_iter()
                            .map(|element| element.into_u64())
                            .collect(),
                    ),
                    Some(Value::String(_)) => Ok(
                        column
                            .into_iter()
                            .map(|element| element.into_string())
                            .collect(),
                    ),
                    Some(Value::Boolean(_)) => Ok(
                        column
                            .into_iter()
                            .map(|element| element.into_boolean())
                            .collect(),
                    ),
                    Some(Value::Timestamp(_)) => Ok(
                        datetime_value_column_to_series(&name, column),
                    ),
                    None => Err(PolarsError::ValueError("Empty column".into())),
                };
                (name, column)
            })
            .collect();

        let mut series_map: HashMap<String, Series> = flatten_map(columns)?;
        series_map.insert("index".to_string(), datetimes_to_series("index", index.into_iter()));

        let series: Vec<Series> = series_map
            .into_iter()
            .map(|(name, mut series)| {
                series.rename(&name);
                series
            })
            .collect();
        let dataframe = DataFrame::new(series)?;
        Ok(DataFrameWrapper(dataframe))
    }
}

fn datetimes_to_series<A>(name: &str, column: A) -> Series
where
    A: Iterator<Item=DateTime<Utc>>
{
    let values: Vec<_> = column
        .map(|element| element.naive_utc())
        .collect();
    let array: ChunkedArray<Date64Type> = FromNaiveDateTime::new_from_naive_datetime(
            name,
            &values,
        );
    array.into()
}

fn values_to_datetimes<A>(values: A) -> impl Iterator<Item=DateTime<Utc>>
where
    A: Iterator<Item=Value>,
{
    values.map(|element| element.into_timestamp())
}

fn datetime_value_column_to_series(name: &str, column: Vec<Value>) -> Series {
    datetimes_to_series(
        name,
        values_to_datetimes(column.into_iter()),
    )
}

fn flatten_map<K, V, E>(map: HashMap<K, Result<V, E>>) -> Result<HashMap<K, V>, E>
where
    K: Eq + std::hash::Hash,
    E: std::error::Error,
{
    map.into_iter()
        .try_fold(HashMap::new(), |mut accumulator, (name, column)| {
            let column = column?;
            accumulator.insert(name, column);
            Ok(accumulator)
        })
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use chrono::TimeZone;

    use super::*;

    macro_rules! named_series{
        ($a:expr, $b:expr)=>{
            {
                let mut series: Series = $b;
                series.rename($a);
                series
            }
        }
    }

    #[test]
    fn polars_dataframe_creation() -> Result<(), Box<dyn std::error::Error>> {
        let name: String = "environment".into();
        let index: Vec<DateTime<Utc>> = vec![
            Utc.ymd(2021, 10, 20).and_hms(5, 20, 21),
            Utc.ymd(2021, 10, 20).and_hms(5, 20, 22),
            Utc.ymd(2021, 10, 20).and_hms(5, 20, 23),
            Utc.ymd(2021, 10, 20).and_hms(5, 20, 24),
        ];
        let mut columns: HashMap<String, Vec<Value>> = HashMap::new();
        columns.insert(
            "temperature".into(),
            vec![
                Value::Float(23.2),
                Value::Float(23.5),
                Value::Float(23.7),
                Value::Float(23.4),
            ]
        );
        columns.insert(
            "humidity".into(),
            vec![
                Value::UnsignedInteger(40_u64),
                Value::UnsignedInteger(38_u64),
                Value::UnsignedInteger(34_u64),
                Value::UnsignedInteger(39_u64),
            ]
        );
        columns.insert(
            "rain".into(),
            vec![
                Value::Boolean(false),
                Value::Boolean(true),
                Value::Boolean(true),
                Value::Boolean(false),
            ]
        );

        let expected_dataframe = DataFrame::new(vec![
                named_series!(
                    "temperature",
                    vec![23.2, 23.5, 23.7, 23.4]
                        .iter()
                        .collect()
                ),
                named_series!(
                    "rain",
                    vec![false, true, true, false]
                        .iter()
                        .collect()
                ),
                named_series!(
                    "humidity",
                    vec![40_u64, 38_u64, 34_u64, 39_u64]
                        .iter()
                        .collect()
                ),
                datetimes_to_series(
                    "index",
                    vec![
                        Utc.ymd(2021, 10, 20).and_hms(5, 20, 21),
                        Utc.ymd(2021, 10, 20).and_hms(5, 20, 22),
                        Utc.ymd(2021, 10, 20).and_hms(5, 20, 23),
                        Utc.ymd(2021, 10, 20).and_hms(5, 20, 24),
                    ]
                        .into_iter()
                ),
            ])?;

        let wrapper: Result<DataFrameWrapper, _> = (name, index, columns).try_into();
        assert!(wrapper.is_ok());

        let wrapper = wrapper?;
        let dataframe = wrapper.0;

        println!("Dataframe: {:?}", dataframe);
        println!("Expected: {:?}", expected_dataframe);

        // Columns order is non-deterministic but dataframes with different
        // columns orders are not compared as equal, so the following assert
        // fails non-deterministically
        //assert!(dataframe.frame_equal(&expected_dataframe));

        // Manually sort the columns and compare them one by one
        let mut columns: Vec<_> = dataframe.get_columns().iter().collect();
        let mut expected_columns: Vec<_> = expected_dataframe.get_columns().iter().collect();
        columns.sort_by_key(|column| column.name());
        expected_columns.sort_by_key(|column| column.name());
        for (column, expected_column) in columns.into_iter().zip(expected_columns.into_iter()) {
            assert!(column.series_equal(expected_column));
        }

        Ok(())
    }
}
