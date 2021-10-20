// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

use std::collections::HashMap;

use super::ResponseError;

/// A set of tags and tag values
pub type TagsMap = HashMap<String, String>;

/// A dataframe accompanied by a set of tags and tag values
pub type TaggedDataframe<DF> = (DF, Option<TagsMap>);

/// The result of an individual statement from an InfluxQL query
pub type StatementResult<DF> = Result<Vec<TaggedDataframe<DF>>, ResponseError>;

/// The result of an entire InfluxQL query
pub type ResponseResult<DF> = Result<Vec<StatementResult<DF>>, ResponseError>;
