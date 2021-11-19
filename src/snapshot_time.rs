// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::base_parsers::{digits, till_newline};
use combine::error::ParseError;
use combine::parser::char::{spaces, string};
use combine::stream::Stream;
use combine::{optional, token, Parser};

pub(crate) fn snapshot_time<I>() -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        string("snapshot_time").skip(optional(token(':'))),
        spaces(),
        digits().skip(token('.')),
        digits().skip(till_newline()),
    )
        .map(|(_, _, secs, nsecs)| format!("{}.{}", secs, nsecs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_time() {
        let x = r#"snapshot_time:         1534158712.738772898 (secs.nsecs)
"#;

        let result = snapshot_time().parse(x);

        assert_eq!(result, Ok(("1534158712.738772898".to_string(), "\n",)));
    }
    #[test]
    fn test_snapshot_time_no_colon() {
        let x = r#"snapshot_time             1534769431.137892896 secs.nsecs
"#;

        let result = snapshot_time().parse(x);

        assert_eq!(result, Ok(("1534769431.137892896".to_string(), "\n")));
    }
}
