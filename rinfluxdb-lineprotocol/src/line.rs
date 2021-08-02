// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use ::std::collections::HashMap;

use ::std::fmt;

use ::chrono::{DateTime, Utc};

use super::FieldName;
use super::FieldValue;
use super::Measurement;
use super::TagName;
use super::TagValue;

/// A line in the Influx Line Protocol
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    measurement: Measurement,
    fields: HashMap<FieldName, FieldValue>,
    tags: HashMap<TagName, TagValue>,
    timestamp: Option<DateTime<Utc>>,
}

impl Line {
    /// Create a new line for a measurement
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// let line = Line::new("measurement");
    /// assert_eq!(line.measurement(), &"measurement".into());
    /// ```
    pub fn new(measurement: impl Into<Measurement>) -> Self {
        Self {
            measurement: measurement.into(),
            fields: HashMap::new(),
            tags: HashMap::new(),
            timestamp: None,
        }
    }

    /// Return the measurement
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// let line = Line::new("measurement");
    /// assert_eq!(line.measurement(), &"measurement".into());
    /// ```
    pub fn measurement(&self) -> &Measurement {
        &self.measurement
    }

    /// Insert a field in the line
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// # use rinfluxdb_lineprotocol::FieldValue;
    /// let mut line = Line::new("measurement");
    /// line.insert_field("latitude", FieldValue::Float(55.383333));
    /// line.insert_field("longitude", FieldValue::Float(10.383333));
    /// assert_eq!(line.field("latitude"), Some(&55.383333.into()));
    /// assert_eq!(line.field("longitude"), Some(&10.383333.into()));
    /// ```
    pub fn insert_field(&mut self, name: impl Into<FieldName>, value: impl Into<FieldValue>) {
        self.fields.insert(name.into(), value.into());
    }

    /// Return the value of a field
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// let mut line = Line::new("measurement");
    /// line.insert_field("latitude", 55.383333);
    /// line.insert_field("longitude", 10.383333);
    /// assert_eq!(line.field("latitude"), Some(&55.383333.into()));
    /// assert_eq!(line.field("longitude"), Some(&10.383333.into()));
    /// ```
    pub fn field(&self, name: impl Into<FieldName>) -> Option<&FieldValue> {
        self.fields.get(&name.into())
    }

    /// Insert a tag in the line
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// let mut line = Line::new("measurement");
    /// line.insert_tag("city", "Odense");
    /// assert_eq!(line.tag("city"), Some(&"Odense".into()));
    /// ```
    pub fn insert_tag(&mut self, name: impl Into<TagName>, value: impl Into<TagValue>) {
        self.tags.insert(name.into(), value.into());
    }

    /// Return the value of a tag
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// let mut line = Line::new("measurement");
    /// line.insert_tag("city", "Odense");
    /// assert_eq!(line.tag("city"), Some(&"Odense".into()));
    /// ```
    pub fn tag(&self, name: impl Into<TagName>) -> Option<&TagValue> {
        self.tags.get(&name.into())
    }

    /// Set the line timestamp
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// # use chrono::{TimeZone, Utc};
    /// let mut line = Line::new("measurement");
    /// line.set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11));
    /// assert_eq!(line.timestamp(), Some(&Utc.ymd(2014, 7, 8).and_hms(9, 10, 11)));
    /// ```
    pub fn set_timestamp(&mut self, timestamp: DateTime<Utc>) {
        self.timestamp = Some(timestamp);
    }

    /// Return the line timestamp
    ///
    /// ```
    /// # use rinfluxdb_lineprotocol::Line;
    /// # use chrono::{TimeZone, Utc};
    /// let mut line = Line::new("measurement");
    /// line.set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11));
    /// assert_eq!(line.timestamp(), Some(&Utc.ymd(2014, 7, 8).and_hms(9, 10, 11)));
    /// ```
    pub fn timestamp(&self) -> Option<&DateTime<Utc>> {
        self.timestamp.as_ref()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fields_vector: Vec<String> = self
            .fields
            .iter()
            .map(|(key, value)| {
                format!(
                    "{}={}",
                    key.escape_to_line_protocol(),
                    value.escape_to_line_protocol()
                )
            })
            .collect();
        fields_vector.sort();
        let fields_chunk = fields_vector.join(",");

        write!(f, "{}", self.measurement.escape_to_line_protocol())?;

        for (tag_name, tag_value) in self.tags.iter() {
            write!(
                f,
                ",{}={}",
                tag_name.escape_to_line_protocol(),
                tag_value.escape_to_line_protocol()
            )?;
        }

        write!(f, " {}", fields_chunk)?;

        if self.timestamp.is_some() {
            write!(f, " {}", self.timestamp.unwrap().timestamp_nanos())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    // use fake::{Fake, Faker};
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    impl Arbitrary for Line {
        fn arbitrary(g: &mut Gen) -> Self {
            let measurement = String::arbitrary(g);
            let tags: Vec<(TagName, TagValue)> = Vec::arbitrary(g);

            let mut line = Line::new(measurement);

            for (tag_name, tag_value) in tags {
                line.insert_tag(tag_name, tag_value);
            }

            line
        }
    }

    #[test]
    fn display_line() {
        let mut line = Line::new("location");

        line.insert_tag("city", "Odense");
        line.insert_field("latitude", FieldValue::Float(55.383333));
        line.insert_field("longitude", FieldValue::Float(10.383333));
        line.set_timestamp(Utc.ymd(2014, 7, 8).and_hms(9, 10, 11));

        let expected = "location,city=Odense latitude=55.383333,longitude=10.383333 1404810611000000000";

        assert_eq!(line.to_string(), expected);
    }

    #[quickcheck]
    #[ignore]
    fn display_line_quickcheck(line: Line) {
        let mut expected = line.measurement.escape_to_line_protocol();
        for (tag_name, tag_value) in line.tags.iter() {
            expected.push_str(&tag_name.escape_to_line_protocol());
            expected.push('=');
            expected.push_str(&tag_value.escape_to_line_protocol());
        }
        expected.push(' ');

        assert_eq!(line.to_string(), expected);
    }
}
