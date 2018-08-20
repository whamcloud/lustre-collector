// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::error::ParseError;
use combine::parser::char::{alpha_num, digit, newline};
use combine::parser::repeat::take_until;
use combine::stream::Stream;
use combine::{many1, token, Parser};

pub fn word<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(alpha_num().or(token('_')))
}

pub fn digits<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(digit())
}

pub fn till_newline<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    take_until(newline())
}
