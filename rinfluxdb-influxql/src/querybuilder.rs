// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::fmt::Write;

use chrono::{DateTime, SecondsFormat, Utc};

use super::query::Query;

/// A builder for InfluxQL queries
///
/// ```
/// # use rinfluxdb_influxql::QueryBuilder;
/// # use chrono::{TimeZone, Utc};
/// let query = QueryBuilder::from("indoor_environment")
///     .field("temperature")
///     .field("humidity")
///     .start(Utc.ymd(2021, 3, 7).and_hms(21, 0, 0))
///     .build();
///
/// assert_eq!(
///     query.as_ref(),
///     "SELECT temperature, humidity \
///     FROM indoor_environment \
///     WHERE time > '2021-03-07T21:00:00Z'",
/// );
/// ```
pub struct QueryBuilder {
    measurement: String,
    database: Option<String>,
    retention_policy: Option<String>,
    fields: Vec<String>,
    start: Option<DateTime<Utc>>,
    stop: Option<DateTime<Utc>>,
    groups: Vec<String>,
}

impl QueryBuilder {
    /// Create a query selecting from a measurement
    ///
    /// This sets the measurement in the `FROM` clause.
    pub fn from<T>(measurement: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            measurement: measurement.into(),
            database: None,
            retention_policy: None,
            fields: Vec::new(),
            start: None,
            stop: None,
            groups: Vec::new(),
        }
    }

    /// Set the database
    ///
    /// This sets the database in the `FROM` clause:
    /// `database.retention_policy.measurement`.
    pub fn database<T>(mut self, database: T) -> Self
    where
        T: Into<String>,
    {
        self.database = Some(database.into());
        self
    }

    /// Set the retention policy
    ///
    /// This sets the retention policy in the `FROM` clause:
    /// `database.retention_policy.measurement`.
    pub fn retention_policy<T>(mut self, retention_policy: T) -> Self
    where
        T: Into<String>,
    {
        self.retention_policy = Some(retention_policy.into());
        self
    }

    /// Add a field to the query
    pub fn field<T>(mut self, field: T) -> Self
    where
        T: Into<String>,
    {
        self.fields.push(field.into());
        self
    }

    /// Restrict query results to a start time
    pub fn start<T>(mut self, start: T) -> Self
    where
        T: Into<DateTime<Utc>>,
    {
        self.start = Some(start.into());
        self
    }

    /// Restrict query results to a stop time
    pub fn stop<T>(mut self, stop: T) -> Self
    where
        T: Into<DateTime<Utc>>,
    {
        self.stop = Some(stop.into());
        self
    }

    /// Group by a tag
    pub fn group_by<T>(mut self, tag: T) -> Self
    where
        T: Into<String>,
    {
        self.groups.push(tag.into());
        self
    }

    /// Create the InfluxQL query
    pub fn build(self) -> Query {
        let mut result = String::new();

        write!(&mut result, "SELECT ").unwrap();

        let mut fields = self.fields.into_iter();

        // TODO: Return error if vecs have not expected number of arguments
        let first_field = fields.next();
        match first_field {
            Some(first_field) => {
                write!(&mut result, "{}", first_field).unwrap();
                for field in fields {
                    write!(&mut result, ", {}", field).unwrap();
                }
            }
            None => write!(&mut result, "*").unwrap(),
        }

        match (self.database, self.retention_policy) {
            (Some(database), Some(retention_policy)) => write!(
                &mut result,
                " FROM {}.{}.{}",
                database,
                retention_policy,
                self.measurement,
            )
            .unwrap(),
            (Some(database), None) => write!(
                &mut result,
                " FROM {}..{}",
                database,
                self.measurement,
            )
            .unwrap(),
            (None, Some(retention_policy)) => write!(
                &mut result,
                " FROM .{}.{}",
                retention_policy,
                self.measurement,
            )
            .unwrap(),
            (None, None) => write!(&mut result, " FROM {}", self.measurement).unwrap(),
        }

        if self.start.is_some() || self.stop.is_some() {
            write!(&mut result, " WHERE").unwrap();

            match (self.start, self.stop) {
                (Some(start), Some(stop)) => write!(
                    &mut result,
                    " time > '{}' AND time < '{}'",
                    start.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                    stop.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                )
                .unwrap(),
                (Some(start), None) => write!(
                    &mut result,
                    " time > '{}'",
                    start.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                )
                .unwrap(),
                (None, Some(stop)) => write!(
                    &mut result,
                    " time < '{}'",
                    stop.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                )
                .unwrap(),
                (None, None) => unreachable!(),
            }
        }

        if !self.groups.is_empty() {
            write!(&mut result, " GROUP BY").unwrap();

            let mut group = self.groups.into_iter();

            // TODO: Return error if vecs have not expected number of arguments
            let first_group = group.next();
            match first_group {
                Some(first_group) => {
                    write!(&mut result, " {}", first_group).unwrap();
                    for group in group {
                        write!(&mut result, ", {}", group).unwrap();
                    }
                }
                None => unreachable!(),
            }

        }

        Query::new(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    #[test]
    fn simple_query() {
        let expected = Query::new(
            "SELECT temperature, humidity \
            FROM indoor_environment",
        );

        let actual = QueryBuilder::from("indoor_environment")
            .field("temperature")
            .field("humidity")
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_all_fields() {
        let expected = Query::new(
            "SELECT * \
            FROM indoor_environment",
        );

        let actual = QueryBuilder::from("indoor_environment")
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_with_range() {
        let expected = Query::new(
            "SELECT temperature, humidity \
            FROM indoor_environment \
            WHERE time > '2021-03-07T21:00:00Z' AND time < '2021-03-07T22:00:00Z'",
        );

        let actual = QueryBuilder::from("indoor_environment")
            .field("temperature")
            .field("humidity")
            .start(Utc.ymd(2021, 3, 7).and_hms(21, 0, 0))
            .stop(Utc.ymd(2021, 3, 7).and_hms(22, 0, 0))
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_with_start() {
        let expected = Query::new(
            "SELECT temperature, humidity \
            FROM indoor_environment \
            WHERE time > '2021-03-07T21:00:00Z'",
        );

        let actual = QueryBuilder::from("indoor_environment")
            .field("temperature")
            .field("humidity")
            .start(Utc.ymd(2021, 3, 7).and_hms(21, 0, 0))
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_with_stop() {
        let expected = Query::new(
            "SELECT temperature, humidity \
            FROM indoor_environment \
            WHERE time < '2021-03-07T22:00:00Z'",
        );

        let actual = QueryBuilder::from("indoor_environment")
            .field("temperature")
            .field("humidity")
            .stop(Utc.ymd(2021, 3, 7).and_hms(22, 0, 0))
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_with_stop_and_groups() {
        let expected = Query::new(
            "SELECT temperature, humidity \
            FROM indoor_environment \
            WHERE time < '2021-03-07T22:00:00Z' \
            GROUP BY room",
        );

        let actual = QueryBuilder::from("indoor_environment")
            .field("temperature")
            .field("humidity")
            .stop(Utc.ymd(2021, 3, 7).and_hms(22, 0, 0))
            .group_by("room")
            .build();

        assert_eq!(actual, expected);
    }
}
