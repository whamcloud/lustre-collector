// Copyright (c) 2019 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, param, period, target, till_period},
    stats_parser::stats,
    types::{Param, Record, Stat, Target, TargetStat, TargetStats, TargetVariant},
};
use combine::{
    attempt, choice,
    error::ParseError,
    parser::char::{newline, string},
    stream::Stream,
    Parser,
};

pub const STATS: &str = "md_stats";
pub const NUM_EXPORTS: &str = "num_exports";
pub const FILES_FREE: &str = "filesfree";
pub const FILES_TOTAL: &str = "filestotal";
pub const KBYTES_AVAIL: &str = "kbytesavail";
pub const KBYTES_FREE: &str = "kbytesfree";
pub const KBYTES_TOTAL: &str = "kbytestotal";

enum MdtStat {
    Stats(Vec<Stat>),
    NumExports(u64),
    /// Available inodes
    FilesFree(u64),
    /// Total inodes
    FilesTotal(u64),
    /// Available disk space
    BytesAvail(u64),
    /// Free disk space
    BytesFree(u64),
    /// Total disk space
    BytesTotal(u64),
}

fn mdt_stat<I>() -> impl Parser<I, Output = (Param, MdtStat)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (
            param(NUM_EXPORTS),
            digits().skip(newline()).map(MdtStat::NumExports),
        ),
        (param(STATS), stats().map(MdtStat::Stats)).message("while parsing mdt_stat"),
        (
            param(FILES_FREE),
            digits().skip(newline()).map(MdtStat::FilesFree),
        ),
        (
            param(FILES_TOTAL),
            digits().skip(newline()).map(MdtStat::FilesTotal),
        ),
        (
            param(KBYTES_AVAIL),
            digits()
                .skip(newline())
                .map(|x| x * 1024)
                .map(MdtStat::BytesAvail),
        ),
        (
            param(KBYTES_FREE),
            digits()
                .skip(newline())
                .map(|x| x * 1024)
                .map(MdtStat::BytesFree),
        ),
        (
            param(KBYTES_TOTAL),
            digits()
                .skip(newline())
                .map(|x| x * 1024)
                .map(MdtStat::BytesTotal),
        ),
    ))
}

pub fn params() -> Vec<String> {
    [
        format!("mdt.*.{}", STATS),
        format!("mdt.*MDT*.{}", NUM_EXPORTS),
        format!("osd-*.*MDT*.{}", FILES_FREE),
        format!("osd-*.*MDT*.{}", FILES_TOTAL),
        format!("osd-*.*MDT*.{}", KBYTES_AVAIL),
        format!("osd-*.*MDT*.{}", KBYTES_FREE),
        format!("osd-*.*MDT*.{}", KBYTES_TOTAL),
    ]
    .iter()
    .map(|x| x.to_owned())
    .collect()
}

fn target_name<I>() -> impl Parser<I, Output = Target>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        attempt(choice((string("mdt"), string("osd-").skip(till_period())))).skip(period()),
        target().skip(period()),
    )
        .map(|(_, x)| x)
        .message("while parsing target_name")
}

pub fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (target_name(), mdt_stat())
        .map(|(target, (param, value))| match value {
            MdtStat::Stats(value) => TargetStats::Stats(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
            MdtStat::NumExports(value) => TargetStats::NumExports(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
            MdtStat::FilesFree(value) => TargetStats::FilesFree(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
            MdtStat::FilesTotal(value) => TargetStats::FilesTotal(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
            MdtStat::BytesAvail(value) => TargetStats::BytesAvail(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
            MdtStat::BytesFree(value) => TargetStats::BytesFree(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
            MdtStat::BytesTotal(value) => TargetStats::BytesTotal(TargetStat {
                kind: TargetVariant::MDT,
                target,
                param,
                value,
            }),
        })
        .map(Record::Target)
        .message("while parsing mdt")
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::{many, parser::EasyParser};
    use insta::assert_debug_snapshot;

    #[test]
    fn test_params() {
        let x = r#"mdt.fs-MDT0000.md_stats=
snapshot_time             1566017453.009677077 secs.nsecs
statfs                    20318 samples [reqs]
mdt.fs-MDT0001.md_stats=
snapshot_time             1566017453.009825550 secs.nsecs
statfs                    20805 samples [reqs]
mdt.fs-MDT0002.md_stats=
snapshot_time             1566017453.009857366 secs.nsecs
statfs                    20805 samples [reqs]
mdt.fs-MDT0000.num_exports=16
mdt.fs-MDT0001.num_exports=13
mdt.fs-MDT0002.num_exports=13
osd-ldiskfs.fs-MDT0000.filesfree=2095841
osd-ldiskfs.fs-MDT0000.filestotal=2097152
osd-ldiskfs.fs-MDT0001.filesfree=2096876
osd-ldiskfs.fs-MDT0001.filestotal=2097152
osd-ldiskfs.fs-MDT0002.filesfree=2096876
osd-ldiskfs.fs-MDT0002.filestotal=2097152
osd-ldiskfs.fs-MDT0000.kbytesavail=2584536
osd-ldiskfs.fs-MDT0000.kbytesfree=2845104
osd-ldiskfs.fs-MDT0000.kbytestotal=2913312
osd-ldiskfs.fs-MDT0001.kbytesavail=2632508
osd-ldiskfs.fs-MDT0001.kbytesfree=2893076
osd-ldiskfs.fs-MDT0001.kbytestotal=2913312
osd-ldiskfs.fs-MDT0002.kbytesavail=2632508
osd-ldiskfs.fs-MDT0002.kbytesfree=2893076
osd-ldiskfs.fs-MDT0002.kbytestotal=2913312
"#;

        let result: (Vec<_>, _) = many(parse()).easy_parse(x).unwrap();

        assert_debug_snapshot!(result)
    }
}
