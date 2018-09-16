// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use base_parsers::{digits, param, period, target};
use stats_parser::stats;
use types::{Param, Record, Stat, Target, TargetStat, TargetStats, TargetVariant};

use combine::{
    choice,
    error::{ParseError, StreamError},
    parser::char::{newline, string},
    stream::{Stream, StreamErrorFor},
    try, Parser,
};

pub const STATS: &str = "stats";
pub const THREADS_MIN: &str = "threads_min";
pub const THREADS_MAX: &str = "threads_max";
pub const NUM_EXPORTS: &str = "num_exports";

pub fn params() -> Vec<String> {
    [
        format!("mgs.*.mgs.{}", STATS),
        format!("mgs.*.mgs.{}", THREADS_MAX),
        format!("mgs.*.mgs.{}", THREADS_MIN),
        format!("mgs.*.{}", NUM_EXPORTS),
    ]
        .into_iter()
        .map(|x| x.to_owned())
        .collect::<Vec<_>>()
}

#[derive(Debug)]
enum MgsStat {
    Stats(Vec<Stat>),
    ThreadsMin(u64),
    ThreadsMax(u64),
    NumExports(u64),
}

/// Parses the name of a target
fn target_name<I>() -> impl Parser<Input = I, Output = Target>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (try(string("mgs")).skip(period()), target().skip(period()))
        .map(|(_, x)| x)
        .message("while parsing target_name")
}

fn mgs_stat<I>() -> impl Parser<Input = I, Output = (Param, MgsStat)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        (
            param(NUM_EXPORTS),
            digits().skip(newline()).map(MgsStat::NumExports),
        ),
        (
            string("mgs").skip(period()),
            choice((
                (param(STATS), stats().map(MgsStat::Stats)),
                (
                    param(THREADS_MIN),
                    digits().skip(newline()).map(MgsStat::ThreadsMin),
                ),
                (
                    param(THREADS_MAX),
                    digits().skip(newline()).map(MgsStat::ThreadsMax),
                ),
            )),
        )
            .map(|(_, (y, z))| (y, z)),
    ))
}

pub fn parse<I>() -> impl Parser<Input = I, Output = Record>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (target_name(), mgs_stat())
        .and_then(|(target, (param, value))| {
            #[allow(unreachable_patterns)]
            let r = match value {
                MgsStat::Stats(value) => Ok(TargetStats::Stats(TargetStat {
                    kind: TargetVariant::MGT,
                    target,
                    param,
                    value,
                })),
                MgsStat::NumExports(value) => Ok(TargetStats::NumExports(TargetStat {
                    kind: TargetVariant::MGT,
                    target,
                    param,
                    value,
                })),
                MgsStat::ThreadsMin(value) => Ok(TargetStats::ThreadsMin(TargetStat {
                    kind: TargetVariant::MGT,
                    target,
                    param,
                    value,
                })),
                MgsStat::ThreadsMax(value) => Ok(TargetStats::ThreadsMax(TargetStat {
                    kind: TargetVariant::MGT,
                    target,
                    param,
                    value,
                })),
                _ => Err(StreamErrorFor::<I>::expected_static_message(
                    "MgsStat Variant",
                )),
            };

            r
        }).map(Record::Target)
        .message("while parsing mgs params")
}
