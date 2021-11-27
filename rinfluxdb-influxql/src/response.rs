// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

//! Functions to parse JSON responses from InfluxDB

use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use chrono::{DateTime, Utc};

use serde::Deserialize;

use serde_json::from_str as json_from_str;
use serde_json::Value as JsonValue;

use thiserror::Error;

use rinfluxdb_types::Value;

use super::{ResponseResult, StatementResult};

type Tags = HashMap<String, String>;

/// An error occurred during parsing InfluxDB JSON response
#[derive(Error, Debug)]
pub enum ResponseError {
    /// Input is not valid JSON
    #[error("invalid JSON")]
    JsonError(#[from] serde_json::Error),

    /// The entire request failed
    #[error("response error {0}")]
    ResponseError(String),

    /// The request succeeded, but one of the statement failed
    #[error("statement error {0}")]
    StatementError(String),

    /// A generic error occurred
    #[error("value error {0}")]
    ValueError(String),

    /// Input is not a valid ISO8601 datetime
    #[error("could not parse datetime")]
    DatetimeError(#[from] chrono::ParseError),

    /// Error while creating dataframe
    #[error("could not create dataframe")]
    DataFrameError(#[from] rinfluxdb_types::DataFrameError),
}

#[derive(Debug, Deserialize, PartialEq)]
enum Response {
    #[serde(rename = "results")]
    Results(Vec<IndexedOutcome>),

    #[serde(rename = "error")]
    Error(String),
}

#[derive(Debug, Deserialize, PartialEq)]
struct IndexedOutcome {
    statement_id: u32,
    series: Option<Vec<Series>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Series {
    name: String,
    columns: Vec<String>,
    values: Vec<Vec<JsonValue>>,
    tags: Option<Tags>,
}

impl TryFrom<Response> for Vec<IndexedOutcome> {
    type Error = ResponseError;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::Results(results) => Ok(results),
            Response::Error(error) => Err(ResponseError::ResponseError(error)),
        }
    }
}

impl TryFrom<IndexedOutcome> for Vec<Series> {
    type Error = ResponseError;

    fn try_from(outcome: IndexedOutcome) -> Result<Self, Self::Error> {
        match outcome.error {
            Some(error) => Err(ResponseError::StatementError(error)),
            None => Ok(outcome.series.unwrap_or_default()),
        }
    }
}

/// Parse a JSON response returned from InfluxDB to a list of tagged dataframes.
///
/// An example of such JSON response is the following.
///
/// ```json
/// {
///     "results": [
///         {
///             "statement_id": 0,
///             "series": [
///                 {
///                     "name": "environment",
///                     "columns": ["time","temperature","humidity"],
///                     "values":[
///                         ["2021-03-04T17:00:00Z",28.4,41.0],
///                         ["2021-03-04T18:00:00Z",29.2,37.0]
///                     ],
///                     "tags": {
///                         "room": "bedroom",
///                         "building": "b1"
///                     }
///                 }
///             ]
///         }
///     ]
/// }
/// ```
///
/// More specifically, a single query to InfluxDB can consist of multiple
/// semicolon-separated statements, and for each of them a result is returned.
/// The result can contain a list of timeseries and a list of tags, or also
/// nothing if the statement did not generate any data.
///
/// For instance, consider the following query.
///
/// ```plain
/// CREATE DATABASE other;
/// SELECT temperature, humidity FROM outdoor;
/// SELECT temperature FROM indoor GROUP BY room
/// ```
///
/// This would return three results:
///
/// * An empty one, corresponding to the first statement `CREATE DATABASE other`.
/// * A single dataframe for outdoor temperature and humidity, without tags.
/// * Multiple dataframes for indoor temperature, one for each room.
///
///
/// ## Return type
///
/// This function is agnostics on the actual return type.
/// The only constraint is that it can be constructed from a string, a list of
/// instants, namely the index, and a map of lists of values, namely the columns.
///
/// I.e. the return type must implement trait
/// `TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>`,
/// where `E` must implement trait `Into<ResponseError>`.
///
///
/// ## Example
///
/// ```
/// # use std::collections::HashMap;
/// # use chrono::{DateTime, Utc};
/// # use rinfluxdb_influxql::{from_str, ResponseError};
/// # use rinfluxdb_types::Value;
///
/// use std::convert::{TryFrom, TryInto};
///
/// struct DummyDataFrame {
///     name: String,
///     index: Vec<DateTime<Utc>>,
///     columns: HashMap<String, Vec<Value>>
/// }
///
/// impl TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>)> for DummyDataFrame {
///     type Error = ResponseError;
///
///     fn try_from(
///         (name, index, columns): (String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>),
///     ) -> Result<Self, Self::Error> {
///         if columns.len() > 0 {
///             Ok(Self { name, index, columns })
///         } else {
///             Err(ResponseError::ValueError("columns list is empty".into()))
///         }
///     }
/// }
///
/// let input = r#"{
///     "results": [
///         {
///             "statement_id": 0,
///             "series": [
///                 {
///                     "name": "environment",
///                     "columns": ["time","temperature","humidity"],
///                     "values":[
///                         ["2021-03-04T17:00:00Z",28.4,41.0],
///                         ["2021-03-04T18:00:00Z",29.2,37.0]
///                     ],
///                     "tags": {
///                         "room": "bedroom",
///                         "building": "b1"
///                     }
///                 }
///             ]
///         }
///     ]
/// }"#;
///
/// let statements: Vec<Result<Vec<(DummyDataFrame, Option<HashMap<String, String>>)>, ResponseError>>;
/// statements = from_str(input)?;
/// assert_eq!(statements.len(), 1);
///
/// for statement in statements {
///     let dataframes_and_tags: Vec<(DummyDataFrame, Option<HashMap<String, String>>)>;
///     dataframes_and_tags = statement?;
///
///     assert_eq!(dataframes_and_tags.len(), 1);
///     let (dataframe, tags) = dataframes_and_tags.first().unwrap();
///
///     assert_eq!(dataframe.index.len(), 2);
///
///     assert_eq!(dataframe.columns.len(), 2);
///     assert!(dataframe.columns.contains_key("temperature"));
///     assert!(dataframe.columns.contains_key("humidity"));
///
///     assert!(tags.is_some());
///     assert_eq!(tags.as_ref().unwrap().get("room"), Some(&"bedroom".to_string()));
///     assert_eq!(tags.as_ref().unwrap().get("building"), Some(&"b1".to_string()));
/// }
/// # Ok::<(), ResponseError>(())
/// ```
pub fn from_str<DF, E>(input: &str) -> ResponseResult<DF>
where
    DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
    E: Into<ResponseError>,
{
    let response: Response = json_from_str(input)?;
    let results: Vec<IndexedOutcome> = response.try_into()?;

    let dataframes = results
        .into_iter()
        // .sorted_by(|IndexedOutcome{statement_id, ..}| statement_id)
        .map(|outcome| {
            let serieses: Result<Vec<Series>, ResponseError> = outcome.try_into();
            serieses.and_then(|serieses| {
                let dataframes = parse_serieses::<DF, E>(serieses)?;
                Ok(dataframes)
            })
        })
        .collect();

    Ok(dataframes)
}

fn parse_serieses<DF, E>(serieses: Vec<Series>) -> StatementResult<DF>
where
    DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
    E: Into<ResponseError>,
{
    serieses
        .into_iter()
        .map(parse_series::<DF, E>)
        .collect()
}

fn parse_series<DF, E>(series: Series) -> Result<(DF, Option<Tags>), ResponseError>
where
    DF: TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>), Error = E>,
    E: Into<ResponseError>,
{
    let name: String = series.name;
    let mut index: Vec<DateTime<Utc>> = vec![];
    let mut data: HashMap<String, Vec<Value>> = HashMap::new();

    for column_name in series.columns.iter().skip(1) {
        data.insert(column_name.clone(), vec![]);
    }

    for row in series.values {
        let instant = row[0]
            .as_str()
            .ok_or_else(|| ResponseError::ValueError("index is not encoded as string".into()))?;
        let instant = instant.parse::<DateTime<Utc>>()?;
        index.push(instant);

        for (column_name, value) in series.columns.iter().skip(1).zip(&row[1..]) {
            let value = match value {
                JsonValue::Null => Err(ResponseError::ValueError("value is null".into())),
                JsonValue::Bool(boolean) => Ok(Value::Boolean(*boolean)),
                JsonValue::Number(ref number) if number.is_i64() => Ok(Value::Integer(number.as_i64().unwrap())),
                JsonValue::Number(ref number) if number.is_u64() => Ok(Value::UnsignedInteger(number.as_u64().unwrap())),
                JsonValue::Number(ref number) if number.is_f64() => Ok(Value::Float(number.as_f64().unwrap())),
                JsonValue::Number(_) => Err(ResponseError::ValueError("value is an invalid array".into())),
                JsonValue::String(string) => Ok(Value::String(string.clone())),
                JsonValue::Array(_) => Err(ResponseError::ValueError("value is a JSON array".into())),
                JsonValue::Object(_) => Err(ResponseError::ValueError("value is a JSON object".into())),
            }?;
            data.get_mut(column_name).expect("Impossible").push(value);
        }
    }

    let dataframe = DF::try_from((name, index, data))
        .map_err(|e| e.into())?;

    Ok((dataframe, series.tags))
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;

    use serde_json::Number as JsonNumber;

    use chrono::TimeZone;

    type TaggedDataFrames = Vec<(DummyDataFrame, Option<Tags>)>;

    #[derive(Debug, PartialEq)]
    struct DummyDataFrame {
        name: String,
        index: Vec<DateTime<Utc>>,
        columns: HashMap<String, Vec<Value>>,
    }

    impl TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>)> for DummyDataFrame {
        type Error = ResponseError;

        fn try_from(
            (name, index, columns): (String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>),
        ) -> Result<Self, Self::Error> {
            Ok(Self { name, index, columns })
        }
    }

    #[test]
    fn parse_error() -> Result<(), serde_json::Error> {
        let input = r#"{
            "error": "error parsing query: found EOF, expected FROM at line 1, char 9"
        }"#;
        let expected = Response::Error("error parsing query: found EOF, expected FROM at line 1, char 9".into());

        let actual: Response = json_from_str(input)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_ok_inner_error() -> Result<(), serde_json::Error> {
        let input = r#"{
            "results": [
                {
                    "statement_id": 12,
                    "error": "database not found: mydb1"
                }
            ]
        }"#;
        let expected = Response::Results(
            vec![
                IndexedOutcome {
                    statement_id: 12,
                    series: None,
                    error: Some("database not found: mydb1".into()),
                }
            ]
        );

        let actual: Response = json_from_str(input)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_ok_inner_second_error() -> Result<(), serde_json::Error> {
        let input = r#"{
            "results": [
                {
                    "statement_id": 15,
                    "series": [
                        {
                            "name": "mymeas",
                            "columns": ["time", "myfield", "mytag1", "mytag2"],
                            "values": [
                                ["2017-03-01T00:16:18Z", 33.1, null, null],
                                ["2017-03-01T00:17:18Z", 12.4, "12", "14"]
                            ]
                        }
                    ]
                },
                {
                    "statement_id": 12,
                    "error": "Other error"
                }
            ]
        }"#;
        let expected = Response::Results(
            vec![
                IndexedOutcome {
                    statement_id: 15,
                    series: Some(
                        vec![
                            Series {
                                name: "mymeas".into(),
                                columns: vec!["time".into(), "myfield".into(), "mytag1".into(), "mytag2".into()],
                                values: vec![
                                    vec![JsonValue::String("2017-03-01T00:16:18Z".into()), JsonValue::Number(JsonNumber::from_f64(33.1).unwrap()), JsonValue::Null, JsonValue::Null],
                                    vec![JsonValue::String("2017-03-01T00:17:18Z".into()), JsonValue::Number(JsonNumber::from_f64(12.4).unwrap()), JsonValue::String("12".into()), JsonValue::String("14".into())],
                                ],
                                tags: None,
                            }
                        ]
                    ),
                    error: None,
                },
                IndexedOutcome {
                    statement_id: 12,
                    series: None,
                    error: Some("Other error".into()),
                }
            ]
        );

        let actual: Response = json_from_str(input)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_ok() -> Result<(), serde_json::Error> {
        let input = r#"{
            "results": [
                {
                    "statement_id": 15,
                    "series": [
                        {
                            "name": "mymeas",
                            "columns": ["time","myfield","mytag1","mytag2"],
                            "values": [
                                ["2017-03-01T00:16:18Z",33.1,null,null],["2017-03-01T00:17:18Z",12.4,"12","14"]
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let expected = Response::Results(
            vec![
                IndexedOutcome {
                    statement_id: 15,
                    series: Some(
                        vec![
                            Series {
                                name: "mymeas".into(),
                                columns: vec!["time".into(), "myfield".into(), "mytag1".into(), "mytag2".into()],
                                values: vec![
                                    vec![JsonValue::String("2017-03-01T00:16:18Z".into()), JsonValue::Number(JsonNumber::from_f64(33.1).unwrap()), JsonValue::Null, JsonValue::Null],
                                    vec![JsonValue::String("2017-03-01T00:17:18Z".into()), JsonValue::Number(JsonNumber::from_f64(12.4).unwrap()), JsonValue::String("12".into()), JsonValue::String("14".into())],
                                ],
                                tags: None,
                            }
                        ]
                    ),
                    error: None,
                }
            ]
        );

        let actual: Response = json_from_str(input)?;
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn parse_ok_to_dataframe() -> Result<(), ResponseError> {
        let input = r#"{
            "results": [
                {
                    "statement_id": 15,
                    "series": [
                        {
                            "name":"mymeas",
                            "columns": ["time","myfield1","myfield2"],
                            "values": [
                                ["2017-03-01T00:16:18Z",33.1,12.5],["2017-03-01T00:17:18Z",12.4,12.7]
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let mut expected = DummyDataFrame {
            name: "mymeas".into(),
            index: vec![Utc.ymd(2017, 3, 1).and_hms(0, 16, 18), Utc.ymd(2017, 3, 1).and_hms(0, 17, 18)],
            columns: HashMap::new(),
        };
        expected.columns.insert("myfield1".into(), vec![Value::Float(33.1), Value::Float(12.4)]);
        expected.columns.insert("myfield2".into(), vec![Value::Float(12.5), Value::Float(12.7)]);

        let actual_response: Vec<Result<TaggedDataFrames, ResponseError>> = from_str(input)?;
        let actual_dataframes: TaggedDataFrames = actual_response.into_iter().next().ok_or_else(|| ResponseError::ValueError("empty list".into()))??;

        let (actual_dataframe, actual_tags): (DummyDataFrame, Option<Tags>) = actual_dataframes.into_iter().next().ok_or_else(|| ResponseError::ValueError("empty list".into()))?;

        assert!(actual_tags.is_none());

        assert_eq!(actual_dataframe, expected);

        Ok(())
    }

    #[test]
    fn parse_ok_to_empty_dataframe() -> Result<(), ResponseError> {
        let input = r#"{
            "results": [
                {
                    "statement_id": 15
                }
            ]
        }"#;

        let actual_response: Vec<Result<TaggedDataFrames, ResponseError>> = from_str(input)?;
        let actual_dataframes: TaggedDataFrames = actual_response.into_iter().next().ok_or_else(|| ResponseError::ValueError("empty list".into()))??;

        assert!(actual_dataframes.is_empty());

        Ok(())
    }

    #[test]
    fn parse_ok_to_dataframes_and_tags() -> Result<(), ResponseError> {
        let input = r#"{
            "results": [
                {
                    "statement_id": 15,
                    "series": [
                        {
                            "name":"mymeas",
                            "columns": ["time","myfield1","myfield2"],
                            "values": [
                                ["2017-03-01T00:16:18Z",33.1,12.5],["2017-03-01T00:17:18Z",12.4,12.7]
                            ],
                            "tags": {
                                "room": "bedroom"
                            }
                        }
                    ]
                }
            ]
        }"#;
        let mut expected_dataframe = DummyDataFrame {
            name: "mymeas".into(),
            index: vec![Utc.ymd(2017, 3, 1).and_hms(0, 16, 18), Utc.ymd(2017, 3, 1).and_hms(0, 17, 18)],
            columns: HashMap::new(),
        };
        expected_dataframe.columns.insert("myfield1".into(), vec![Value::Float(33.1), Value::Float(12.4)]);
        expected_dataframe.columns.insert("myfield2".into(), vec![Value::Float(12.5), Value::Float(12.7)]);

        let mut expected_tags = HashMap::new();
        expected_tags.insert("room".into(), "bedroom".into());

        let actual_response: Vec<Result<TaggedDataFrames, ResponseError>> = from_str(input)?;
        let actual_dataframes: TaggedDataFrames = actual_response.into_iter().next().ok_or_else(|| ResponseError::ValueError("empty list".into()))??;

        let (actual_dataframe, actual_tags): (DummyDataFrame, Option<Tags>) = actual_dataframes.into_iter().next().ok_or_else(|| ResponseError::ValueError("empty list".into()))?;

        assert_eq!(actual_tags, Some(expected_tags));

        assert_eq!(actual_dataframe, expected_dataframe);

        Ok(())
    }

}
