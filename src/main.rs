// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

#![warn(clippy::all)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::let_and_return))]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;

mod base_parsers;
mod lnetctl_parser;
mod mgs;
mod oss;
mod parser;
mod snapshot_time;
mod stats_parser;
mod top_level_parser;
mod types;

use crate::types::Record;
use clap::{App, Arg};
use combine::{stream::state::State, Parser};
use std::{process::Command, str, thread};

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}

use self::errors::*;

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum Format {
        Json,
        Yaml
    }
}

fn get_lctl_output() -> Result<Vec<u8>> {
    let r = Command::new("lctl")
        .arg("get_param")
        .args(parser::params())
        .output()
        .chain_err(|| "calling lctl get_param")?;

    Ok(r.stdout)
}

fn parse_lctl_output(lctl_output: &[u8]) -> Result<Vec<Record>> {
    let lctl_stats =
        str::from_utf8(lctl_output).chain_err(|| "converting lctl stdout from utf8")?;

    let (lctl_record, state) = parser::parse()
        .easy_parse(State::new(lctl_stats))
        .expect("while parsing stats");

    if state.input != "" {
        eprintln!("Content left in input buffer: {}", state.input)
    }

    Ok(lctl_record)
}

fn main() {
    let variants = &Format::variants()
        .iter()
        .map(|x| x.to_ascii_lowercase())
        .collect::<Vec<_>>();

    let matches = App::new("lustre_collector")
        .version("0.1.0")
        .author("IML Team")
        .about("Grabs various Lustre statistics for display in JSON or YAML")
        .arg(
            Arg::with_name("format")
                .short("f")
                .long("format")
                .possible_values(&variants.iter().map(|x| x.as_str()).collect::<Vec<_>>()[..])
                .default_value(&variants[0])
                .help("Sets the output formatting")
                .takes_value(true),
        )
        .get_matches();

    let format = value_t!(matches, "format", Format).unwrap_or_else(|e| e.exit());

    let handle = thread::spawn(move || -> Result<Vec<Record>> {
        let lctl_output = get_lctl_output()?;
        let lctl_record = parse_lctl_output(&lctl_output)?;

        Ok(lctl_record)
    });

    let lnetctl_output = Command::new("lnetctl")
        .arg("export")
        .output()
        .expect("failed to get lnetctl stats");

    let lnetctl_stats =
        str::from_utf8(&lnetctl_output.stdout).expect("while converting lnetctl stdout from utf8");

    let lnet_record = lnetctl_parser::parse(lnetctl_stats).expect("while parsing lnetctl stats");

    let lctl_record = handle.join().unwrap().unwrap();

    let mut record = vec![];
    record.extend(lctl_record);
    record.extend(lnet_record);

    let r = match format {
        Format::Json => serde_json::to_string(&record).chain_err(|| "serializing to JSON"),
        Format::Yaml => serde_yaml::to_string(&record).chain_err(|| "serializing to YAML"),
    };

    match r {
        Ok(x) => println!("{}", x),
        Err(ref e) => {
            println!("error: {}", e);

            for e in e.iter().skip(1) {
                println!("caused by: {}", e);
            }

            if let Some(backtrace) = e.backtrace() {
                println!("backtrace: {:?}", backtrace);
            }

            ::std::process::exit(1);
        }
    }
}
