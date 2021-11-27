// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use super::ResponseError;

/// The result of an entire InfluxQL query
pub type ResponseResult<DF> = Result<DF, ResponseError>;
