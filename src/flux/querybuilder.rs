// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use std::fmt::Write;

use crate::types::{Duration, InstantOrDuration};

use super::query::Query;

#[derive(Debug)]
enum Statement {
    Range(InstantOrDuration, InstantOrDuration),
    RangeStart(InstantOrDuration),
    RangeStop(InstantOrDuration),
    Filter(String),
    Window(Duration),
    Aggregate(String),
    Duplicate(String, String),
    AggregateWindow(String, Duration),
}

/// A builder for Flux queries
///
/// ```
/// # use rinfluxdb::types::Duration;
/// # use rinfluxdb::flux::QueryBuilder;
/// let query = QueryBuilder::from("telegraf/autogen")
///     .range_start(Duration::Minutes(-15))
///     .filter(
///         r#"r._measurement == "cpu" and
///         r._field == "usage_system" and
///         r.cpu == "cpu-total""#
///     )
///     .build();
///
/// assert_eq!(
///     query.as_ref(),
///     r#"from(bucket: "telegraf/autogen")
///   |> range(start: -15m)
///   |> filter(fn: (r) =>
///     r._measurement == "cpu" and
///     r._field == "usage_system" and
///     r.cpu == "cpu-total"
///   )
///   |> yield()"#,
/// );
/// ```
pub struct QueryBuilder {
    bucket: String,
    statements: Vec<Statement>,
}

impl QueryBuilder {
    /// Create a query selecting from a bucket.
    pub fn from<T>(bucket: T) -> Self
    where T: Into<String> {
        Self { bucket: bucket.into(), statements: vec![] }
    }

    fn statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    /// Restrict query results to a start time
    pub fn range_start<T>(mut self, start: T) -> Self
    where T: Into<InstantOrDuration> {
        self.statement(Statement::RangeStart(start.into()));
        self
    }

    /// Restrict query results to a stop time
    pub fn range_stop<T>(mut self, stop: T) -> Self
    where T: Into<InstantOrDuration> {
        self.statement(Statement::RangeStop(stop.into()));
        self
    }

    /// Restrict query results to a between two instants
    pub fn range<T, S>(mut self, start: T, stop: S) -> Self
    where
        T: Into<InstantOrDuration>,
        S: Into<InstantOrDuration>,
    {
        self.statement(Statement::Range(start.into(), stop.into()));
        self
    }

    /// Add a filter to the query
    pub fn filter<T>(mut self, filter: T) -> Self
    where T: Into<String> {
        self.statement(Statement::Filter(filter.into()));
        self
    }

    /// Add a window to the query
    pub fn window<T>(mut self, every: T) -> Self
    where T: Into<Duration> {
        self.statement(Statement::Window(every.into()));
        self
    }

    /// Aggregate results
    pub fn aggregate<T>(mut self, fn_: T) -> Self
    where T: Into<String> {
        self.statement(Statement::Aggregate(fn_.into()));
        self
    }

    /// Aggregate results using the `mean` function
    pub fn mean(self) -> Self {
        self.aggregate("mean")
    }

    /// Duplicate fields
    pub fn duplicate<T, S>(mut self, column: T, as_: S) -> Self
    where
        T: Into<String>,
        S: Into<String>,
    {
        // let statement = Statement::new("duplicate")
        //     .argument("column", "\"".to_owned() + column.as_ref() + "\"")
        //     .argument("as", "\"".to_owned() + as_.as_ref() + "\"");
        self.statement(Statement::Duplicate(column.into(), as_.into()));
        self
    }

    /// Aggregate results over a window
    pub fn aggregate_window<T, S>(mut self, fn_: S, every: Duration) -> Self
    where
        T: Into<String>,
        S: Into<String>,
    {
        self.statement(Statement::AggregateWindow(fn_.into(), every));
        self
    }

    /// Create the Flux query
    pub fn build(self) -> Query {
        let mut result = String::new();

        writeln!(&mut result, "from(bucket: \"{}\")", self.bucket).unwrap();

        for statement in self.statements {

            // TODO: Return error if vecs have not expected number of arguments
            match statement {
                Statement::Range(start, stop) => writeln!(&mut result, "  |> range(start: {}, stop: {})", start.to_string(), stop.to_string()).unwrap(),
                Statement::RangeStart(start) => writeln!(&mut result, "  |> range(start: {})", start.to_string()).unwrap(),
                Statement::RangeStop(stop) => writeln!(&mut result, "  |> range(stop: {})", stop.to_string()).unwrap(),
                Statement::Filter(filter) => {
                    writeln!(&mut result, "  |> filter(fn: (r) =>").unwrap();
                    for line in filter.lines() {
                        writeln!(&mut result, "    {}", line.trim_start()).unwrap();
                    }
                    writeln!(&mut result, "  )").unwrap();
                }
                Statement::Window(every) => writeln!(&mut result, "  |> window(every: {})", every.to_string()).unwrap(),
                Statement::Aggregate(fn_) => writeln!(&mut result, "  |> {}()", fn_).unwrap(),
                Statement::Duplicate(column, as_) => writeln!(&mut result, "  |> duplicate(column: \"{}\", as: \"{}\")", column, as_).unwrap(),
                Statement::AggregateWindow(fn_, every) => writeln!(&mut result, "  |> aggregate_window(fn: {}, every: {})", fn_, every.to_string()).unwrap(),
            }
        }

        write!(&mut result, "  |> yield()").unwrap();

        Query::new(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_query() {
        let expected = Query::new("from(bucket: \"telegraf/autogen\")
  |> range(start: -15m)
  |> yield()");

        let actual = QueryBuilder::from("telegraf/autogen")
            .range_start(Duration::Minutes(-15))
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_with_filter() {
        let expected = Query::new(r#"from(bucket: "telegraf/autogen")
  |> range(start: -15m)
  |> filter(fn: (r) =>
    r._measurement == "cpu" and
    r._field == "usage_system" and
    r.cpu == "cpu-total"
  )
  |> yield()"#);

        let actual = QueryBuilder::from("telegraf/autogen")
            .range_start(Duration::Minutes(-15))
            .filter(
                r#"r._measurement == "cpu" and
                r._field == "usage_system" and
                r.cpu == "cpu-total""#
            )
            .build();

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_query_with_statements() {
        let expected = Query::new(r#"from(bucket: "telegraf/autogen")
  |> range(start: -1h)
  |> filter(fn: (r) =>
    r._measurement == "cpu" and
    r._field == "usage_system" and
    r.cpu == "cpu-total"
  )
  |> window(every: 5m)
  |> mean()
  |> duplicate(column: "_stop", as: "_time")
  |> window(every: inf)
  |> yield()"#);

        let actual = QueryBuilder::from("telegraf/autogen")
            .range_start(Duration::Hours(-1))
            .filter(
                r#"r._measurement == "cpu" and
                r._field == "usage_system" and
                r.cpu == "cpu-total""#
            )
            .window(Duration::Minutes(5))
            .mean()
            .duplicate("_stop", "_time")
            .window(Duration::Infinity)
            .build();

        assert_eq!(actual, expected);
    }
}
