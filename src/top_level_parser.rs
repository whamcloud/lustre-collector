// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, param, words},
    types::{HostStat, HostStats, Param, Record},
};
use combine::{choice, error::ParseError, parser::char::newline, stream::Stream, Parser};

pub const MEMUSED_MAX: &str = "memused_max";
pub const MEMUSED: &str = "memused";
pub const LNET_MEMUSED: &str = "lnet_memused";
pub const HEALTH_CHECK: &str = "health_check";

pub const TOP_LEVEL_PARAMS: [&str; 4] = [MEMUSED, MEMUSED_MAX, LNET_MEMUSED, HEALTH_CHECK];

pub fn top_level_params() -> Vec<String> {
    TOP_LEVEL_PARAMS.iter().map(|x| (*x).to_string()).collect()
}

enum TopLevelStat {
    Memused(u64),
    MemusedMax(u64),
    LnetMemused(u64),
    HealthCheck(String),
}

fn top_level_stat<I>() -> impl Parser<I, Output = (Param, TopLevelStat)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((
        (param(MEMUSED), digits().map(TopLevelStat::Memused)),
        (param(MEMUSED_MAX), digits().map(TopLevelStat::MemusedMax)),
        (param(LNET_MEMUSED), digits().map(TopLevelStat::LnetMemused)),
        (param(HEALTH_CHECK), words().map(TopLevelStat::HealthCheck)),
    ))
    .skip(newline())
}

pub fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    top_level_stat()
        .map(|(param, v)| match v {
            TopLevelStat::Memused(value) => HostStats::Memused(HostStat { param, value }),
            TopLevelStat::MemusedMax(value) => HostStats::MemusedMax(HostStat { param, value }),
            TopLevelStat::LnetMemused(value) => HostStats::LNetMemUsed(HostStat { param, value }),
            TopLevelStat::HealthCheck(value) => HostStats::HealthCheck(HostStat { param, value }),
        })
        .map(Record::Host)
        .message("while parsing top_level_param")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::{HostStat, HostStats, Param};

    #[test]
    fn test_params() {
        assert_eq!(
            top_level_params(),
            vec![
                "memused".to_string(),
                "memused_max".to_string(),
                "lnet_memused".to_string(),
                "health_check".to_string(),
            ]
        )
    }

    #[test]
    fn test_row() {
        let result = parse().parse("memused_max=77991501\n");

        assert_eq!(
            result,
            Ok((
                Record::Host(HostStats::MemusedMax(HostStat {
                    param: Param(MEMUSED_MAX.to_string()),
                    value: 77_991_501
                })),
                ""
            ))
        )
    }
}
