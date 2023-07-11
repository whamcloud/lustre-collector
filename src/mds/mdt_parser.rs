// Copyright (c) 2023 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, param, param_period, period, target}, exports_parser::exports_stats, mds::job_stats, oss::obdfilter_parser::{EXPORTS, EXPORTS_PARAMS}, stats_parser::stats, types::{JobStatMdt, Param, Record, Stat, Target, TargetStat, TargetStats, TargetVariant}, ExportStats
};
use combine::{
    attempt, choice,
    error::ParseError,
    parser::char::{newline, string},
    stream::Stream,
    Parser,
};

pub(crate) const JOB_STATS: &str = "job_stats";
pub(crate) const STATS: &str = "md_stats";
pub(crate) const NUM_EXPORTS: &str = "num_exports";
pub(crate) const FILES_FREE: &str = "filesfree";
pub(crate) const FILES_TOTAL: &str = "filestotal";
pub(crate) const KBYTES_AVAIL: &str = "kbytesavail";
pub(crate) const KBYTES_FREE: &str = "kbytesfree";
pub(crate) const KBYTES_TOTAL: &str = "kbytestotal";

enum MdtStat {
    JobStats(Option<Vec<JobStatMdt>>),
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
    ExportStats(Vec<ExportStats>),
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
        (param(JOB_STATS), job_stats::parse().map(MdtStat::JobStats))
            .message("while parsing job_stats"),
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
        (
            param_period(EXPORTS),
            exports_stats().map(MdtStat::ExportStats),
        ),
    ))
}

pub(crate) fn params() -> Vec<String> {
    [
        format!("mdt.*.{JOB_STATS}"),
        format!("mdt.*.{STATS}"),
        format!("mdt.*MDT*.{NUM_EXPORTS}"),
        format!("mdt.*MDT*.{EXPORTS_PARAMS}"),
    ]
    .into_iter()
    .collect()
}

fn target_name<I>() -> impl Parser<I, Output = Target>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        attempt(string("mdt")).skip(period()),
        target().skip(period()),
    )
        .map(|(_, x)| x)
        .message("while parsing target_name")
}

pub(crate) fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (target_name(), mdt_stat())
        .map(|(target, (param, value))| match value {
            MdtStat::JobStats(value) => TargetStats::JobStatsMdt(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::Stats(value) => TargetStats::Stats(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::NumExports(value) => TargetStats::NumExports(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::FilesFree(value) => TargetStats::FilesFree(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::FilesTotal(value) => TargetStats::FilesTotal(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::BytesAvail(value) => TargetStats::KBytesAvail(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::BytesFree(value) => TargetStats::KBytesFree(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::BytesTotal(value) => TargetStats::KBytesTotal(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
            MdtStat::ExportStats(value) => TargetStats::ExportStats(TargetStat {
                kind: TargetVariant::Mdt,
                target,
                param,
                value,
            }),
        })
        .map(Record::Target)
        .message("while parsing mdt")
}
