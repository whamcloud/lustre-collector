// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{mds::mds_parser, mgs::mgs_parser, oss::oss_parser, top_level_parser, types::Record};
use combine::{choice, error::ParseError, many, Parser, Stream};

pub fn params() -> Vec<String> {
    let mut a = top_level_parser::top_level_params();
    a.extend(mgs_parser::params());
    a.extend(oss_parser::params());
    a.extend(mds_parser::params());
    a
}

pub fn parse<I>() -> impl Parser<I, Output = Vec<Record>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many(choice((
        top_level_parser::parse(),
        mgs_parser::parse(),
        mds_parser::parse(),
        oss_parser::parse(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_params() {
        assert_debug_snapshot!(params());
    }
}
