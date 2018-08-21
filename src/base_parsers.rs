// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::error::ParseError;
use combine::parser::char::{alpha_num, digit, newline};
use combine::parser::repeat::take_until;
use combine::stream::Stream;
use combine::{many1, token, try, unexpected, value, Parser};

pub fn period<I>() -> impl Parser<Input = I, Output = char>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    token('.')
}

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

pub fn not_word<I>(x: &'static str) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    try(word().then(move |y| {
        if x.to_string() == y {
            unexpected(x).map(|_| "".to_string()).right()
        } else {
            value(y.to_string()).left()
        }
    }))
}
