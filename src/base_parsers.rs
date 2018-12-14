// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    attempt,
    char::{alpha_num, digit, newline, string},
    error::ParseError,
    many1, one_of,
    parser::repeat::take_until,
    stream::Stream,
    token, unexpected, value, Parser,
};

use crate::types::{Param, Target};

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

/// Parses a target name
pub fn target<I>() -> impl Parser<Input = I, Output = Target>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(alpha_num().or(one_of("_-".chars()))).map(Target)
}

/// Takes many consecutive digits and
/// returns them as u64
pub fn digits<I>() -> impl Parser<Input = I, Output = u64>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(digit()).map(|x: String| x.parse::<u64>().unwrap())
}

pub fn till_newline<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    take_until(newline())
}

pub fn string_to<I>(x: &'static str, y: &'static str) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string(x).map(move |_| String::from(y))
}

pub fn not_words<I>(xs: &'static [&'static str]) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    attempt(word().then(move |y| {
        for &x in xs {
            if x == y {
                return unexpected(x).map(|_| "".to_string()).right();
            }
        }

        value(y.to_string()).left()
    }))
}

pub fn param<I>(x: &'static str) -> impl Parser<Input = I, Output = Param>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    attempt(string(x).skip(token('=')))
        .map(|x| Param(x.to_string()))
        .message("while getting param")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Param;
    use combine::stream::state::{SourcePosition, State};

    #[test]
    fn test_param() {
        let result = param("memused").easy_parse(State::new("memused=77991501\n"));

        assert_eq!(
            result,
            Ok((
                Param("memused".to_string()),
                State {
                    input: "77991501\n",
                    positioner: SourcePosition { line: 1, column: 9 }
                }
            ))
        )
    }
}
