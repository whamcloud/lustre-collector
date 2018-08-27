// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    char::{alpha_num, digit, newline, string},
    error::ParseError,
    many1,
    parser::repeat::take_until,
    stream::Stream,
    token, try, unexpected, value, Parser,
};

use stats::Param;

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

pub fn not_word<I>(x: &'static str) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    try(word().then(move |y| {
        if x == y {
            unexpected(x).map(|_| "".to_string()).right()
        } else {
            value(y.to_string()).left()
        }
    }))
}

pub fn param<I>(x: &'static str) -> impl Parser<Input = I, Output = Param>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    try(string(x).skip(token('=')))
        .map(|x| Param(x.to_string()))
        .message("while getting param")
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::state::{SourcePosition, State};
    use stats::Param;

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
