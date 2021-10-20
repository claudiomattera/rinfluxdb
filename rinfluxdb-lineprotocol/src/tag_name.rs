// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

/// Represent a tag name
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagName(String);

impl TagName {
    /// Escape a tag value to [InfluxDB line protocol](https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/)
    ///
    /// Characters ` `, `,` and `=` are escaped.
    pub fn escape_to_line_protocol(&self) -> String {
        self.0
            .replace(" ", "\\ ")
            .replace(",", "\\,")
            .replace("=", "\\=")
    }
}

impl From<&str> for TagName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for TagName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for TagName {
        fn arbitrary(g: &mut Gen) -> Self {
            let name = String::arbitrary(g);
            TagName(name)
        }
    }
}
