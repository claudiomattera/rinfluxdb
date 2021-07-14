// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

/// Represent a tag value
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagValue(String);

impl TagValue {
    /// Escape a tag value to [InfluxDB line protocol](https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/)
    ///
    /// Characters ` `, `,` and `\` are escaped.
    pub fn escape_to_line_protocol(&self) -> String {
        self.0
            .replace(" ", "\\ ")
            .replace(",", "\\,")
            .replace("=", "\\=")
    }
}

impl From<&str> for TagValue {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for TagValue {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for TagValue {
        fn arbitrary(g: &mut Gen) -> Self {
            let value = String::arbitrary(g);
            TagValue(value)
        }
    }
}
