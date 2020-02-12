// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, param, period, target},
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

pub const STATS: &str = "stats";
pub const THREADS_MIN: &str = "threads_min";
pub const THREADS_MAX: &str = "threads_max";
pub const THREADS_STARTED: &str = "threads_started";
pub const NUM_EXPORTS: &str = "num_exports";

pub fn params() -> Vec<String> {
    [
        format!("mgs.*.mgs.{}", STATS),
        format!("mgs.*.mgs.{}", THREADS_MAX),
        format!("mgs.*.mgs.{}", THREADS_MIN),
        format!("mgs.*.mgs.{}", THREADS_STARTED),
        format!("mgs.*.{}", NUM_EXPORTS),
    ]
    .iter()
    .map(|x| x.to_owned())
    .collect::<Vec<_>>()
}

#[derive(Debug)]
enum MgsStat {
    Stats(Vec<Stat>),
    ThreadsMin(u64),
    ThreadsMax(u64),
    ThreadsStarted(u64),
    NumExports(u64),
}

/// Parses the name of a target
fn target_name<I>() -> impl Parser<I, Output = Target>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        attempt(string("mgs")).skip(period()),
        target().skip(period()),
    )
        .map(|(_, x)| x)
        .message("while parsing target_name")
}

fn mgs_stat<I>() -> impl Parser<I, Output = (Param, MgsStat)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
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
                (
                    param(THREADS_STARTED),
                    digits().skip(newline()).map(MgsStat::ThreadsStarted),
                ),
            )),
        )
            .map(|(_, (y, z))| (y, z)),
    ))
}

pub fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (target_name(), mgs_stat())
        .map(|(target, (param, value))| match value {
            MgsStat::Stats(value) => TargetStats::Stats(TargetStat {
                kind: TargetVariant::MGT,
                target,
                param,
                value,
            }),
            MgsStat::NumExports(value) => TargetStats::NumExports(TargetStat {
                kind: TargetVariant::MGT,
                target,
                param,
                value,
            }),
            MgsStat::ThreadsMin(value) => TargetStats::ThreadsMin(TargetStat {
                kind: TargetVariant::MGT,
                target,
                param,
                value,
            }),
            MgsStat::ThreadsMax(value) => TargetStats::ThreadsMax(TargetStat {
                kind: TargetVariant::MGT,
                target,
                param,
                value,
            }),
            MgsStat::ThreadsStarted(value) => TargetStats::ThreadsStarted(TargetStat {
                kind: TargetVariant::MGT,
                target,
                param,
                value,
            }),
        })
        .map(Record::Target)
        .message("while parsing mgs params")
}
