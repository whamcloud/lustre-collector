// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use base_parsers::{digits, param, period, till_newline};
use combine::error::ParseError;
use combine::parser::char::{alpha_num, newline, string};
use combine::parser::choice::choice;
use combine::stream::Stream;
use combine::{many1, one_of, Parser};
use oss::brw_stats_parser::brw_stats;
use stats::{ObdFilterStat, ObdFilterStats, Param, Target};
use stats_parser::stats;

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

pub const OBD_STATS: [&str; 12] = [
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
        .into_iter()
        .map(|x| format!("obdfilter.*OST*.{}", x))
        .collect::<Vec<String>>()
}

/// Parses the name of a target
fn target_name<I>() -> impl Parser<Input = I, Output = Target>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("obdfilter").skip(period()),
        many1(alpha_num().or(one_of("_-".chars()))).skip(period()),
    ).map(|(_, target)| Target(target))
        .message("while parsing target_name")
}

fn obdfilter_stat<I>() -> impl Parser<Input = I, Output = (Param, ObdFilterStats)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        // "job_stats",
        (param(STATS), stats().map(ObdFilterStats::Stats)),
        (param(BRW_STATS), brw_stats().map(ObdFilterStats::BrwStats)),
        (
            param(FILES_FREE),
            digits().skip(newline()).map(ObdFilterStats::FilesFree),
        ),
        (
            param(FILES_TOTAL),
            digits().skip(newline()).map(ObdFilterStats::FilesTotal),
        ),
        (
            param(FILES_TOTAL),
            digits().skip(newline()).map(ObdFilterStats::FilesTotal),
        ),
        (
            param(FS_TYPE),
            till_newline().skip(newline()).map(ObdFilterStats::FsType),
        ),
        (
            param(KBYTES_AVAIL),
            digits()
                .skip(newline())
                .map(|x| ObdFilterStats::BytesAvail(x * 1024)),
        ),
        (
            param(KBYTES_FREE),
            digits()
                .skip(newline())
                .map(|x| ObdFilterStats::BytesFree(x * 1024)),
        ),
        (
            param(KBYTES_TOTAL),
            digits()
                .skip(newline())
                .map(|x| ObdFilterStats::BytesTotal(x * 1024)),
        ),
        (
            param(NUM_EXPORTS),
            digits().skip(newline()).map(ObdFilterStats::NumExports),
        ),
        (
            param(TOT_DIRTY),
            digits().skip(newline()).map(ObdFilterStats::TotDirty),
        ),
        (
            param(TOT_GRANTED),
            digits().skip(newline()).map(ObdFilterStats::TotGranted),
        ),
        (
            param(TOT_PENDING),
            digits().skip(newline()).map(ObdFilterStats::TotPending),
        ),
    )).message("while getting obdfilter_stat")
}

pub fn parse<I>() -> impl Parser<Input = I, Output = ObdFilterStat>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (target_name(), obdfilter_stat())
        .map(|(target, (param, value))| ObdFilterStat {
            host: None,
            target,
            param,
            value,
        })
        .message("while parsing")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_name() {
        let result = target_name().easy_parse("obdfilter.fs-OST0000.num_exports=");

        assert_eq!(
            result,
            Ok((Target("fs-OST0000".to_string()), "num_exports="))
        );
    }
}
