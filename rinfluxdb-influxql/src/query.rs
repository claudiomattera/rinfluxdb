// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

/// An InfluxQL query
///
/// A query such as
///
/// * `SELECT temperature, humidity FROM house..indoor_environment`
/// * `SELECT temperature, humidity FROM house..indoor_environment WHERE time > now() - 1`
/// * `SELECT temperature, humidity FROM house..indoor_environment GROUP BY room`
#[derive(Debug, PartialEq)]
pub struct Query(String);

impl Query {
    /// Create a query from a string-like object
    pub fn new<T>(query: T) -> Self
    where
        T: Into<String>,
    {
        Self(query.into())
    }
}

impl AsRef<str> for Query {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
