// Copyright (c) 2023 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{base_parsers::period, Record};
use combine::{parser::char::string, ParseError, Parser, Stream};

mod qmt_parser;

pub(crate) const QMT: &str = "qmt";

pub(crate) fn params() -> Vec<String> {
    qmt_parser::params()
}

pub(crate) fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (string(QMT), period()).with(qmt_parser::parse())
}
