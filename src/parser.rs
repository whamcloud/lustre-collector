// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use mgs::mgs_parser;
use oss::oss_parser;
use top_level_parser;

use combine::{choice, error::ParseError, many, Parser, Stream};
use types::Record;

pub fn params() -> Vec<String> {
    let mut a = top_level_parser::top_level_params();
    a.extend(mgs_parser::params());
    a.extend(oss_parser::params());
    a
}

pub fn parse<I>() -> impl Parser<Input = I, Output = Vec<Record>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many(choice((
        top_level_parser::parse(),
        mgs_parser::parse(),
        oss_parser::parse(),
    )))
}
