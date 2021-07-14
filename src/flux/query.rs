// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

/// A Flux query
#[derive(Debug, PartialEq)]
pub struct Query(String);

impl Query {
    /// Create a query from a string-like object
    pub fn new<T>(query: T) -> Self
    where T: Into<String> {
        Self(query.into())
    }
}

impl AsRef<str> for Query {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
