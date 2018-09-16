// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

#![allow(unknown_lints)]
#![warn(clippy)]
#![cfg_attr(feature = "cargo-clippy", allow(let_and_return))]
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

extern crate combine;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use clap::{App, Arg};

use combine::stream::state::State;
use combine::Parser;
use std::str;

mod base_parsers;
mod mgs;
mod oss;
mod parser;
mod snapshot_time;
mod stats_parser;
mod top_level_parser;
mod types;

use std::process::Command;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;

arg_enum!{
    #[derive(PartialEq, Debug)]
    enum Format {
        Json,
        Yaml
    }
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
        ).get_matches();

    let format = value_t!(matches, "format", Format).unwrap_or_else(|e| e.exit());

    let output = Command::new("lctl")
        .arg("get_param")
        .args(parser::params())
        .output()
        .expect("failed to get lctl stats");

    let stats = str::from_utf8(&output.stdout).expect("while converting stdout from utf8");

    let (record, state) = parser::parse()
        .easy_parse(State::new(stats))
        .expect("while parsing stats");

    if state.input != "" {
        eprintln!("Content left in input buffer: {}", state.input)
    }

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
