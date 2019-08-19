// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    between,
    error::ParseError,
    many1,
    parser::{
        char::{newline, spaces, string},
        choice::or,
    },
    stream::Stream,
    token, Parser,
};

use crate::{
    base_parsers::{digits, not_words, word},
    snapshot_time::snapshot_time,
    types::Stat,
};

fn name_count_units<I>() -> impl Parser<Input = I, Output = (String, u64, String)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        not_words(&["obdfilter", "mgs", "mdt"]).skip(spaces()),
        digits(),
        spaces().with(string("samples")),
        spaces().with(between(token('['), token(']'), word())),
    )
        .map(|(x, y, _, z)| (x, y, z))
}

fn min_max_sum<I>() -> impl Parser<Input = I, Output = (u64, u64, u64)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().with(digits()),
    )
}

fn sum_sq<I>() -> impl Parser<Input = I, Output = u64>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    spaces().with(digits())
}

fn stat<I>() -> impl Parser<Input = I, Output = Stat>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        name_count_units(),
        or(
            newline().map(|_| (None, None)),
            (
                min_max_sum().map(Some),
                or(newline().map(|_| None), sum_sq().map(Some).skip(newline())),
            ),
        ),
    )
        .map(
            |((name, samples, units), (min_max, sum))| match (min_max, sum) {
                (Some((min, max, sum)), Some(sumsquare)) => Stat {
                    name,
                    samples,
                    units,
                    min: Some(min),
                    max: Some(max),
                    sum: Some(sum),
                    sumsquare: Some(sumsquare),
                },
                (Some((min, max, sum)), None) => Stat {
                    name,
                    samples,
                    units,
                    min: Some(min),
                    max: Some(max),
                    sum: Some(sum),
                    sumsquare: None,
                },
                (None, _) => Stat {
                    name,
                    samples,
                    units,
                    min: None,
                    max: None,
                    sum: None,
                    sumsquare: None,
                },
            },
        )
}

pub fn stats<I>() -> impl Parser<Input = I, Output = Vec<Stat>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (newline().with(snapshot_time()), newline(), many1(stat())).map(|(_, _, xs)| xs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::state::{SourcePosition, State};
    use insta::assert_debug_snapshot_matches;

    #[test]
    fn test_name_count_units() {
        let x = State::new(
            r#"create                    726 samples [reqs]
"#,
        );

        let result = name_count_units().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                ("create".to_string(), 726, "reqs".to_string()),
                State {
                    input: "\n",
                    positioner: SourcePosition {
                        line: 1,
                        column: 45
                    }
                }
            ))
        );
    }

    #[test]
    fn test_stat_no_sumsquare() {
        let x = State::new(
            r#"cache_miss                21108 samples [pages] 1 1 21108
"#,
        );

        let result = stat().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                Stat {
                    name: "cache_miss".to_string(),
                    samples: 21108,
                    units: "pages".to_string(),
                    min: Some(1),
                    max: Some(1),
                    sum: Some(21108),
                    sumsquare: None
                },
                State {
                    input: "",
                    positioner: SourcePosition { line: 2, column: 1 }
                }
            ))
        );
    }

    #[test]
    fn test_stat() {
        let x = State::new(
            r#"obd_ping                  1108 samples [usec] 15 72 47014 2156132
"#,
        );

        let result = stat().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                Stat {
                    name: "obd_ping".to_string(),
                    units: "usec".to_string(),
                    samples: 1108,
                    min: Some(15),
                    max: Some(72),
                    sum: Some(47014),
                    sumsquare: Some(2_156_132)
                },
                State {
                    input: "",
                    positioner: SourcePosition { line: 2, column: 1 }
                }
            ))
        );
    }

    #[test]
    fn test_stats() {
        let x = State::new(
            r#"
snapshot_time             1534770326.579119384 secs.nsecs
write_bytes               9 samples [bytes] 98303 4194304 33554431
create                    4 samples [reqs]
statfs                    5634 samples [reqs]
get_info                  2 samples [reqs]
connect                   4 samples [reqs]
reconnect                 1 samples [reqs]
disconnect                3 samples [reqs]
statfs                    18 samples [reqs]
preprw                    9 samples [reqs]
commitrw                  9 samples [reqs]
ping                      1075 samples [reqs]
"#,
        );

        let result = stats().easy_parse(x).unwrap();

        assert_debug_snapshot_matches!(result);
    }

    #[test]
    fn test_mdstats() {
        let x = State::new(
            r#"
snapshot_time             1566007540.707634939 secs.nsecs
statfs                    16360 samples [reqs]
"#,
        );

        let result = stats().easy_parse(x).unwrap();

        assert_debug_snapshot_matches!(result);
    }
}
