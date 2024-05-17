// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use clap::{value_parser, Arg, ValueEnum};
use lustre_collector::{error::LustreCollectorError, parse, utils::CommandMode};
use std::{
    fmt,
    path::PathBuf,
    process::ExitCode,
    str::{self, FromStr},
};

#[derive(ValueEnum, PartialEq, Debug, Clone, Copy)]
enum Format {
    Json,
    Yaml,
}

impl FromStr for Format {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "json" => Ok(Format::Json),
            "yaml" => Ok(Format::Yaml),
            _ => Err(format!("Could not convert {s} to format type")),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");

            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), LustreCollectorError> {
    tracing_subscriber::fmt::init();

    let matches = clap::Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Whamcloud")
        .about("Grabs various Lustre statistics for display in JSON or YAML")
        .args(vec![
            Arg::new("format")
                .short('f')
                .long("format")
                .value_parser(value_parser!(Format))
                .default_value("json")
                .help("Sets the output formatting"),
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_parser(value_parser!(CommandMode))
                .default_value("none")
                .help("Record/Plays the command output for integration testing"),
        ])
        .get_matches();

    let format = matches
        .get_one::<Format>("format")
        .expect("Required argument `format` missing");

    let mode = matches
        .get_one::<CommandMode>("mode")
        .expect("Required argument `mode` missing");

    let lctl_record = parse(mode, &PathBuf::new())?;

    let x = match format {
        Format::Json => serde_json::to_string(&lctl_record)?,
        Format::Yaml => serde_yaml::to_string(&lctl_record)?,
    };

    println!("{x}");

    Ok(())
}
