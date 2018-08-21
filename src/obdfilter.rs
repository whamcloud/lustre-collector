use base_parsers::{period, word};
use brw_stats_parser::{brw_stats, BrwStats};
use combine::error::ParseError;
use combine::parser::char::{alpha_num, newline, string};
use combine::parser::choice::choice;
use combine::parser::repeat::take_until;
use combine::stream::Stream;
use combine::{many1, one_of, token, try, Parser};
use stats_parser::{stats, Stat};

#[derive(Debug, PartialEq, Serialize)]
pub struct TargetName(String);

#[derive(Debug, PartialEq, Serialize)]
pub struct ParamName(String);

#[derive(PartialEq, Debug, Serialize)]
#[serde(untagged)]
pub enum ObdfilterStat {
    Stats(Vec<Stat>),
    BrwStats(Vec<BrwStats>),
    Value(String),
}

fn target_name<I>() -> impl Parser<Input = I, Output = TargetName>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("obdfilter").skip(period()),
        many1(alpha_num().or(one_of("_-".chars()))).skip(period()),
    ).map(|(_, target)| TargetName(target))
        .message("while parsing target_name")
}

fn simple_value<I>() -> impl Parser<Input = I, Output = String>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    take_until(newline()).skip(token('\n'))
}

fn prop_and_stat<I>() -> impl Parser<Input = I, Output = (ParamName, ObdfilterStat)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        (try(string("brw_stats")).skip(token('=')), brw_stats())
            .map(|(p, (_, x))| (ParamName(p.to_string()), ObdfilterStat::BrwStats(x))),
        (try(string("stats")).skip(token('=')), stats())
            .map(|(p, x)| (ParamName(p.to_string()), ObdfilterStat::Stats(x))),
        (word().skip(token('=')), simple_value())
            .map(|(p, x)| (ParamName(p.to_string()), ObdfilterStat::Value(x))),
    ))
}

pub fn obdfilter_stats<I>(
) -> impl Parser<Input = I, Output = Vec<(TargetName, ParamName, ObdfilterStat)>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1((target_name(), prop_and_stat()).map(|(t, (p, s))| (t, p, s)))
}

#[cfg(test)]
mod tests {
    use super::ObdfilterStat::{Stats, Value};
    use super::*;
    use brw_stats_parser::{BrwStats, BrwStatsBucketVals, BrwStatsBuckets};
    use combine::error::StreamError;
    use combine::stream::state::State;
    use combine::{easy, error};

    #[test]
    fn test_target_name() {
        let result = target_name().easy_parse("obdfilter.fs-OST0000.num_exports=");

        assert_eq!(
            result,
            Ok((TargetName("fs-OST0000".to_string()), "num_exports="))
        );
    }

    #[test]
    fn test_obdfilter_stats() {
        let result = obdfilter_stats().easy_parse(
            r#"obdfilter.fs-OST0000.stats=
snapshot_time             1534862608.096725484 secs.nsecs
write_bytes               9 samples [bytes] 98303 4194304 33554431
create                    4 samples [reqs]
statfs                    12777 samples [reqs]
get_info                  2 samples [reqs]
connect                   6 samples [reqs]
reconnect                 1 samples [reqs]
disconnect                4 samples [reqs]
statfs                    156 samples [reqs]
preprw                    9 samples [reqs]
commitrw                  9 samples [reqs]
ping                      2309 samples [reqs]
obdfilter.fs-OST0000.blocksize=4096
obdfilter.fs-OST0000.brw_size=4
obdfilter.fs-OST0000.brw_stats=
snapshot_time:         1534862608.097085730 (secs.nsecs)

                           read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
32:		         0   0   0   |    1  11  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    0   0  11
256:		         0   0   0   |    0   0  11
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
1:		         0   0   0   |    2   6   6
2:		         0   0   0   |    1   3   9
3:		         0   0   0   |    1   3  12
4:		         0   0   0   |    1   3  15
5:		         0   0   0   |    3   9  24
6:		         0   0   0   |    3   9  33
7:		         0   0   0   |    3   9  42
8:		         0   0   0   |    3   9  51
9:		         0   0   0   |    1   3  54
10:		         0   0   0   |    1   3  57
11:		         0   0   0   |    1   3  60
12:		         0   0   0   |    1   3  63
13:		         0   0   0   |    2   6  69
14:		         0   0   0   |    2   6  75
15:		         0   0   0   |    2   6  81
16:		         0   0   0   |    2   6  87
17:		         0   0   0   |    1   3  90
18:		         0   0   0   |    1   3  93
19:		         0   0   0   |    1   3  96
20:		         0   0   0   |    1   3 100

                           read      |     write
I/O time (1/1000s)     ios   % cum % |  ios         % cum %
4:		         0   0   0   |    1  11  11
8:		         0   0   0   |    0   0  11
16:		         0   0   0   |    0   0  11
32:		         0   0   0   |    0   0  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    3  33  44
256:		         0   0   0   |    5  55 100

                           read      |     write
disk I/O size          ios   % cum % |  ios         % cum %
128K:		         0   0   0   |    1   3   3
256K:		         0   0   0   |    0   0   3
512K:		         0   0   0   |    0   0   3
1M:		         0   0   0   |   32  96 100
obdfilter.fs-OST0000.checksum_dump=0
obdfilter.fs-OST0000.client_cache_count=128
obdfilter.fs-OST0000.client_cache_seconds=110
obdfilter.fs-OST0000.filesfree=327382
obdfilter.fs-OST0000.filestotal=327680
obdfilter.fs-OST0000.fstype=osd-ldiskfs
obdfilter.fs-OST0000.grant_compat_disable=0
obdfilter.fs-OST0000.grant_precreate=278208
obdfilter.fs-OST0000.instance=1
obdfilter.fs-OST0000.ir_factor=5
obdfilter.fs-OST0000.job_cleanup_interval=600
obdfilter.fs-OST0000.kbytesavail=4486468
obdfilter.fs-OST0000.kbytesfree=4764996
obdfilter.fs-OST0000.kbytestotal=4831716
obdfilter.fs-OST0000.last_id=0x100000000:65
obdfilter.fs-OST0000.lfsck_speed_limit=0
obdfilter.fs-OST0000.num_exports=2
obdfilter.fs-OST0000.precreate_batch=128
obdfilter.fs-OST0000.read_cache_enable=1
obdfilter.fs-OST0000.readcache_max_filesize=18446744073709551615
obdfilter.fs-OST0000.recovery_status=status: INACTIVE
obdfilter.fs-OST0000.recovery_time_hard=900
obdfilter.fs-OST0000.recovery_time_soft=150
obdfilter.fs-OST0000.seqs_allocated=1
obdfilter.fs-OST0000.site_stats=20/89 89/262144 1 114 32 99 0 0 0
obdfilter.fs-OST0000.soft_sync_limit=16
obdfilter.fs-OST0000.sync_journal=0
obdfilter.fs-OST0000.sync_on_lock_cancel=never
obdfilter.fs-OST0000.tot_dirty=0
obdfilter.fs-OST0000.tot_granted=8666816
obdfilter.fs-OST0000.tot_pending=0
obdfilter.fs-OST0000.uuid=fs-OST0000_UUID
obdfilter.fs-OST0000.writethrough_cache_enable=1
"#,
        );

        assert_eq!(
            result,
            Ok((
                vec![
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("stats".to_string()),
                        Stats(vec![
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
                                samples: "12777".to_string(),
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
                                samples: "6".to_string(),
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
                                samples: "4".to_string(),
                                units: "reqs".to_string(),
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "statfs".to_string(),
                                samples: "156".to_string(),
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
                                samples: "2309".to_string(),
                                units: "reqs".to_string(),
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                        ]),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("blocksize".to_string()),
                        Value("4096".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("brw_size".to_string()),
                        Value("4".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("brw_stats".to_string()),
                        super::ObdfilterStat::BrwStats(vec![
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
                                            count: "2".to_string(),
                                            pct: "6".to_string(),
                                            cum_pct: "6".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "9".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "12".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "15".to_string(),
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
                                            cum_pct: "24".to_string(),
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
                                            cum_pct: "33".to_string(),
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
                                            cum_pct: "42".to_string(),
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
                                            cum_pct: "51".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "54".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "57".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "60".to_string(),
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
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "63".to_string(),
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
                                            count: "2".to_string(),
                                            pct: "6".to_string(),
                                            cum_pct: "69".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "14".to_string(),
                                        read: BrwStatsBucketVals {
                                            count: "0".to_string(),
                                            pct: "0".to_string(),
                                            cum_pct: "0".to_string(),
                                        },
                                        write: BrwStatsBucketVals {
                                            count: "2".to_string(),
                                            pct: "6".to_string(),
                                            cum_pct: "75".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "15".to_string(),
                                        read: BrwStatsBucketVals {
                                            count: "0".to_string(),
                                            pct: "0".to_string(),
                                            cum_pct: "0".to_string(),
                                        },
                                        write: BrwStatsBucketVals {
                                            count: "2".to_string(),
                                            pct: "6".to_string(),
                                            cum_pct: "81".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "16".to_string(),
                                        read: BrwStatsBucketVals {
                                            count: "0".to_string(),
                                            pct: "0".to_string(),
                                            cum_pct: "0".to_string(),
                                        },
                                        write: BrwStatsBucketVals {
                                            count: "2".to_string(),
                                            pct: "6".to_string(),
                                            cum_pct: "87".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "17".to_string(),
                                        read: BrwStatsBucketVals {
                                            count: "0".to_string(),
                                            pct: "0".to_string(),
                                            cum_pct: "0".to_string(),
                                        },
                                        write: BrwStatsBucketVals {
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "90".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "18".to_string(),
                                        read: BrwStatsBucketVals {
                                            count: "0".to_string(),
                                            pct: "0".to_string(),
                                            cum_pct: "0".to_string(),
                                        },
                                        write: BrwStatsBucketVals {
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "93".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "19".to_string(),
                                        read: BrwStatsBucketVals {
                                            count: "0".to_string(),
                                            pct: "0".to_string(),
                                            cum_pct: "0".to_string(),
                                        },
                                        write: BrwStatsBucketVals {
                                            count: "1".to_string(),
                                            pct: "3".to_string(),
                                            cum_pct: "96".to_string(),
                                        },
                                    },
                                    BrwStatsBuckets {
                                        name: "20".to_string(),
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
                                        name: "4".to_string(),
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
                                        name: "8".to_string(),
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
                                        name: "16".to_string(),
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
                                        name: "32".to_string(),
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
                                            count: "3".to_string(),
                                            pct: "33".to_string(),
                                            cum_pct: "44".to_string(),
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
                                            count: "5".to_string(),
                                            pct: "55".to_string(),
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
                        ]),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("checksum_dump".to_string()),
                        Value("0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("client_cache_count".to_string()),
                        Value("128".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("client_cache_seconds".to_string()),
                        Value("110".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("filesfree".to_string()),
                        Value("327382".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("filestotal".to_string()),
                        Value("327680".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("fstype".to_string()),
                        Value("osd-ldiskfs".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("grant_compat_disable".to_string()),
                        Value("0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("grant_precreate".to_string()),
                        Value("278208".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("instance".to_string()),
                        Value("1".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("ir_factor".to_string()),
                        Value("5".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("job_cleanup_interval".to_string()),
                        Value("600".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("kbytesavail".to_string()),
                        Value("4486468".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("kbytesfree".to_string()),
                        Value("4764996".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("kbytestotal".to_string()),
                        Value("4831716".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("last_id".to_string()),
                        Value("0x100000000:65".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("lfsck_speed_limit".to_string()),
                        Value("0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("num_exports".to_string()),
                        Value("2".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("precreate_batch".to_string()),
                        Value("128".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("read_cache_enable".to_string()),
                        Value("1".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("readcache_max_filesize".to_string()),
                        Value("18446744073709551615".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("recovery_status".to_string()),
                        Value("status: INACTIVE".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("recovery_time_hard".to_string()),
                        Value("900".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("recovery_time_soft".to_string()),
                        Value("150".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("seqs_allocated".to_string()),
                        Value("1".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("site_stats".to_string()),
                        Value("20/89 89/262144 1 114 32 99 0 0 0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("soft_sync_limit".to_string()),
                        Value("16".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("sync_journal".to_string()),
                        Value("0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("sync_on_lock_cancel".to_string()),
                        Value("never".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("tot_dirty".to_string()),
                        Value("0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("tot_granted".to_string()),
                        Value("8666816".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("tot_pending".to_string()),
                        Value("0".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("uuid".to_string()),
                        Value("fs-OST0000_UUID".to_string()),
                    ),
                    (
                        TargetName("fs-OST0000".to_string()),
                        ParamName("writethrough_cache_enable".to_string()),
                        Value("1".to_string()),
                    ),
                ],
                ""
            ))
        );
    }
}
