// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, param, period, target, till_newline},
    oss::brw_stats_parser::brw_stats,
    oss::job_stats,
    stats_parser::stats,
    types::{
        BrwStats, JobStatOst, Param, Record, Stat, Target, TargetStat, TargetStats, TargetVariant,
    },
};
use combine::{
    choice,
    error::ParseError,
    parser::char::{newline, string},
    stream::Stream,
    Parser,
};

pub const JOBSTATS: &str = "job_stats";
pub const STATS: &str = "stats";
pub const BRW_STATS: &str = "brw_stats";
pub const FILES_FREE: &str = "filesfree";
pub const FILES_TOTAL: &str = "filestotal";
pub const FS_TYPE: &str = "fstype";
pub const KBYTES_AVAIL: &str = "kbytesavail";
pub const KBYTES_FREE: &str = "kbytesfree";
pub const KBYTES_TOTAL: &str = "kbytestotal";
pub const NUM_EXPORTS: &str = "num_exports";
pub const TOT_DIRTY: &str = "tot_dirty";
pub const TOT_GRANTED: &str = "tot_granted";
pub const TOT_PENDING: &str = "tot_pending";

pub const OBD_STATS: [&str; 13] = [
    JOBSTATS,
    STATS,
    BRW_STATS,
    FILES_FREE,
    FILES_TOTAL,
    FS_TYPE,
    KBYTES_AVAIL,
    KBYTES_FREE,
    KBYTES_TOTAL,
    NUM_EXPORTS,
    TOT_DIRTY,
    TOT_GRANTED,
    TOT_PENDING,
];

/// Takes OBD_STATS and produces a list of params for
/// consumption in proper ltcl get_param format.
pub fn obd_params() -> Vec<String> {
    OBD_STATS
        .iter()
        .map(|x| format!("obdfilter.*OST*.{}", x))
        .collect()
}

/// Parses the name of a target
fn target_name<I>() -> impl Parser<I, Output = Target>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (string("obdfilter").skip(period()), target().skip(period()))
        .map(|(_, x)| x)
        .message("while parsing target_name")
}

#[derive(Debug)]
enum ObdfilterStat {
    JobStats(Option<Vec<JobStatOst>>),
    Stats(Vec<Stat>),
    BrwStats(Vec<BrwStats>),
    /// Available inodes
    FilesFree(u64),
    /// Total inodes
    FilesTotal(u64),
    /// Type of target
    FsType(String),
    /// Available disk space
    BytesAvail(u64),
    /// Free disk space
    BytesFree(u64),
    /// Total disk space
    BytesTotal(u64),
    NumExports(u64),
    TotDirty(u64),
    TotGranted(u64),
    TotPending(u64),
}

fn obdfilter_stat<I>() -> impl Parser<I, Output = (Param, ObdfilterStat)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (
            param(JOBSTATS),
            job_stats::parse().map(ObdfilterStat::JobStats),
        ),
        (param(STATS), stats().map(ObdfilterStat::Stats)),
        (param(BRW_STATS), brw_stats().map(ObdfilterStat::BrwStats)),
        (
            param(FILES_FREE),
            digits().skip(newline()).map(ObdfilterStat::FilesFree),
        ),
        (
            param(FILES_TOTAL),
            digits().skip(newline()).map(ObdfilterStat::FilesTotal),
        ),
        (
            param(FS_TYPE),
            till_newline().skip(newline()).map(ObdfilterStat::FsType),
        ),
        (
            param(KBYTES_AVAIL),
            digits()
                .skip(newline())
                .map(|x| x * 1024)
                .map(ObdfilterStat::BytesAvail),
        ),
        (
            param(KBYTES_FREE),
            digits()
                .skip(newline())
                .map(|x| x * 1024)
                .map(ObdfilterStat::BytesFree),
        ),
        (
            param(KBYTES_TOTAL),
            digits()
                .skip(newline())
                .map(|x| x * 1024)
                .map(ObdfilterStat::BytesTotal),
        ),
        (
            param(NUM_EXPORTS),
            digits().skip(newline()).map(ObdfilterStat::NumExports),
        ),
        (
            param(TOT_DIRTY),
            digits().skip(newline()).map(ObdfilterStat::TotDirty),
        ),
        (
            param(TOT_GRANTED),
            digits().skip(newline()).map(ObdfilterStat::TotGranted),
        ),
        (
            param(TOT_PENDING),
            digits().skip(newline()).map(ObdfilterStat::TotPending),
        ),
    ))
    .message("while parsing obdfilter")
}

pub fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (target_name(), obdfilter_stat())
        .map(|(target, (param, value))| match value {
            ObdfilterStat::JobStats(value) => TargetStats::JobStatsOst(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::Stats(value) => TargetStats::Stats(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::BrwStats(value) => TargetStats::BrwStats(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::FilesFree(value) => TargetStats::FilesFree(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::FilesTotal(value) => TargetStats::FilesTotal(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::FsType(value) => TargetStats::FsType(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::BytesAvail(value) => TargetStats::BytesAvail(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::BytesFree(value) => TargetStats::BytesFree(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::BytesTotal(value) => TargetStats::BytesTotal(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::NumExports(value) => TargetStats::NumExports(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::TotDirty(value) => TargetStats::TotDirty(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::TotGranted(value) => TargetStats::TotGranted(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
            ObdfilterStat::TotPending(value) => TargetStats::TotPending(TargetStat {
                kind: TargetVariant::Ost,
                target,
                param,
                value,
            }),
        })
        .map(Record::Target)
        .message("while parsing obdfilter")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_name() {
        let result = target_name().parse("obdfilter.fs-OST0000.num_exports=");

        assert_eq!(
            result,
            Ok((Target("fs-OST0000".to_string()), "num_exports="))
        );
    }
}
