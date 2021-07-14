// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use ::chrono::{DateTime, Utc};

/// Represent a field value
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Represent a floating point number field value
    Float(f64),

    /// Represent a signed integer number field value
    Integer(i64),

    /// Represent an unsigned integer number field value
    UnsignedInteger(u64),

    /// Represent a string field value
    String(String),

    /// Represent a boolean field value
    Boolean(bool),

    /// Represent an instant field value
    ///
    /// InfluxDB does not natively support instants as field values, so this is
    /// represented as a nanosecond timestamp.
    Timestamp(DateTime<Utc>),
}

impl FieldValue {
    /// Escape a field value to [InfluxDB line protocol](https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/)
    ///
    /// Numeric and boolean values are escaped as they are.
    /// Timestamps are converted to nanoseconds from epoch.
    /// Strings are enclosed in double quotes, and characters `"` and `\` are escaped.
    ///
    /// ```
    /// # use rinfluxdb::line_protocol::FieldValue;
    /// let mut value = FieldValue::String("a string \"value\"".into());
    /// assert_eq!(value.escape_to_line_protocol(), "\"a string \\\\\"value\\\\\"\"".to_string());
    /// ```
    pub fn escape_to_line_protocol(&self) -> String {
        match self {
            FieldValue::Float(f) => format!("{}", f),
            FieldValue::Integer(i) => format!("{}", i),
            FieldValue::UnsignedInteger(u) => format!("{}", u),
            FieldValue::String(s) => {
                format!("\"{}\"", s.replace("\"", "\\\"").replace("\\", "\\\\"))
            }
            FieldValue::Boolean(true) => "true".to_string(),
            FieldValue::Boolean(false) => "false".to_string(),
            FieldValue::Timestamp(ts) => format!("{}i", ts.timestamp_nanos()),
        }
    }
}

impl From<&str> for FieldValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for FieldValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<u64> for FieldValue {
    fn from(value: u64) -> Self {
        Self::UnsignedInteger(value)
    }
}

impl From<bool> for FieldValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<DateTime<Utc>> for FieldValue {
    fn from(value: DateTime<Utc>) -> Self {
        Self::Timestamp(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fake::{Fake, Faker};
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    #[derive(Debug, Clone)]
    struct PositiveInteger(pub i64);

    impl Arbitrary for PositiveInteger {
        fn arbitrary(g: &mut Gen) -> Self {
            let integer = i64::arbitrary(g);
            if integer >= 0 {
                Self(integer)
            } else {
                Self(integer.wrapping_neg())
            }
        }
    }

    #[derive(Debug, Clone)]
    struct NegativeInteger(pub i64);

    impl Arbitrary for NegativeInteger {
        fn arbitrary(g: &mut Gen) -> Self {
            let integer = i64::arbitrary(g);
            if integer < 0 {
                Self(integer)
            } else {
                Self(integer.wrapping_neg())
            }
        }
    }

    #[test]
    fn escape_integer() {
        let value = Faker.fake::<i64>();
        let field_value = FieldValue::Integer(value);
        let expected = value.to_string();

        assert_eq!(field_value.escape_to_line_protocol(), expected);
    }

    #[quickcheck]
    fn escape_integer_quickcheck(positive_integer: PositiveInteger) {
        let value = positive_integer.0;
        let field_value = FieldValue::Integer(value);
        let expected = value.to_string();

        assert_eq!(field_value.escape_to_line_protocol(), expected);
    }

    #[test]
    fn escape_negative_integer() {
        let field_value = FieldValue::Integer(-55);

        assert_eq!(field_value.escape_to_line_protocol(), "-55");
    }

    #[quickcheck]
    fn escape_negative_integer_quickcheck(negative_integer: NegativeInteger) {
        let value = negative_integer.0;
        let field_value = FieldValue::Integer(value);
        let expected = value.to_string();

        assert_eq!(field_value.escape_to_line_protocol(), expected);
    }

    #[test]
    fn escape_boolean() {
        let value = FieldValue::Boolean(true);

        assert_eq!(value.escape_to_line_protocol(), "true");
    }

    #[quickcheck]
    fn escape_boolean_quickcheck(value: bool) {
        let field_value = FieldValue::Boolean(value);
        let expected = value.to_string();

        assert_eq!(field_value.escape_to_line_protocol(), expected);
    }

    #[test]
    fn escape_float() {
        let value = FieldValue::Float(12.5);

        assert_eq!(value.escape_to_line_protocol(), "12.5");
    }

    #[quickcheck]
    fn escape_float_quickcheck(value: f64) {
        let field_value = FieldValue::Float(value);
        let expected = value.to_string();

        assert_eq!(field_value.escape_to_line_protocol(), expected);
    }

    #[test]
    fn escape_string() {
        let value = FieldValue::String("a string \"value\"".into());

        assert_eq!(
            value.escape_to_line_protocol(),
            "\"a string \\\\\"value\\\\\"\""
        );
    }

    #[quickcheck]
    fn escape_string_quickcheck(value: String) {
        let field_value = FieldValue::String(value.clone());
        let expected = format!("\"{}\"", value.replace("\"", "\\\"").replace("\\", "\\\\"));

        assert_eq!(field_value.escape_to_line_protocol(), expected);
    }
}
