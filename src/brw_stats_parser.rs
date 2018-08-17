use combine::error::ParseError;
use combine::parser::char::{alpha_num, digit, newline, spaces, string};
use combine::parser::repeat::take_until;
use combine::stream::Stream;
use combine::{choice, many, many1, token, try, Parser};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct BrwStatsBucketVals {
    count: String,
    pct: String,
    cum_pct: String,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct BrwStatsBuckets {
    pub name: String,
    pub read: BrwStatsBucketVals,
    pub write: BrwStatsBucketVals,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct BrwStats {
    pub name: String,
    pub unit: String,
    pub buckets: Vec<BrwStatsBuckets>,
}

fn string_to<I>(x: &'static str, y: &'static str) -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    string(x).map(move |_| String::from(y))
}

fn word<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(alpha_num().or(token('_')))
}

fn digits<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1(digit())
}

fn till_newline<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    take_until(newline())
}

fn snapshot_time<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("snapshot_time:").skip(spaces()),
        digits().skip(token('.')),
        digits().skip(till_newline()),
    ).map(|(_, secs, nsecs)| format!("{}.{}", secs, nsecs))
}

fn rw_columns<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        spaces(),
        string("read"),
        spaces(),
        token('|'),
        spaces(),
        string("write"),
        till_newline(),
    ).map(|_| ())
}

fn header<I>() -> impl Parser<Input = I, Output = BrwStats>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let keys = choice([
        try(string_to("pages per bulk r/w", "pages")),
        try(string_to("discontiguous pages", "discont_pages")),
        try(string_to("discontiguous blocks", "discont_blocks")),
        try(string_to("disk fragmented I/Os", "dio_frags")),
        try(string_to("disk I/Os in flight", "rpc_hist")),
        try(string_to("I/O time (1/1000s)", "io_time")),
        try(string_to("disk I/O size", "disk_iosize")),
    ]);

    (keys.skip(spaces()), word().skip(till_newline())).map(|(name, unit)| BrwStats {
        name,
        unit,
        buckets: vec![],
    })
}

fn bucket<I>() -> impl Parser<Input = I, Output = BrwStatsBuckets>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        word().skip(token(':')),
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().skip(token('|')),
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().with(digits()),
        till_newline(),
    ).map(
        |(
            name,
            read_count,
            read_pct,
            read_cum_pct,
            _,
            write_count,
            write_pct,
            write_cum_pct,
            _,
        )| {
            BrwStatsBuckets {
                name,
                read: BrwStatsBucketVals {
                    count: read_count,
                    pct: read_pct,
                    cum_pct: read_cum_pct,
                },
                write: BrwStatsBucketVals {
                    count: write_count,
                    pct: write_pct,
                    cum_pct: write_cum_pct,
                },
            }
        },
    )
}

fn section<I>() -> impl Parser<Input = I, Output = BrwStats>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    ((
        rw_columns().skip(newline()),
        header().skip(newline()),
        many(bucket().skip(newline())),
    )).map(|(_, stats, xs)| BrwStats {
        buckets: xs,
        ..stats
    })
}

pub fn brw_stats<I>() -> impl Parser<Input = I, Output = (String, Vec<BrwStats>)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (newline().with(snapshot_time()), spaces(), many1(section())).map(|(x, _, y)| (x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::state::{SourcePosition, State};

    #[test]
    fn test_snapshot_time() {
        let x = State::new(
            r#"snapshot_time:         1534158712.738772898 (secs.nsecs)
"#,
        );

        let result = snapshot_time().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                "1534158712.738772898".to_string(),
                State {
                    input: "\n",
                    positioner: SourcePosition {
                        line: 1,
                        column: 57
                    }
                }
            ))
        );
    }

    #[test]
    fn test_rw_columns() {
        let x = State::new(
            r#"                           read      |     write
"#,
        );

        let result = rw_columns().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                (),
                State {
                    input: "\n",
                    positioner: SourcePosition {
                        line: 1,
                        column: 49
                    }
                }
            ))
        );
    }

    #[test]
    fn test_header() {
        let x = State::new(
            r#"pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
"#,
        );

        let result = header().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStats {
                    name: "pages".to_string(),
                    unit: "rpcs".to_string(),
                    buckets: vec![],
                },
                State {
                    input: "\n",
                    positioner: SourcePosition {
                        line: 1,
                        column: 60,
                    },
                }
            ))
        );
    }

    #[test]
    fn test_bucket() {
        let x = State::new(
            r#"32:		         0   0   0   |    1  11  11
"#,
        );

        let result = bucket().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStatsBuckets {
                    name: "32".to_string(),
                    read: BrwStatsBucketVals {
                        count: "0".to_string(),
                        pct: "0".to_string(),
                        cum_pct: "0".to_string(),
                    },
                    write: BrwStatsBucketVals {
                        count: "1".to_string(),
                        pct: "11".to_string(),
                        cum_pct: "11".to_string(),
                    },
                },
                State {
                    input: "\n",
                    positioner: SourcePosition {
                        line: 1,
                        column: 41
                    }
                }
            ))
        );
    }

    #[test]
    fn test_section() {
        let x = State::new(
            r#"                           read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
32:		         0   0   0   |    1  11  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    0   0  11
256:		         0   0   0   |    0   0  11
512:		         0   0   0   |    0   0  11
1K:		         0   0   0   |    8  88 100
"#,
        );

        let result = section().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStats {
                    name: "pages".to_string(),
                    unit: "rpcs".to_string(),
                    buckets: vec![
                        BrwStatsBuckets {
                            name: "32".to_string(),
                            read: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "0".to_string(),
                            },
                            write: BrwStatsBucketVals {
                                count: "1".to_string(),
                                pct: "11".to_string(),
                                cum_pct: "11".to_string(),
                            },
                        },
                        BrwStatsBuckets {
                            name: "64".to_string(),
                            read: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "0".to_string(),
                            },
                            write: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "11".to_string(),
                            },
                        },
                        BrwStatsBuckets {
                            name: "128".to_string(),
                            read: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "0".to_string(),
                            },
                            write: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "11".to_string(),
                            },
                        },
                        BrwStatsBuckets {
                            name: "256".to_string(),
                            read: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "0".to_string(),
                            },
                            write: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "11".to_string(),
                            },
                        },
                        BrwStatsBuckets {
                            name: "512".to_string(),
                            read: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "0".to_string(),
                            },
                            write: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "11".to_string(),
                            },
                        },
                        BrwStatsBuckets {
                            name: "1K".to_string(),
                            read: BrwStatsBucketVals {
                                count: "0".to_string(),
                                pct: "0".to_string(),
                                cum_pct: "0".to_string(),
                            },
                            write: BrwStatsBucketVals {
                                count: "8".to_string(),
                                pct: "88".to_string(),
                                cum_pct: "100".to_string(),
                            },
                        },
                    ],
                },
                State {
                    input: "",
                    positioner: SourcePosition { line: 9, column: 1 },
                }
            ))
        );
    }

    #[test]
    fn test_empty_section() {
        let x = State::new(
            r#"                           read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
"#,
        );

        let result = section().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStats {
                    name: "pages".to_string(),
                    unit: "rpcs".to_string(),
                    buckets: vec![],
                },
                State {
                    input: "",
                    positioner: SourcePosition { line: 3, column: 1 },
                }
            ))
        );
    }

    #[test]
    fn test_empty_brw_stats() {
        let x = State::new(
            r#"
snapshot_time:         1534429278.185762481 (secs.nsecs)

                           read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %

                           read      |     write
discontiguous pages    rpcs  % cum % |  rpcs        % cum %

                           read      |     write
discontiguous blocks   rpcs  % cum % |  rpcs        % cum %

                           read      |     write
disk fragmented I/Os   ios   % cum % |  ios         % cum %

                           read      |     write
disk I/Os in flight    ios   % cum % |  ios         % cum %

                           read      |     write
I/O time (1/1000s)     ios   % cum % |  ios         % cum %

                           read      |     write
disk I/O size          ios   % cum % |  ios         % cum %
"#,
        );

        let result = brw_stats().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                (
                    "1534429278.185762481".to_string(),
                    vec![
                        BrwStats {
                            name: "pages".to_string(),
                            unit: "rpcs".to_string(),
                            buckets: vec![],
                        },
                        BrwStats {
                            name: "discont_pages".to_string(),
                            unit: "rpcs".to_string(),
                            buckets: vec![],
                        },
                        BrwStats {
                            name: "discont_blocks".to_string(),
                            unit: "rpcs".to_string(),
                            buckets: vec![],
                        },
                        BrwStats {
                            name: "dio_frags".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![],
                        },
                        BrwStats {
                            name: "rpc_hist".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![],
                        },
                        BrwStats {
                            name: "io_time".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![],
                        },
                        BrwStats {
                            name: "disk_iosize".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![],
                        },
                    ]
                ),
                State {
                    input: "",
                    positioner: SourcePosition {
                        line: 24,
                        column: 1,
                    },
                }
            ))
        );
    }

    #[test]
    fn test_brw_stats() {
        let x = State::new(
            r#"
snapshot_time:         1534158712.738772898 (secs.nsecs)

                           read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
32:		         0   0   0   |    1  11  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    0   0  11
256:		         1   2   3   |    0   0  11
512:		         0   0   0   |    0   0  11
1K:		         0   0   0   |    8  88 100

                           read      |     write
discontiguous pages    rpcs  % cum % |  rpcs        % cum %
0:		         0   0   0   |    6  66  66
1:		         0   0   0   |    3  33 100

                           read      |     write
discontiguous blocks   rpcs  % cum % |  rpcs        % cum %
0:		         0   0   0   |    9 100 100

                           read      |     write
disk fragmented I/Os   ios   % cum % |  ios         % cum %
1:		         0   0   0   |    1  11  11
2:		         0   0   0   |    0   0  11
3:		         0   0   0   |    0   0  11
4:		         0   0   0   |    8  88 100

                           read      |     write
disk I/Os in flight    ios   % cum % |  ios         % cum %
1:		         0   0   0   |    3   9   9
2:		         0   0   0   |    3   9  18
3:		         0   0   0   |    3   9  27
4:		         0   0   0   |    3   9  36
5:		         0   0   0   |    3   9  45
6:		         0   0   0   |    3   9  54
7:		         0   0   0   |    3   9  63
8:		         0   0   0   |    3   9  72
9:		         0   0   0   |    2   6  78
10:		         0   0   0   |    2   6  84
11:		         0   0   0   |    2   6  90
12:		         0   0   0   |    2   6  96
13:		         0   0   0   |    1   3 100

                           read      |     write
I/O time (1/1000s)     ios   % cum % |  ios         % cum %
32:		         0   0   0   |    1  11  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    2  22  33
256:		         0   0   0   |    6  66 100

                           read      |     write
disk I/O size          ios   % cum % |  ios         % cum %
128K:		         0   0   0   |    1   3   3
256K:		         0   0   0   |    0   0   3
512K:		         0   0   0   |    0   0   3
1M:		         0   0   0   |   32  96 100
"#,
        );

        let result = brw_stats().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                (
                    "1534158712.738772898".to_string(),
                    vec![
                        BrwStats {
                            name: "pages".to_string(),
                            unit: "rpcs".to_string(),
                            buckets: vec![
                                BrwStatsBuckets {
                                    name: "32".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "1".to_string(),
                                        pct: "11".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "64".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "128".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "256".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "1".to_string(),
                                        pct: "2".to_string(),
                                        cum_pct: "3".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "512".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "1K".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "8".to_string(),
                                        pct: "88".to_string(),
                                        cum_pct: "100".to_string(),
                                    },
                                },
                            ],
                        },
                        BrwStats {
                            name: "discont_pages".to_string(),
                            unit: "rpcs".to_string(),
                            buckets: vec![
                                BrwStatsBuckets {
                                    name: "0".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "6".to_string(),
                                        pct: "66".to_string(),
                                        cum_pct: "66".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "1".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "33".to_string(),
                                        cum_pct: "100".to_string(),
                                    },
                                },
                            ],
                        },
                        BrwStats {
                            name: "discont_blocks".to_string(),
                            unit: "rpcs".to_string(),
                            buckets: vec![BrwStatsBuckets {
                                name: "0".to_string(),
                                read: BrwStatsBucketVals {
                                    count: "0".to_string(),
                                    pct: "0".to_string(),
                                    cum_pct: "0".to_string(),
                                },
                                write: BrwStatsBucketVals {
                                    count: "9".to_string(),
                                    pct: "100".to_string(),
                                    cum_pct: "100".to_string(),
                                },
                            }],
                        },
                        BrwStats {
                            name: "dio_frags".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![
                                BrwStatsBuckets {
                                    name: "1".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "1".to_string(),
                                        pct: "11".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "2".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "3".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "4".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "8".to_string(),
                                        pct: "88".to_string(),
                                        cum_pct: "100".to_string(),
                                    },
                                },
                            ],
                        },
                        BrwStats {
                            name: "rpc_hist".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![
                                BrwStatsBuckets {
                                    name: "1".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "9".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "2".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "18".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "3".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "27".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "4".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "36".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "5".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "45".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "6".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "54".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "7".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "63".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "8".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "3".to_string(),
                                        pct: "9".to_string(),
                                        cum_pct: "72".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "9".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "2".to_string(),
                                        pct: "6".to_string(),
                                        cum_pct: "78".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "10".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "2".to_string(),
                                        pct: "6".to_string(),
                                        cum_pct: "84".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "11".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "2".to_string(),
                                        pct: "6".to_string(),
                                        cum_pct: "90".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "12".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "2".to_string(),
                                        pct: "6".to_string(),
                                        cum_pct: "96".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "13".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "1".to_string(),
                                        pct: "3".to_string(),
                                        cum_pct: "100".to_string(),
                                    },
                                },
                            ],
                        },
                        BrwStats {
                            name: "io_time".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![
                                BrwStatsBuckets {
                                    name: "32".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "1".to_string(),
                                        pct: "11".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "64".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "11".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "128".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "2".to_string(),
                                        pct: "22".to_string(),
                                        cum_pct: "33".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "256".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "6".to_string(),
                                        pct: "66".to_string(),
                                        cum_pct: "100".to_string(),
                                    },
                                },
                            ],
                        },
                        BrwStats {
                            name: "disk_iosize".to_string(),
                            unit: "ios".to_string(),
                            buckets: vec![
                                BrwStatsBuckets {
                                    name: "128K".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "1".to_string(),
                                        pct: "3".to_string(),
                                        cum_pct: "3".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "256K".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "3".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "512K".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "3".to_string(),
                                    },
                                },
                                BrwStatsBuckets {
                                    name: "1M".to_string(),
                                    read: BrwStatsBucketVals {
                                        count: "0".to_string(),
                                        pct: "0".to_string(),
                                        cum_pct: "0".to_string(),
                                    },
                                    write: BrwStatsBucketVals {
                                        count: "32".to_string(),
                                        pct: "96".to_string(),
                                        cum_pct: "100".to_string(),
                                    },
                                },
                            ],
                        },
                    ]
                ),
                State {
                    input: "",
                    positioner: SourcePosition {
                        line: 58,
                        column: 1,
                    },
                }
            ))
        );
    }
}
