// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    choice,
    error::{ParseError, StreamError},
    parser::char::{newline, string},
    stream::{Stream, StreamErrorFor},
    Parser,
};

use crate::{
    base_parsers::{digits, param, period, target},
    types::{Param, Record, Target, TargetStat, TargetStats, TargetVariant},
};

pub const CONTENDED_LOCKS: &str = "contended_locks";
pub const CONTENTION_SECONDS: &str = "contention_seconds";
pub const CTIME_AGE_LIMIT: &str = "ctime_age_limit";
pub const EARLY_LOCK_CANCEL: &str = "early_lock_cancel";
pub const LOCK_COUNT: &str = "lock_count";
pub const LOCK_TIMEOUTS: &str = "lock_timeouts";
pub const LOCK_UNUSED_COUNT: &str = "lock_unused_count";
pub const LRU_MAX_AGE: &str = "lru_max_age";
pub const LRU_SIZE: &str = "lru_size";
pub const MAX_NOLOCK_BYTES: &str = "max_nolock_bytes";
pub const MAX_PARALLEL_AST: &str = "max_parallel_ast";
pub const RESOURCE_COUNT: &str = "resource_count";

pub const LDLM_STATS: [&str; 12] = [
    CONTENDED_LOCKS,
    CONTENTION_SECONDS,
    CTIME_AGE_LIMIT,
    EARLY_LOCK_CANCEL,
    LOCK_COUNT,
    LOCK_TIMEOUTS,
    LOCK_UNUSED_COUNT,
    LRU_MAX_AGE,
    LRU_SIZE,
    MAX_NOLOCK_BYTES,
    MAX_PARALLEL_AST,
    RESOURCE_COUNT,
];

/// Takes LDLM_STATS and produces a list of params for
/// consumption in proper ltcl get_param format.
pub fn ldlm_params() -> Vec<String> {
    LDLM_STATS
        .into_iter()
        .map(|x| format!("ldlm.namespaces.{{mdt-,filter-}}*.{}", x))
        .collect::<Vec<String>>()
}

/// Parses the name of the target
pub fn ldlm_target<I>() -> impl Parser<Input = I, Output = Target>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        string("ldlm.namespaces."),
        choice((string("mdt-"), string("filter-"))).with(target()),
    )
        .and_then(|(_, Target(x))| {
            let xs: Vec<&str> = x.split("_UUID").collect();

            match xs.as_slice() {
                [y, _] => Ok(Target(y.to_string())),
                _ => Err(StreamErrorFor::<I>::expected_static_message("_UUID")),
            }
        })
        .skip(period())
        .message("while parsing lock_namespaces")
}

pub fn ldlm_stat<I>() -> impl Parser<Input = I, Output = (Param, u64)>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice((
        (param(CONTENDED_LOCKS), digits().skip(newline())),
        (param(CONTENTION_SECONDS), digits().skip(newline())),
        (param(CTIME_AGE_LIMIT), digits().skip(newline())),
        (param(EARLY_LOCK_CANCEL), digits().skip(newline())),
        (param(LOCK_COUNT), digits().skip(newline())),
        (param(LOCK_TIMEOUTS), digits().skip(newline())),
        (param(LOCK_UNUSED_COUNT), digits().skip(newline())),
        (param(LRU_MAX_AGE), digits().skip(newline())),
        (param(LRU_SIZE), digits().skip(newline())),
        (param(MAX_NOLOCK_BYTES), digits().skip(newline())),
        (param(MAX_PARALLEL_AST), digits().skip(newline())),
        (param(RESOURCE_COUNT), digits().skip(newline())),
    ))
}

pub fn parse<I>() -> impl Parser<Input = I, Output = Record>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (ldlm_target(), ldlm_stat())
        .and_then(|(target, (Param(p), value))| match p.clone().as_ref() {
            CONTENDED_LOCKS => Ok(TargetStats::ContendedLocks(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            CONTENTION_SECONDS => Ok(TargetStats::ContentionSeconds(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            CTIME_AGE_LIMIT => Ok(TargetStats::CtimeAgeLimit(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            EARLY_LOCK_CANCEL => Ok(TargetStats::EarlyLockCancel(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            LOCK_COUNT => Ok(TargetStats::LockCount(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            LOCK_TIMEOUTS => Ok(TargetStats::LockTimeouts(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            LOCK_UNUSED_COUNT => Ok(TargetStats::LockUnusedCount(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            LRU_MAX_AGE => Ok(TargetStats::LruMaxAge(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            LRU_SIZE => Ok(TargetStats::LruSize(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            MAX_NOLOCK_BYTES => Ok(TargetStats::MaxNolockBytes(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            MAX_PARALLEL_AST => Ok(TargetStats::MaxParallelAst(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            RESOURCE_COUNT => Ok(TargetStats::ResourceCount(TargetStat {
                kind: TargetVariant::OST,
                target,
                param: Param(p),
                value,
            })),
            _ => Err(StreamErrorFor::<I>::unexpected_static_message(
                "Unexpected top-level param",
            )),
        })
        .map(Record::Target)
        .message("while parsing ldlm")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldlm_params() {
        assert_eq!(
            ldlm_params(),
            vec![
                "ldlm.namespaces.{mdt-,filter-}*.contended_locks".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.contention_seconds".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.ctime_age_limit".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.early_lock_cancel".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.lock_count".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.lock_timeouts".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.lock_unused_count".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.lru_max_age".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.lru_size".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.max_nolock_bytes".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.max_parallel_ast".to_string(),
                "ldlm.namespaces.{mdt-,filter-}*.resource_count".to_string(),
            ]
        )
    }

    #[test]
    fn test_lock_namespaces() {
        let result = ldlm_stat().easy_parse("contended_locks=32\n");

        let r = Ok(((Param(CONTENDED_LOCKS.to_string()), 32), ""));

        assert_eq!(result, r);
    }
}
