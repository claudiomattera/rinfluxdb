// Copyright Claudio Mattera 2021.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use super::ResponseError;

/// The result of an entire InfluxQL query
pub type ResponseResult<DF> = Result<DF, ResponseError>;
