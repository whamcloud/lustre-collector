// Copyright (c) 2020 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

mod base_parsers;
pub mod error;
mod lnetctl_parser;
mod mds;
pub mod mgs;
mod node_stats_parsers;
mod oss;
pub mod parser;
mod snapshot_time;
mod stats_parser;
mod top_level_parser;
pub mod types;

pub use crate::error::LustreCollectorError;
use combine::parser::EasyParser;
pub use lnetctl_parser::parse as parse_lnetctl_output;
pub use node_stats_parsers::{parse_cpustats_output, parse_meminfo_output};
use std::{io, str};
pub use types::*;

pub fn parse_lctl_output(lctl_output: &[u8]) -> Result<Vec<Record>, LustreCollectorError> {
    let lctl_stats = str::from_utf8(lctl_output)?;

    let (lctl_record, state) = parser::parse()
        .easy_parse(lctl_stats)
        .map_err(|err| err.map_position(|p| p.translate_position(lctl_stats)))?;

    if state != "" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Content left in input buffer: {}", state),
        )
        .into());
    }

    Ok(lctl_record)
}
