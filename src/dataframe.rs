// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

//! Dummy dataframe implementation

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

use chrono::{DateTime, Utc};

use crate::types::{ResponseError, Value};

/// Column type
#[derive(Clone, Debug, PartialEq)]
pub enum Column {
    /// A column of floating point values
    Float(Vec<f64>),

    /// A column of integer values
    Integer(Vec<i64>),

    /// A column of unsigned integer values
    UnsignedInteger(Vec<u64>),

    /// A column of string values
    String(Vec<String>),

    /// A column of boolean values
    Boolean(Vec<bool>),

    /// A column of datetime values
    Timestamp(Vec<DateTime<Utc>>),
}

impl Column {
    fn display_index(&self, index: usize, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Column::Float(values) => write!(f, "{:16}  ", values[index])?,
            Column::Integer(values) => write!(f, "{:16}  ", values[index])?,
            Column::UnsignedInteger(values) => write!(f, "{:16}  ", values[index])?,
            Column::String(values) => write!(f, "{:16}  ", values[index])?,
            Column::Boolean(values) => write!(f, "{:16}  ", values[index])?,
            Column::Timestamp(values) => write!(f, "{:16}  ", values[index])?,
        }

        Ok(())
    }
}

/// A time-indexed dataframe
///
/// A dataframe contains multiple named columns indexed by the same index.
#[derive(Clone, Debug)]
pub struct DataFrame {
    name: String,
    index: Vec<DateTime<Utc>>,
    columns: HashMap<String, Column>,
}

impl fmt::Display for DataFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:>23}  ", "datetime")?;
        for column in self.columns.keys() {
            write!(f, "{:>16}  ", column)?;
        }
        write!(f, "\n-----------------------  ")?;
        for _column in self.columns.keys() {
            write!(f, "----------------  ")?;
        }
        writeln!(f)?;

        for (i, index) in self.index.iter().enumerate() {
            write!(f, "{:>23}  ", index)?;
            for column in self.columns.values() {
                column.display_index(i, f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>)> for DataFrame {
    type Error = ResponseError;

    fn try_from((name, index, columns): (String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>)) -> Result<Self, Self::Error> {
        let columns = columns
            .into_iter()
            .map(|(name, column)| {
                let column = match column.first() {
                    Some(Value::Float(_)) => {
                        Column::Float(
                            column.into_iter()
                                .map(|element| element.into_f64())
                                .collect()
                        )
                    },
                    Some(Value::Integer(_)) => {
                        Column::Integer(
                            column.into_iter()
                                .map(|element| element.into_i64())
                                .collect()
                        )
                    },
                    Some(Value::UnsignedInteger(_)) => {
                        Column::UnsignedInteger(
                            column.into_iter()
                                .map(|element| element.into_u64())
                                .collect()
                        )
                    },
                    Some(Value::String(_)) => {
                        Column::String(
                            column.into_iter()
                                .map(|element| element.into_string())
                                .collect()
                        )
                    },
                    Some(Value::Boolean(_)) => {
                        Column::Boolean(
                            column.into_iter()
                                .map(|element| element.into_boolean())
                                .collect()
                        )
                    },
                    Some(Value::Timestamp(_)) => {
                        Column::Timestamp(
                            column.into_iter()
                                .map(|element| element.into_timestamp())
                                .collect()
                        )
                    },
                    None => panic!("Panic"),
                };
                (name, column)
            })
            .collect();
        Ok(Self { name, index, columns })
    }
}
