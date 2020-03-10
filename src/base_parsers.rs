// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    attempt,
    error::ParseError,
    many1, one_of,
    parser::{
        char::{alpha_num, digit, newline, string},
        repeat::take_until,
    },
    stream::Stream,
    token, unexpected, value, Parser,
};

use crate::types::{Param, Target};

pub fn period<I>() -> impl Parser<I, Output = char>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    token('.')
}

pub fn equals<I>() -> impl Parser<I, Output = char>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    token('=')
}

pub fn word<I>() -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1(alpha_num().or(token('_')))
}

pub fn words<I>() -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1(alpha_num().or(token('_').or(token(' '))))
}

/// Parses a target name
pub fn target<I>() -> impl Parser<I, Output = Target>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1(alpha_num().or(one_of("_-".chars()))).map(Target)
}

/// Takes many consecutive digits and
/// returns them as u64
pub fn digits<I>() -> impl Parser<I, Output = u64>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1(digit()).map(|x: String| x.parse::<u64>().unwrap())
}

pub fn till_newline<I>() -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    take_until(newline())
}

pub fn till_period<I>() -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    take_until(period())
}

pub fn till_equals<I>() -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    take_until(equals())
}

pub fn string_to<I>(x: &'static str, y: &'static str) -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    string(x).map(move |_| String::from(y))
}

pub fn not_words<I>(xs: &'static [&'static str]) -> impl Parser<I, Output = String>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(word().then(move |y| {
        for &x in xs {
            if x == y {
                return unexpected(x).map(|_| "".to_string()).right();
            }
        }

        value(y).left()
    }))
}

pub fn param<I>(x: &'static str) -> impl Parser<I, Output = Param>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    attempt(string(x).skip(token('=')))
        .map(|x| Param(x.to_string()))
        .message("while getting param")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Param;

    #[test]
    fn test_param() {
        let result = param("memused").parse("memused=77991501\n");

        assert_eq!(result, Ok((Param("memused".to_string()), "77991501\n")))
    }
}
