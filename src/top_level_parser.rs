// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    choice,
    error::{ParseError, StreamError},
    parser::char::newline,
    stream::{Stream, StreamErrorFor},
    Parser,
};

use crate::{
    base_parsers::{digits, param, word},
    types::{HostStat, HostStats, Param, Record},
};

pub const MEMUSED_MAX: &str = "memused_max";
pub const MEMUSED: &str = "memused";
pub const LNET_MEMUSED: &str = "lnet_memused";
pub const HEALTH_CHECK: &str = "health_check";

pub const TOP_LEVEL_PARAMS: [&str; 4] = [MEMUSED, MEMUSED_MAX, LNET_MEMUSED, HEALTH_CHECK];

pub fn top_level_params() -> Vec<String> {
    TOP_LEVEL_PARAMS
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}

enum TopLevelStat {
    Memused(u64),
    MemusedMax(u64),
    LnetMemused(u64),
    HealthCheck(String),
}

fn top_level_stat<I>() -> impl Parser<Input = I, Output = (Param, TopLevelStat)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        (param(MEMUSED), digits().map(TopLevelStat::Memused)),
        (param(MEMUSED_MAX), digits().map(TopLevelStat::MemusedMax)),
        (param(LNET_MEMUSED), digits().map(TopLevelStat::LnetMemused)),
        (param(HEALTH_CHECK), word().map(TopLevelStat::HealthCheck)),
    ))
    .skip(newline())
}

pub fn parse<I>() -> impl Parser<Input = I, Output = Record>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    top_level_stat()
        .and_then(|(param, v)| {
            #[allow(unreachable_patterns)]
            let r = match v {
                TopLevelStat::Memused(value) => Ok(HostStats::Memused(HostStat { param, value })),
                TopLevelStat::MemusedMax(value) => {
                    Ok(HostStats::MemusedMax(HostStat { param, value }))
                }
                TopLevelStat::LnetMemused(value) => {
                    Ok(HostStats::LNetMemUsed(HostStat { param, value }))
                }
                TopLevelStat::HealthCheck(value) => {
                    Ok(HostStats::HealthCheck(HostStat { param, value }))
                }
                _ => Err(StreamErrorFor::<I>::unexpected_static_message(
                    "Unexpected top-level param",
                )),
            };

            r
        })
        .map(Record::Host)
        .message("while parsing top_level_param")
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::{HostStat, HostStats, Param};
    use combine::stream::state::{SourcePosition, State};

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
        let result = parse().easy_parse(State::new("memused_max=77991501\n"));

        assert_eq!(
            result,
            Ok((
                Record::Host(HostStats::MemusedMax(HostStat {
                    param: Param(MEMUSED_MAX.to_string()),
                    value: 77_991_501
                })),
                State {
                    input: "",
                    positioner: SourcePosition { line: 2, column: 1 }
                }
            ))
        )
    }
}
