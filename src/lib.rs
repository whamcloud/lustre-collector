// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

mod base_parsers;
pub(crate) mod brw_stats_parser;
pub mod error;
pub(crate) mod exports_parser;
pub(crate) mod ldlm;
pub(crate) mod llite;
mod lnetctl_parser;
mod mdd_parser;
mod mds;
pub mod mgs;
mod node_stats_parsers;
mod osd_parser;
mod oss;
pub mod parser;
pub(crate) mod quota;
pub mod recovery_status_parser;
mod stats_parser;
mod time;
mod top_level_parser;
pub mod types;
pub mod utils;

pub use crate::error::LustreCollectorError;
use crate::mgs::mgs_fs_parser;
use crate::utils::{get_output, CommandMock, CommandMode};
use combine::parser::EasyParser;
pub use lnetctl_parser::parse as parse_lnetctl_output;
pub use lnetctl_parser::parse_lnetctl_stats;
pub use node_stats_parsers::{parse_cpustats_output, parse_meminfo_output};
use std::panic;
use std::path::PathBuf;
use std::thread;
use std::{io, str};
pub use types::*;

fn check_output(records: Vec<Record>, state: &str) -> Result<Vec<Record>, LustreCollectorError> {
    let params = crate::parser::params().join(" ");

    if !state.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Content left in input buffer. Please run and supply to support: `lctl get_param {params}`"),
        )
        .into());
    }

    Ok(records)
}

/// Must be called with output of `lctl get_params` for all params returned from `parser::parse()`
pub fn parse_lctl_output(lctl_output: &[u8]) -> Result<Vec<Record>, LustreCollectorError> {
    let lctl_stats = str::from_utf8(lctl_output)?;

    let (lctl_record, state) = parser::parse()
        .easy_parse(lctl_stats)
        .map_err(|err| err.map_position(|p| p.translate_position(lctl_stats)))?;

    check_output(lctl_record, state)
}

pub fn parse_mgs_fs_output(mgs_fs_output: &[u8]) -> Result<Vec<Record>, LustreCollectorError> {
    let mgs_fs = str::from_utf8(mgs_fs_output)?;

    let (mgs_fs_record, state) = mgs::mgs_fs_parser::parse()
        .easy_parse(mgs_fs)
        .map_err(|err| err.map_position(|p| p.translate_position(mgs_fs)))?;

    check_output(mgs_fs_record, state)
}

pub fn parse_recovery_status_output(
    recovery_status_output: &[u8],
) -> Result<Vec<Record>, LustreCollectorError> {
    let recovery_status = str::from_utf8(recovery_status_output)?;
    let recovery_status = recovery_status.trim();

    let (recovery_statuses, state) = recovery_status_parser::parse()
        .easy_parse(recovery_status)
        .map_err(|err| err.map_position(|p| p.translate_position(recovery_status)))?;

    check_output(recovery_statuses, state)
}

fn get_lctl_output(mode: &CommandMode, path: &PathBuf) -> Result<Vec<u8>, LustreCollectorError> {
    let mock = CommandMock::new("lctl_output")
        .with_mode(*mode)
        .with_path(path);
    let mut args = vec!["get_param".to_string()];
    args.extend(parser::params());
    get_output("lctl", args, mock)
}

fn get_lctl_mgs_fs_output(
    mode: &CommandMode,
    path: &PathBuf,
) -> Result<Vec<u8>, LustreCollectorError> {
    let mock = CommandMock::new("lctl_mgs_fs_output")
        .with_mode(*mode)
        .with_path(path);
    let mut args = vec!["get_param".to_string(), "-N".to_string()];
    args.extend(mgs_fs_parser::params());
    get_output("lctl", args, mock)
}

fn get_recovery_status_output(
    mode: &CommandMode,
    path: &PathBuf,
) -> Result<Vec<u8>, LustreCollectorError> {
    let mock = CommandMock::new("lctl_recovery_status_output")
        .with_mode(*mode)
        .with_path(path);
    let mut args: Vec<String> = vec!["get_param".to_string(), "-N".to_string()];
    args.extend(recovery_status_parser::params());
    get_output("lctl", args, mock)
}

fn get_lnetctl_stats_output(
    mode: &CommandMode,
    path: &PathBuf,
) -> Result<Vec<u8>, LustreCollectorError> {
    let mock = CommandMock::new("lnetctl_stats_output")
        .with_mode(*mode)
        .with_path(path);
    get_output(
        "lnetctl",
        ["stats", "show"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
        mock,
    )
}

fn get_lnetctl_show_output(
    mode: &CommandMode,
    path: &PathBuf,
) -> Result<Vec<u8>, LustreCollectorError> {
    let mock = CommandMock::new("lnetctl_show_output")
        .with_mode(*mode)
        .with_path(path);
    get_output(
        "lnetctl",
        ["net", "show", "-v", "4"]
            .into_iter()
            .map(|x| x.to_string())
            .collect(),
        mock,
    )
}

pub fn parse(mode: &CommandMode, path: &PathBuf) -> Result<Vec<Record>, LustreCollectorError> {
    let mode_clone = *mode;
    let path_clone = path.clone();
    let handle = thread::spawn(move || -> Result<Vec<Record>, LustreCollectorError> {
        let lctl_output = get_lctl_output(&mode_clone, &path_clone)?;

        let lctl_record = parse_lctl_output(&lctl_output)?;

        Ok(lctl_record)
    });
    let mode_clone = *mode;
    let path_clone = path.clone();
    let mgs_fs_handle = thread::spawn(move || -> Result<Vec<Record>, LustreCollectorError> {
        let lctl_output = get_lctl_mgs_fs_output(&mode_clone, &path_clone)?;
        let lctl_record = parse_mgs_fs_output(&lctl_output)?;

        Ok(lctl_record)
    });

    let mode_clone = *mode;
    let path_clone = path.clone();
    let lnetctl_stats_handle =
        thread::spawn(move || -> Result<Vec<Record>, LustreCollectorError> {
            let lnetctl_stats_output = get_lnetctl_stats_output(&mode_clone, &path_clone)?;
            let lnetctl_stats_record = parse_lnetctl_stats(str::from_utf8(&lnetctl_stats_output)?)?;

            Ok(lnetctl_stats_record)
        });

    let mode_clone = *mode;
    let path_clone = path.clone();
    let recovery_status_handle =
        thread::spawn(move || -> Result<Vec<Record>, LustreCollectorError> {
            let recovery_status_output = get_recovery_status_output(&mode_clone, &path_clone)?;
            let recovery_statuses = parse_recovery_status_output(&recovery_status_output)?;

            Ok(recovery_statuses)
        });

    let lnetctl_net_show_output = get_lnetctl_show_output(mode, path)?;

    let lnetctl_net_show_stats = str::from_utf8(&lnetctl_net_show_output)
        .expect("while converting 'lnetctl net show -v 4' stdout from utf8");

    let mut lnet_record = parse_lnetctl_output(lnetctl_net_show_stats)
        .expect("while parsing 'lnetctl net show -v 4' stats");

    let mut lctl_record = match handle.join() {
        Ok(r) => r?,
        Err(e) => panic::resume_unwind(e),
    };

    let mut mgs_fs_record = match mgs_fs_handle.join() {
        Ok(r) => r.unwrap_or_default(),
        Err(e) => panic::resume_unwind(e),
    };

    let mut recovery_status_records = match recovery_status_handle.join() {
        Ok(r) => r.unwrap_or_default(),
        Err(e) => panic::resume_unwind(e),
    };

    let mut lnetctl_stats_record = match lnetctl_stats_handle.join() {
        Ok(r) => r.unwrap_or_default(),
        Err(e) => panic::resume_unwind(e),
    };

    lctl_record.append(&mut lnet_record);
    lctl_record.append(&mut mgs_fs_record);
    lctl_record.append(&mut recovery_status_records);
    lctl_record.append(&mut lnetctl_stats_record);

    Ok(lctl_record)
}

#[cfg(test)]
mod tests {
    use super::{parse_lctl_output, Record};

    #[test]
    fn ex8761_job_stats() {
        let xs = include_bytes!("./fixtures/valid/ex8761-lctl.txt");
        let expected = parse_lctl_output(xs).unwrap();

        let y = serde_json::to_string(&expected).unwrap();
        let z: Vec<Record> = serde_json::from_str(&y).unwrap();

        assert_eq!(expected, z);
    }

    #[test]
    fn es_6_2_0_job_stats_unhealthy() {
        let xs = include_bytes!("./fixtures/valid/params-6.2.0-r9.txt");
        let expected = parse_lctl_output(xs).unwrap();

        let y = serde_json::to_string(&expected).unwrap();
        let z: Vec<Record> = serde_json::from_str(&y).unwrap();

        assert_eq!(expected, z);
    }

    #[test]
    fn params() {
        let xs = super::parser::params();

        insta::assert_snapshot!(xs.join(" "));
    }
}
