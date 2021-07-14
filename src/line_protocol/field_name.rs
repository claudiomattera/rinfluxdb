// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

/// Represent a field value
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldName(String);

impl FieldName {
    /// Escape a field name to [InfluxDB line protocol](https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/)
    ///
    /// The name is enclosed in double quotes, and characters ` `, `,` and `=` are escaped.
    pub fn escape_to_line_protocol(&self) -> String {
        self.0
            .replace(" ", "\\ ")
            .replace(",", "\\,")
            .replace("=", "\\=")
    }
}

impl From<&str> for FieldName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for FieldName {
    fn from(s: String) -> Self {
        Self(s)
    }
}
