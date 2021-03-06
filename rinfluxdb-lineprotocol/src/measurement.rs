// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

/// Represent a measurement
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Measurement(String);

impl Measurement {
    /// Escape a measurement to [InfluxDB line protocol](https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_reference/)
    ///
    /// The name is enclosed in double quotes, and characters ` `, `,` and `=` are escaped.
    pub fn escape_to_line_protocol(&self) -> String {
        self.0
            .replace(" ", "\\ ")
            .replace(",", "\\,")
            .replace("=", "\\=")
    }
}

impl From<&str> for Measurement {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Measurement {
    fn from(s: String) -> Self {
        Self(s)
    }
}
