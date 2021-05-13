// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{param, period, target},
    types::{Param, Record, RecoveryStatus, Target, TargetStat, TargetStats, TargetVariant},
};
use combine::{
    attempt, eof, many, optional,
    parser::{
        char::{newline, spaces, string},
        repeat::{skip_until, take_until},
    },
    stream::Stream,
    token, ParseError, Parser,
};

pub const RECOVERY_STATUS: &str = "recovery_status";

pub fn params() -> Vec<String> {
    vec![
        format!("obdfilter.*OST*.{}", RECOVERY_STATUS),
        format!("mdt.*MDT*.{}", RECOVERY_STATUS),
    ]
}

fn ost_or_mdt<I>() -> impl Parser<I, Output = TargetVariant>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    string("obdfilter")
        .map(|_| TargetVariant::Ost)
        .or(string("mdt").map(|_| TargetVariant::Mdt))
        .message("while parsing target_name")
}

/// Parses the name and kind of a target
fn target_info<I>() -> impl Parser<I, Output = (TargetVariant, Target, Param)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        ost_or_mdt().skip(period()),
        target().skip(period()),
        param(RECOVERY_STATUS),
    )
        .map(|(variant, x, param)| (variant, x, param))
        .message("while parsing target_name")
}

fn status_line<I>() -> impl Parser<I, Output = RecoveryStatus>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        string("status").skip(optional(token(':'))),
        spaces(),
        take_until(newline().map(drop).or(eof())),
    )
        .map(|(_, _, x): (_, _, String)| match x.as_ref() {
            "COMPLETE" => RecoveryStatus::Complete,
            "INACTIVE" => RecoveryStatus::Inactive,
            "WAITING" => RecoveryStatus::Waiting,
            "WAITING_FOR_CLIENTS" => RecoveryStatus::WaitingForClients,
            "RECOVERING" => RecoveryStatus::Recovering,
            _ => RecoveryStatus::Unknown,
        })
}

fn target_status<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        target_info().skip(optional(newline())),
        status_line().skip(optional(newline())),
    )
        .map(|((kind, target, param), value)| TargetStat {
            kind,
            param,
            target,
            value,
        })
        .map(TargetStats::RecoveryStatus)
        .map(Record::Target)
}

pub fn parse<I>() -> impl Parser<I, Output = Vec<Record>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many(
        (
            target_status(),
            skip_until(attempt(ost_or_mdt().map(drop)).or(eof())),
        )
            .map(|(x, _)| x),
    )
}

#[cfg(test)]
mod tests {
    use super::parse;
    use combine::{parser::EasyParser, stream::position};

    #[test]
    fn test_multiple() {
        let x = include_str!("../fixtures/recovery-multiple.txt");

        let (records, _): (Vec<_>, _) = parse().easy_parse(position::Stream::new(x)).unwrap();

        insta::assert_debug_snapshot!(records);
    }

    #[test]
    fn test_multiple_recovering() {
        let x = include_str!("../fixtures/recovery-multiple-recovering.txt");

        let (records, _): (Vec<_>, _) = parse().easy_parse(position::Stream::new(x)).unwrap();

        insta::assert_debug_snapshot!(records);
    }

    #[test]
    fn test_empty_input() {
        let (records, _): (Vec<_>, _) = parse().easy_parse(position::Stream::new("")).unwrap();

        assert_eq!(records, vec![]);
    }

    #[test]
    fn test_waiting_for_clients() {
        let x = include_str!("../fixtures/recovery-waiting-for-clients.txt");

        let (records, _): (Vec<_>, _) = parse().easy_parse(position::Stream::new(x)).unwrap();

        insta::assert_debug_snapshot!(records);
    }
}
