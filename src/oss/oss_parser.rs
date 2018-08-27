// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{choice, error::ParseError, many, Parser, Stream};
use oss::{obdfilter_parser, top_level_parser};
use stats::Stats;

pub fn params() -> Vec<String> {
    let mut a = top_level_parser::top_level_params();
    a.extend(obdfilter_parser::obd_params());

    a
}

pub fn parse<I>() -> impl Parser<Input = I, Output = Vec<Stats>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many(choice((
        top_level_parser::parse().map(Stats::HostStats),
        obdfilter_parser::parse().map(Stats::TargetStats),
    )))
}

#[cfg(test)]
mod tests {

    use super::*;
    use combine::stream::state::{SourcePosition, State};
    use stats::{
        BrwStats, BrwStatsBucket, HostStat, HostStats, ObdFilterStat, ObdFilterStats, Param, Stat,
        Stats, Target,
    };

    #[test]
    fn test_params() {
        let x = State::new(
            r#"memused=77988429
memused_max=77991501
lnet_memused=8082453
health_check=healthy
obdfilter.fs-OST0000.stats=
snapshot_time             1535148988.363769785 secs.nsecs
write_bytes               9 samples [bytes] 98303 4194304 33554431
create                    4 samples [reqs]
statfs                    42297 samples [reqs]
get_info                  2 samples [reqs]
connect                   6 samples [reqs]
reconnect                 1 samples [reqs]
disconnect                4 samples [reqs]
statfs                    46806 samples [reqs]
preprw                    9 samples [reqs]
commitrw                  9 samples [reqs]
ping                      8229 samples [reqs]
obdfilter.fs-OST0000.brw_stats=
snapshot_time:         1535148988.364041639 (secs.nsecs)

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
obdfilter.fs-OST0000.filesfree=327382
obdfilter.fs-OST0000.filestotal=327680
obdfilter.fs-OST0000.fstype=osd-ldiskfs
obdfilter.fs-OST0000.kbytesavail=4486468
obdfilter.fs-OST0000.kbytesfree=4764996
obdfilter.fs-OST0000.kbytestotal=4831716
obdfilter.fs-OST0000.num_exports=2
obdfilter.fs-OST0000.tot_dirty=0
obdfilter.fs-OST0000.tot_granted=8666816
obdfilter.fs-OST0000.tot_pending=0
"#,
        );

        let result = parse().easy_parse(x);

        assert_eq!(
            result,
            Ok((
                vec![
                    Stats::HostStats(HostStats::MemUsed(HostStat {
                        host: None,
                        param: Param("memused".to_string()),
                        value: 77988429,
                    })),
                    Stats::HostStats(HostStats::MemUsedMax(HostStat {
                        host: None,
                        param: Param("memused_max".to_string()),
                        value: 77991501,
                    })),
                    Stats::HostStats(HostStats::LNetMemUsed(HostStat {
                        host: None,
                        param: Param("lnet_memused".to_string()),
                        value: 8082453,
                    })),
                    Stats::HostStats(HostStats::Health(HostStat {
                        host: None,
                        param: Param("health_check".to_string()),
                        value: "healthy".to_string(),
                    })),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("stats".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::Stats(vec![
                            Stat {
                                name: "write_bytes".to_string(),
                                units: "bytes".to_string(),
                                samples: 9,
                                min: Some(98303),
                                max: Some(4194304),
                                sum: Some(33554431),
                                sumsquare: None,
                            },
                            Stat {
                                name: "create".to_string(),
                                units: "reqs".to_string(),
                                samples: 4,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "statfs".to_string(),
                                units: "reqs".to_string(),
                                samples: 42297,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "get_info".to_string(),
                                units: "reqs".to_string(),
                                samples: 2,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "connect".to_string(),
                                units: "reqs".to_string(),
                                samples: 6,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "reconnect".to_string(),
                                units: "reqs".to_string(),
                                samples: 1,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "disconnect".to_string(),
                                units: "reqs".to_string(),
                                samples: 4,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "statfs".to_string(),
                                units: "reqs".to_string(),
                                samples: 46806,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "preprw".to_string(),
                                units: "reqs".to_string(),
                                samples: 9,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "commitrw".to_string(),
                                units: "reqs".to_string(),
                                samples: 9,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                            Stat {
                                name: "ping".to_string(),
                                units: "reqs".to_string(),
                                samples: 8229,
                                min: None,
                                max: None,
                                sum: None,
                                sumsquare: None,
                            },
                        ]),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("brw_stats".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::BrwStats(vec![
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
                                        write: 2,
                                    },
                                    BrwStatsBucket {
                                        name: 2,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 3,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 4,
                                        read: 0,
                                        write: 1,
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
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 10,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 11,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 12,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 13,
                                        read: 0,
                                        write: 2,
                                    },
                                    BrwStatsBucket {
                                        name: 14,
                                        read: 0,
                                        write: 2,
                                    },
                                    BrwStatsBucket {
                                        name: 15,
                                        read: 0,
                                        write: 2,
                                    },
                                    BrwStatsBucket {
                                        name: 16,
                                        read: 0,
                                        write: 2,
                                    },
                                    BrwStatsBucket {
                                        name: 17,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 18,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 19,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 20,
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
                                        name: 4,
                                        read: 0,
                                        write: 1,
                                    },
                                    BrwStatsBucket {
                                        name: 8,
                                        read: 0,
                                        write: 0,
                                    },
                                    BrwStatsBucket {
                                        name: 16,
                                        read: 0,
                                        write: 0,
                                    },
                                    BrwStatsBucket {
                                        name: 32,
                                        read: 0,
                                        write: 0,
                                    },
                                    BrwStatsBucket {
                                        name: 64,
                                        read: 0,
                                        write: 0,
                                    },
                                    BrwStatsBucket {
                                        name: 128,
                                        read: 0,
                                        write: 3,
                                    },
                                    BrwStatsBucket {
                                        name: 256,
                                        read: 0,
                                        write: 5,
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
                        ]),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("filesfree".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::FilesFree(327382),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("filestotal".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::FilesTotal(327680),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("fstype".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::FsType("osd-ldiskfs".to_string()),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("kbytesavail".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::BytesAvail(4594143232),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("kbytesfree".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::BytesFree(4879355904),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("kbytestotal".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::BytesTotal(4947677184),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("num_exports".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::NumExports(2),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("tot_dirty".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::TotDirty(0),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("tot_granted".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::TotGranted(8666816),
                    }),
                    Stats::TargetStats(ObdFilterStat {
                        host: None,
                        param: Param("tot_pending".to_string()),
                        target: Target("fs-OST0000".to_string()),
                        value: ObdFilterStats::TotPending(0),
                    }),
                ],
                State {
                    input: "",
                    positioner: SourcePosition {
                        line: 95,
                        column: 1,
                    },
                }
            ))
        )
    }
}