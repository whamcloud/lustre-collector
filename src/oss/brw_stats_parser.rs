// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use base_parsers::{digits, string_to, till_newline, word};
use combine::error::ParseError;
use combine::parser::char::{newline, spaces, string};
use combine::stream::Stream;
use combine::{choice, many, many1, one_of, optional, token, try, Parser};
use snapshot_time::snapshot_time;
use stats::{BrwStats, BrwStatsBucket};

fn human_to_bytes((x, y): (u64, Option<char>)) -> u64 {
    let mult = match y {
        None => 1,
        Some('K') | Some('k') => 2_u64.pow(10),
        Some('M') | Some('m') => 2_u64.pow(20),
        Some('G') | Some('g') => 2_u64.pow(30),
        Some(x) => panic!(format!("Conversion to : {} not covered", x)),
    };

    x * mult
}

fn rw_columns<I>() -> impl Parser<Input = I, Output = ()>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
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

fn bucket<I>() -> impl Parser<Input = I, Output = BrwStatsBucket>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        digits()
            .and(optional(one_of("KkMmGg".chars())))
            .map(human_to_bytes),
        token(':'),
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().skip(token('|')),
        spaces().with(digits()),
        spaces().with(digits()),
        spaces().with(digits()),
        till_newline(),
    ).map(|(name, _, read, _, _, _, write, _, _, _)| BrwStatsBucket { name, read, write })
}

fn section<I>() -> impl Parser<Input = I, Output = BrwStats>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        rw_columns().skip(newline()),
        header().skip(newline()),
        many(bucket().skip(newline())).skip(spaces()),
    ).map(|(_, stats, xs)| BrwStats {
        buckets: xs,
        ..stats
    })
}

pub fn brw_stats<I>() -> impl Parser<Input = I, Output = Vec<BrwStats>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (newline().with(snapshot_time()), spaces(), many1(section())).map(|(_, _, y)| y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::state::{SourcePosition, State};

    #[test]
    fn test_human_to_bytes() {
        assert_eq!(human_to_bytes((1, Some('k'))), 1024);
        assert_eq!(human_to_bytes((2, Some('K'))), 2048);
        assert_eq!(human_to_bytes((1, Some('m'))), 1048576);
        assert_eq!(human_to_bytes((2, Some('M'))), 2097152);
        assert_eq!(human_to_bytes((1, Some('g'))), 1073741824);
        assert_eq!(human_to_bytes((5, Some('G'))), 5368709120);
        assert_eq!(human_to_bytes((5, None)), 5);
    }

    #[test]
    fn test_rw_columns() {
        let x = State::new("read      |     write\n");

        let result = rw_columns().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                (),
                State {
                    input: "\n",
                    positioner: SourcePosition {
                        line: 1,
                        column: 22
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
                BrwStatsBucket {
                    name: 32,
                    read: 0,
                    write: 1,
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
            r#"read      |     write
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
                        BrwStatsBucket {
                            name: 32,
                            read: 0,
                            write: 1,
                        },
                        BrwStatsBucket {
                            name: 64,
                            read: 0,
                            write: 0,
                        },
                        BrwStatsBucket {
                            name: 128,
                            read: 0,
                            write: 0,
                        },
                        BrwStatsBucket {
                            name: 256,
                            read: 0,
                            write: 0,
                        },
                        BrwStatsBucket {
                            name: 512,
                            read: 0,
                            write: 0,
                        },
                        BrwStatsBucket {
                            name: 1024,
                            read: 0,
                            write: 8,
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
            r#"read      |     write
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
                ],
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
                vec![
                    BrwStats {
                        name: "pages".to_string(),
                        unit: "rpcs".to_string(),
                        buckets: vec![
                            BrwStatsBucket {
                                name: 32,
                                read: 0,
                                write: 1,
                            },
                            BrwStatsBucket {
                                name: 64,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 128,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 256,
                                read: 1,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 512,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 1024,
                                read: 0,
                                write: 8,
                            },
                        ],
                    },
                    BrwStats {
                        name: "discont_pages".to_string(),
                        unit: "rpcs".to_string(),
                        buckets: vec![
                            BrwStatsBucket {
                                name: 0,
                                read: 0,
                                write: 6,
                            },
                            BrwStatsBucket {
                                name: 1,
                                read: 0,
                                write: 3,
                            },
                        ],
                    },
                    BrwStats {
                        name: "discont_blocks".to_string(),
                        unit: "rpcs".to_string(),
                        buckets: vec![BrwStatsBucket {
                            name: 0,
                            read: 0,
                            write: 9,
                        }],
                    },
                    BrwStats {
                        name: "dio_frags".to_string(),
                        unit: "ios".to_string(),
                        buckets: vec![
                            BrwStatsBucket {
                                name: 1,
                                read: 0,
                                write: 1,
                            },
                            BrwStatsBucket {
                                name: 2,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 3,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 4,
                                read: 0,
                                write: 8,
                            },
                        ],
                    },
                    BrwStats {
                        name: "rpc_hist".to_string(),
                        unit: "ios".to_string(),
                        buckets: vec![
                            BrwStatsBucket {
                                name: 1,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 2,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 3,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 4,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 5,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 6,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 7,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 8,
                                read: 0,
                                write: 3,
                            },
                            BrwStatsBucket {
                                name: 9,
                                read: 0,
                                write: 2,
                            },
                            BrwStatsBucket {
                                name: 10,
                                read: 0,
                                write: 2,
                            },
                            BrwStatsBucket {
                                name: 11,
                                read: 0,
                                write: 2,
                            },
                            BrwStatsBucket {
                                name: 12,
                                read: 0,
                                write: 2,
                            },
                            BrwStatsBucket {
                                name: 13,
                                read: 0,
                                write: 1,
                            },
                        ],
                    },
                    BrwStats {
                        name: "io_time".to_string(),
                        unit: "ios".to_string(),
                        buckets: vec![
                            BrwStatsBucket {
                                name: 32,
                                read: 0,
                                write: 1,
                            },
                            BrwStatsBucket {
                                name: 64,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 128,
                                read: 0,
                                write: 2,
                            },
                            BrwStatsBucket {
                                name: 256,
                                read: 0,
                                write: 6,
                            },
                        ],
                    },
                    BrwStats {
                        name: "disk_iosize".to_string(),
                        unit: "ios".to_string(),
                        buckets: vec![
                            BrwStatsBucket {
                                name: 131072,
                                read: 0,
                                write: 1,
                            },
                            BrwStatsBucket {
                                name: 262144,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 524288,
                                read: 0,
                                write: 0,
                            },
                            BrwStatsBucket {
                                name: 1048576,
                                read: 0,
                                write: 32,
                            },
                        ],
                    },
                ],
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
