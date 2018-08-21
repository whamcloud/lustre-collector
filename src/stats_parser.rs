// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use base_parsers::{digits, not_word, word};
use combine::error::ParseError;
use combine::parser::char::{newline, spaces, string};
use combine::parser::choice::or;
use combine::stream::Stream;
use combine::{between, many1, token, Parser};
use snapshot_time::snapshot_time;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub samples: String,
    pub units: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub sum: Option<String>,
    pub sumsquare: Option<String>,
}

fn name_count_units<I>() -> impl Parser<Input = I, Output = (String, String, String)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        not_word("obdfilter").skip(spaces()),
        digits(),
        spaces().with(string("samples")),
        spaces().with(between(token('['), token(']'), word())),
    ).map(|(x, y, _, z)| (x, y, z))
}

fn min_max_sum<I>() -> impl Parser<Input = I, Output = (String, String, String)>
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

fn sum_sq<I>() -> impl Parser<Input = I, Output = String>
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
                min_max_sum().map(|x| Some(x)),
                or(
                    newline().map(|_| None),
                    sum_sq().map(|x| Some(x)).skip(newline()),
                ),
            ),
        ),
    ).map(
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
                ("create".to_string(), "726".to_string(), "reqs".to_string()),
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
                    samples: "21108".to_string(),
                    units: "pages".to_string(),
                    min: Some("1".to_string()),
                    max: Some("1".to_string()),
                    sum: Some("21108".to_string()),
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
                    samples: "1108".to_string(),
                    units: "usec".to_string(),
                    min: Some("15".to_string()),
                    max: Some("72".to_string()),
                    sum: Some("47014".to_string()),
                    sumsquare: Some("2156132".to_string())
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

        let result = stats().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                vec![
                    Stat {
                        name: "write_bytes".to_string(),
                        samples: "9".to_string(),
                        units: "bytes".to_string(),
                        min: Some("98303".to_string()),
                        max: Some("4194304".to_string()),
                        sum: Some("33554431".to_string()),
                        sumsquare: None,
                    },
                    Stat {
                        name: "create".to_string(),
                        samples: "4".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "statfs".to_string(),
                        samples: "5634".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "get_info".to_string(),
                        samples: "2".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "connect".to_string(),
                        samples: "4".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "reconnect".to_string(),
                        samples: "1".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "disconnect".to_string(),
                        samples: "3".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "statfs".to_string(),
                        samples: "18".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "preprw".to_string(),
                        samples: "9".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "commitrw".to_string(),
                        samples: "9".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                    Stat {
                        name: "ping".to_string(),
                        samples: "1075".to_string(),
                        units: "reqs".to_string(),
                        min: None,
                        max: None,
                        sum: None,
                        sumsquare: None,
                    },
                ],
                State {
                    input: "",
                    positioner: SourcePosition {
                        line: 14,
                        column: 1,
                    },
                }
            ))
        );
    }
}
