// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use base_parsers::{digits, param, word};
use combine::{choice, error::ParseError, parser::char::newline, Parser, Stream};
use stats::{HostStat, HostStats};

pub const MEMUSED_MAX: &str = "memused_max";
pub const MEMUSED: &str = "memused";
pub const LNET_MEMUSED: &str = "lnet_memused";
pub const HEALTH_CHECK: &str = "health_check";

pub const TOP_LEVEL_PARAMS: [&str; 4] = [MEMUSED, MEMUSED_MAX, LNET_MEMUSED, HEALTH_CHECK];

pub fn top_level_params() -> Vec<String> {
    TOP_LEVEL_PARAMS
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}

pub fn parse<I>() -> impl Parser<Input = I, Output = HostStats>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        (param(MEMUSED), digits()).map(|(param, value)| {
            HostStats::MemUsed(HostStat {
                host: None,
                param,
                value,
            })
        }),
        (param(MEMUSED_MAX), digits()).map(|(param, value)| {
            HostStats::MemUsedMax(HostStat {
                host: None,
                param,
                value,
            })
        }),
        (param(LNET_MEMUSED), digits()).map(|(param, value)| {
            HostStats::LNetMemUsed(HostStat {
                host: None,
                param,
                value,
            })
        }),
        (param(HEALTH_CHECK), word()).map(|(param, value)| {
            HostStats::Health(HostStat {
                host: None,
                param,
                value,
            })
        }),
    )).skip(newline())
        .message("while getting top_level_param")
}

#[cfg(test)]
mod tests {

    use super::*;
    use combine::stream::state::{SourcePosition, State};
    use stats::{HostStat, HostStats, Param};

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
                HostStats::MemUsedMax(HostStat {
                    host: None,
                    param: Param(MEMUSED_MAX.to_string()),
                    value: 77991501
                }),
                State {
                    input: "",
                    positioner: SourcePosition { line: 2, column: 1 }
                }
            ))
        )
    }
}
