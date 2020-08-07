// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    base_parsers::{period, target, word},
    types::{FsName, Param, Record, Target, TargetStat, TargetStats, TargetVariant},
};
use combine::{
    attempt,
    error::ParseError,
    many1,
    parser::char::{newline, string},
    stream::Stream,
    Parser,
};
use std::collections::HashMap;

pub fn params() -> Vec<String> {
    vec!["mgs.*.live.*".to_string()]
}

#[derive(Debug)]
enum MgsFsStat {
    MgsFsNames(Vec<(Target, FsName)>),
}

fn fsname<I>() -> impl Parser<I, Output = (Target, FsName)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        attempt(string("mgs")).skip(period()),
        target().skip(period()),
        string("live").skip(period()),
        word().skip(newline()).map(FsName),
    )
        .map(|(_, target, _, fsname)| (target, fsname))
}

fn fsnames<I>() -> impl Parser<I, Output = Vec<(Target, FsName)>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    many1(fsname())
}

fn mgs_fs_stat<I>() -> impl Parser<I, Output = (Param, MgsFsStat)>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    fsnames()
        .map(|xs| {
            let xs: Vec<(Target, FsName)> = xs
                .into_iter()
                .filter(|(_, name)| name.0 != "params")
                .collect();

            xs
        })
        .map(|xs| (Param("fsnames".into()), MgsFsStat::MgsFsNames(xs)))
}

pub fn parse<I>() -> impl Parser<I, Output = Vec<Record>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (mgs_fs_stat())
        .map(|(param, stat)| match stat {
            MgsFsStat::MgsFsNames(xs) => {
                let mgs_map: HashMap<Target, Vec<FsName>> =
                    xs.into_iter()
                        .fold(HashMap::new(), |mut acc, (target, fs_name)| {
                            if let Some(fs_names) = acc.get(&target) {
                                let names: Vec<FsName> =
                                    [&fs_names[..], &vec![fs_name][..]].concat();
                                acc.insert(target, names);
                            } else {
                                acc.insert(target, vec![fs_name]);
                            }

                            acc
                        });

                mgs_map
                    .into_iter()
                    .map(|(target, fs_name)| {
                        TargetStats::FsNames(TargetStat {
                            kind: TargetVariant::MGT,
                            target,
                            param: param.clone(),
                            value: fs_name,
                        })
                    })
                    .map(Record::Target)
                    .collect::<Vec<_>>()
            }
        })
        .message("while parsing mgs fs params")
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::{many, parser::EasyParser};

    #[test]
    fn test_single_mounted_fs() {
        let x = r#"mgs.MGS.live.fs
mgs.MGS.live.params
"#;

        let result: (Vec<_>, _) = many(parse()).easy_parse(x).unwrap();
        let records = result.0.into_iter().flatten().collect::<Vec<Record>>();

        assert_eq!(
            vec![Record::Target(TargetStats::FsNames(TargetStat {
                kind: TargetVariant::MGT,
                param: Param("fsnames".into()),
                target: Target("MGS".into()),
                value: vec![FsName("fs".into())],
            }))],
            records
        );
    }

    #[test]
    fn test_multi_mounted_fs() {
        let x = r#"mgs.MGS.live.fs
mgs.MGS.live.fs2
mgs.MGS.live.params
"#;

        let result: (Vec<_>, _) = many(parse()).easy_parse(x).unwrap();
        let records = result.0.into_iter().flatten().collect::<Vec<Record>>();

        assert_eq!(
            vec![Record::Target(TargetStats::FsNames(TargetStat {
                kind: TargetVariant::MGT,
                param: Param("fsnames".into()),
                target: Target("MGS".into()),
                value: vec![FsName("fs".into()), FsName("fs2".into())],
            }))],
            records
        );
    }

    #[test]
    fn test_multi_target_multi_mounted_fs() {
        let x = r#"mgs.MGS.live.fs
mgs.MGS2.live.mgs2fs1
mgs.MGS.live.fs2
mgs.MGS.live.params
mgs.MGS2.live.mgs2fs2
"#;

        let result: (Vec<_>, _) = many(parse()).easy_parse(x).unwrap();
        let mut records = result.0.into_iter().flatten().collect::<Vec<Record>>();
        records.sort_by(|a, b| {
            let record_a = match a {
                Record::Target(TargetStats::FsNames(x)) => x,
                _ => panic!("Error getting target record."),
            };
            let record_b = match b {
                Record::Target(TargetStats::FsNames(x)) => x,
                _ => panic!("Error getting target record."),
            };

            let a1 = record_a.value[0].0.to_string();
            let b1 = record_b.value[0].0.to_string();

            a1.partial_cmp(&b1).unwrap()
        });

        assert_eq!(
            vec![
                Record::Target(TargetStats::FsNames(TargetStat {
                    kind: TargetVariant::MGT,
                    param: Param("fsnames".into()),
                    target: Target("MGS".into()),
                    value: vec![FsName("fs".into()), FsName("fs2".into())],
                })),
                Record::Target(TargetStats::FsNames(TargetStat {
                    kind: TargetVariant::MGT,
                    param: Param("fsnames".into()),
                    target: Target("MGS2".into()),
                    value: vec![FsName("mgs2fs1".into()), FsName("mgs2fs2".into())],
                }))
            ],
            records
        );
    }
}
