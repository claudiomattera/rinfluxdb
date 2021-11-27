// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

//! Types used by other modules

use std::fmt;

use tracing::*;

use thiserror::Error;

use chrono::{DateTime, SecondsFormat, Utc};

/// Value types supported by InfluxDB
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A floating point value
    Float(f64),

    /// An integer value
    Integer(i64),

    /// An unsigned integer value
    UnsignedInteger(u64),

    /// A string value
    String(String),

    /// A boolean value
    Boolean(bool),

    /// A datetime value (as nanosecond epoch)
    Timestamp(DateTime<Utc>),
}

impl Value {
    pub fn into_f64(self) -> f64 {
        if let Value::Float(value) = self {
            value
        } else if let Value::Integer(value) = self {
            value as f64
        } else if let Value::UnsignedInteger(value) = self {
            value as f64
        } else {
            panic!("Not a f64: {:?}", self);
        }
    }

    pub fn into_i64(self) -> i64 {
        if let Value::Integer(value) = self {
            value
        } else if let Value::UnsignedInteger(value) = self {
            value as i64
        } else if let Value::Float(value) = self {
            warn!("Casting float to integer");
            value as i64
        } else {
            panic!("Not a i64: {:?}", self);
        }
    }

    pub fn into_u64(self) -> u64 {
        if let Value::UnsignedInteger(value) = self {
            value
        } else {
            panic!("Not a u64: {:?}", self);
        }
    }

    pub fn into_boolean(self) -> bool {
        if let Value::Boolean(value) = self {
            value
        } else {
            panic!("Not a boolean: {:?}", self);
        }
    }

    pub fn into_string(self) -> String {
        if let Value::String(value) = self {
            value
        } else {
            panic!("Not a string: {:?}", self);
        }
    }

    pub fn into_timestamp(self) -> DateTime<Utc> {
        if let Value::Timestamp(value) = self {
            value
        } else {
            panic!("Not a timestamp: {:?}", self);
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Value::Float(value) => {
                write!(f, "{}", value)?;
            }
            Value::Integer(value) => {
                write!(f, "{}", value)?;
            }
            Value::UnsignedInteger(value) => {
                write!(f, "{}", value)?;
            }
            Value::String(value) => {
                write!(f, "{}", value)?;
            }
            Value::Boolean(value) => {
                write!(f, "{}", value)?;
            }
            Value::Timestamp(value) => {
                write!(f, "{}", value)?;
            }
        }

        Ok(())
    }
}

/// A duration
///
/// Note: this type is almost entirely equivalent to `chrono::Duration`, but
/// it also supports infinite duration in addition.
///
/// Since `chrono::Duration` implements `Into<Duration>`, the former can be
/// used everywhere the latter is expected.
#[derive(Debug)]
pub enum Duration {
    /// A duration expressed in nanoseconds
    Nanoseconds(i64),

    /// A duration expressed in microseconds
    Microseconds(i64),

    /// A duration expressed in milliseconds
    Milliseconds(i64),

    /// A duration expressed in seconds
    Seconds(i64),

    /// A duration expressed in minutes
    Minutes(i64),

    /// A duration expressed in hours
    Hours(i64),

    /// A duration expressed in days
    Days(i64),

    /// An infinite duration
    Infinity,
}

impl ToString for Duration {
    fn to_string(&self) -> String {
        match self {
            Duration::Nanoseconds(seconds) => format!("{}ns", seconds),
            Duration::Microseconds(seconds) => format!("{}us", seconds),
            Duration::Milliseconds(seconds) => format!("{}ms", seconds),
            Duration::Seconds(seconds) => format!("{}s", seconds),
            Duration::Minutes(minutes) => format!("{}m", minutes),
            Duration::Hours(hours) => format!("{}h", hours),
            Duration::Days(days) => format!("{}d", days),
            Duration::Infinity => "inf".to_string(),
        }
    }
}

impl From<chrono::Duration> for Duration {
    fn from(duration: chrono::Duration) -> Self {
        Duration::Seconds(duration.num_seconds())
    }
}

/// An entity which is either an instant or a duration
///
/// InfluxDB allows to use durations where instants are expected, and
/// interprets them as the point in time relative to the current instant.
/// E.g. if now is `2021-03-10T22:43:32Z`, the duration `Duration::Minutes(-4)`
/// is interpreted as the instant `2021-03-10T22:39:32Z`.
#[derive(Debug)]
pub enum InstantOrDuration {
    /// An instant in time
    Instant(DateTime<Utc>),

    /// The instant corresponding to the current time plus a duration
    Duration(Duration),
}

impl ToString for InstantOrDuration {
    fn to_string(&self) -> String {
        match self {
            InstantOrDuration::Instant(instant) => format!(
                "'{}'",
                instant.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            ),
            InstantOrDuration::Duration(duration) => duration.to_string(),
        }
    }
}

impl From<DateTime<Utc>> for InstantOrDuration {
    fn from(instant: DateTime<Utc>) -> Self {
        InstantOrDuration::Instant(instant)
    }
}

impl From<Duration> for InstantOrDuration {
    fn from(duration: Duration) -> Self {
        InstantOrDuration::Duration(duration)
    }
}

impl From<chrono::Duration> for InstantOrDuration {
    fn from(duration: chrono::Duration) -> Self {
        InstantOrDuration::Duration(duration.into())
    }
}

/// An error occurred while creating the dataframe
#[derive(Error, Debug)]
pub enum DataFrameError {
    /// Error while creating the dataframe
    #[error("Error while creating the dataframe")]
    Creation,
}
