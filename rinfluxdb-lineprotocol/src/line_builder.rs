// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use ::chrono::{DateTime, Utc};

use super::FieldName;
use super::FieldValue;
use super::Line;
use super::Measurement;
use super::TagName;
use super::TagValue;

/// Build a record
#[derive(Clone, Debug, PartialEq)]
pub struct LineBuilder {
    line: Line,
}

impl LineBuilder {
    /// Create a new line for a measurement
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::LineBuilder;
    /// let line = LineBuilder::new("measurement")
    ///     .build();
    /// assert_eq!(line.measurement(), &"measurement".into());
    /// ```
    pub fn new(measurement: impl Into<Measurement>) -> Self {
        Self {
            line: Line::new(measurement),
        }
    }

    /// Insert a field in the line
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::LineBuilder;
    /// let line = LineBuilder::new("measurement")
    ///     .insert_field("latitude", 55.383333)
    ///     .insert_field("longitude", 10.383333)
    ///     .build();
    /// assert_eq!(line.field("latitude"), Some(&55.383333.into()));
    /// assert_eq!(line.field("longitude"), Some(&10.383333.into()));
    /// ```
    pub fn insert_field(self, name: impl Into<FieldName>, value: impl Into<FieldValue>) -> Self {
        let mut line = self.line;
        line.insert_field(name, value);
        Self { line }
    }

    /// Insert a tag in the line
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::LineBuilder;
    /// let line = LineBuilder::new("measurement")
    ///     .insert_tag("city", "Odense")
    ///     .build();
    /// assert_eq!(line.tag("city"), Some(&"Odense".into()));
    /// ```
    pub fn insert_tag(self, name: impl Into<TagName>, value: impl Into<TagValue>) -> Self {
        let mut line = self.line;
        line.insert_tag(name, value);
        Self { line }
    }

    /// Set the line timestamp
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::LineBuilder;
    /// # use chrono::{TimeZone, Utc};
    /// let line = LineBuilder::new("measurement")
    ///     .set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
    ///     .build();
    /// assert_eq!(line.timestamp(), Some(&Utc.ymd(2014, 7, 8).and_hms(9, 10, 11)));
    /// ```
    pub fn set_timestamp(self, timestamp: DateTime<Utc>) -> Self {
        let mut line = self.line;
        line.set_timestamp(timestamp);
        Self { line }
    }

    /// Build the line
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::LineBuilder;
    /// # use chrono::{TimeZone, Utc};
    /// let line = LineBuilder::new("measurement")
    ///     .insert_field("latitude", 55.383333)
    ///     .insert_field("longitude", 10.383333)
    ///     .insert_tag("city", "Odense")
    ///     .set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
    ///     .build();
    /// assert_eq!(line.measurement(), &"measurement".into());
    /// assert_eq!(line.field("latitude"), Some(&55.383333.into()));
    /// assert_eq!(line.field("longitude"), Some(&10.383333.into()));
    /// assert_eq!(line.tag("city"), Some(&"Odense".into()));
    /// assert_eq!(line.timestamp(), Some(&Utc.ymd(2014, 7, 8).and_hms(9, 10, 11)));
    /// ```
    pub fn build(self) -> Line {
        self.line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    #[test]
    fn create_record() {
        let actual = LineBuilder::new("location")
            .insert_tag("city", "Odense")
            .insert_field("latitude", FieldValue::Float(55.383333))
            .insert_field("longitude", FieldValue::Float(10.383333))
            .set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11))
            .build();

        let mut expected = Line::new("location");
        expected.insert_tag("city", "Odense");
        expected.insert_field("latitude", FieldValue::Float(55.383333));
        expected.insert_field("longitude", FieldValue::Float(10.383333));
        expected.set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11));

        assert_eq!(actual, expected);
    }
}
