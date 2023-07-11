// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{digits, param, period, target, till_eof, till_newline},
    types::{Param, Record, RecoveryStatus, Target, TargetStat, TargetStats, TargetVariant},
};
use combine::{
    attempt, choice, eof, many, optional,
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

fn clients_line<I>(x: &'static str) -> impl Parser<I, Output = u64>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        attempt(string(x).skip(optional(token(':')))),
        spaces(),
        digits(),
        optional((token('/'), digits())),
        optional(newline().map(drop).or(eof())),
    )
        .map(|(_, _, x, _, _): (_, _, u64, _, _)| x)
}

#[derive(Debug)]
enum RecoveryStat {
    Status(RecoveryStatus),
    Completed(u64),
    Connected(u64),
    Evicted(u64),
}

fn target_recovery_stats<I>() -> impl Parser<I, Output = Vec<RecoveryStat>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many(choice((
        status_line()
            .skip(optional(newline()))
            .map(RecoveryStat::Status)
            .map(Some),
        clients_line("completed_clients")
            .skip(optional(newline()))
            .map(RecoveryStat::Completed)
            .map(Some),
        clients_line("connected_clients")
            .skip(optional(newline()))
            .map(RecoveryStat::Connected)
            .map(Some),
        clients_line("evicted_clients")
            .skip(optional(newline()))
            .map(RecoveryStat::Evicted)
            .map(Some),
        attempt((
            target(),
            token(':'),
            till_newline().skip(newline()).or(till_eof().skip(eof())),
        ))
        .map(|_| None),
    )))
    .map(|xs: Vec<_>| xs.into_iter().flatten().collect())
}

fn target_status<I>() -> impl Parser<I, Output = Vec<TargetStats>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        target_info().skip(optional(newline())),
        target_recovery_stats(),
    )
        .map(|((kind, target, param), values)| {
            values
                .iter()
                .map(|value| match value {
                    RecoveryStat::Status(value) => TargetStats::RecoveryStatus(TargetStat {
                        kind,
                        param: param.clone(),
                        target: target.clone(),
                        value: *value,
                    }),
                    RecoveryStat::Completed(value) => {
                        TargetStats::RecoveryCompletedClients(TargetStat {
                            kind,
                            param: param.clone(),
                            target: target.clone(),
                            value: *value,
                        })
                    }
                    RecoveryStat::Connected(value) => {
                        TargetStats::RecoveryConnectedClients(TargetStat {
                            kind,
                            param: param.clone(),
                            target: target.clone(),
                            value: *value,
                        })
                    }
                    RecoveryStat::Evicted(value) => {
                        TargetStats::RecoveryEvictedClients(TargetStat {
                            kind,
                            param: param.clone(),
                            target: target.clone(),
                            value: *value,
                        })
                    }
                })
                .collect()
        })
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
            .map(|(x, _)| x.into_iter().map(Record::Target).collect()),
    )
    .map(|x: Vec<Vec<Record>>| x.into_iter().flatten().collect())
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
