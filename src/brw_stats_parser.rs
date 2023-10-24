// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, string_to, till_newline, word},
    time::time_triple,
    types::{BrwStats, BrwStatsBucket},
};
use combine::{
    attempt, choice,
    error::ParseError,
    many, many1, one_of, optional,
    parser::char::{newline, spaces, string},
    stream::Stream,
    token, Parser,
};

fn human_to_bytes((x, y): (u64, Option<char>)) -> u64 {
    let mult = match y {
        None => 1,
        Some('K') | Some('k') => 2_u64.pow(10),
        Some('M') | Some('m') => 2_u64.pow(20),
        Some('G') | Some('g') => 2_u64.pow(30),
        Some(x) => panic!("Conversion to : {} not covered", x),
    };

    x * mult
}

fn rw_columns<I>() -> impl Parser<I, Output = ()>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        string("read"),
        spaces(),
        token('|'),
        spaces(),
        string("write"),
        till_newline(),
    )
        .map(|_| ())
}

fn header<I>() -> impl Parser<I, Output = BrwStats>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let keys = choice([
        attempt(string_to("pages per bulk r/w", "pages")),
        attempt(string_to("discontiguous pages", "discont_pages")),
        attempt(string_to("discontiguous blocks", "discont_blocks")),
        attempt(string_to("disk fragmented I/Os", "dio_frags")),
        attempt(string_to("disk I/Os in flight", "rpc_hist")),
        attempt(string_to("I/O time (1/1000s)", "io_time")),
        attempt(string_to("disk I/O size", "disk_iosize")),
        attempt(string_to("block maps msec", "block_maps_msec")),
    ]);

    (keys.skip(spaces()), word().skip(till_newline())).map(|(name, unit)| BrwStats {
        name,
        unit,
        buckets: vec![],
    })
}

fn bucket<I>() -> impl Parser<I, Output = BrwStatsBucket>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
    )
        .map(|(name, _, read, _, _, _, write, _, _, _)| BrwStatsBucket { name, read, write })
}

fn section<I>() -> impl Parser<I, Output = BrwStats>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        rw_columns().skip(newline()),
        header().skip(newline()),
        many(bucket().skip(newline())).skip(spaces()),
    )
        .map(|(_, stats, xs)| BrwStats {
            buckets: xs,
            ..stats
        })
}

pub(crate) fn brw_stats<I>() -> impl Parser<I, Output = Vec<BrwStats>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (newline().with(time_triple()), spaces(), many1(section())).map(|(_, _, y)| y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_human_to_bytes() {
        assert_eq!(human_to_bytes((1, Some('k'))), 1024);
        assert_eq!(human_to_bytes((2, Some('K'))), 2048);
        assert_eq!(human_to_bytes((1, Some('m'))), 1_048_576);
        assert_eq!(human_to_bytes((2, Some('M'))), 2_097_152);
        assert_eq!(human_to_bytes((1, Some('g'))), 1_073_741_824);
        assert_eq!(human_to_bytes((5, Some('G'))), 5_368_709_120);
        assert_eq!(human_to_bytes((5, None)), 5);
    }

    #[test]
    fn test_rw_columns() {
        let x = "read      |     write\n";

        let result = rw_columns().parse(x);

        assert_eq!(result, Ok(((), "\n")));
    }

    #[test]
    fn test_header() {
        let x = r#"pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
"#;

        let result = header().parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStats {
                    name: "pages".to_string(),
                    unit: "rpcs".to_string(),
                    buckets: vec![],
                },
                "\n"
            ))
        );
    }

    #[test]
    fn test_bucket() {
        let x = r#"32:		         0   0   0   |    1  11  11
"#;

        let result = bucket().parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStatsBucket {
                    name: 32,
                    read: 0,
                    write: 1,
                },
                "\n",
            ))
        );
    }

    #[test]
    fn test_section() {
        let x = r#"read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
32:		         0   0   0   |    1  11  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    0   0  11
256:		         0   0   0   |    0   0  11
512:		         0   0   0   |    0   0  11
1K:		         0   0   0   |    8  88 100
"#;

        let result = section().parse(x);

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
                ""
            ))
        );
    }

    #[test]
    fn test_empty_section() {
        let x = r#"read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
"#;

        let result = section().parse(x);

        assert_eq!(
            result,
            Ok((
                BrwStats {
                    name: "pages".to_string(),
                    unit: "rpcs".to_string(),
                    buckets: vec![],
                },
                "",
            ))
        );
    }

    #[test]
    fn test_empty_brw_stats() {
        let x = r#"
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
"#;

        let result = brw_stats().parse(x);

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
                ""
            ))
        );
    }

    #[test]
    fn test_brw_stats() {
        let x = r#"
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
"#;

        let result: (Vec<_>, _) = brw_stats().parse(x).unwrap();

        assert_debug_snapshot!(result);
    }

    #[test]
    fn test_brw_stats_with_start_and_elapsed_time() {
        let x = include_str!("../fixtures/brw_stats.txt");

        let result = brw_stats().parse(x).unwrap();

        assert_debug_snapshot!(result);
    }
}
