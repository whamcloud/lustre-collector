// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

#![allow(unknown_lints)]
#![warn(clippy)]

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate serde_derive;

extern crate combine;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use combine::stream::state::State;
use combine::Parser;
use std::str;

mod base_parsers;
mod oss;
mod snapshot_time;
mod stats;
mod stats_parser;

use oss::oss_parser;

use std::process::Command;

fn main() {
    // let host = env::var("HOSTNAME").expect("HOSTNAME env var must be supplied to lustre_collector");

    let output = Command::new("lctl")
        .arg("get_param")
        .args(oss_parser::params())
        .output()
        .expect("failed to get obdfilter stats");

    let stats = str::from_utf8(&output.stdout).unwrap();

    let parsed = oss_parser::parse().easy_parse(State::new(stats));

    println!("{}", stats);

    println!("{:#?}", &parsed);

    println!("{:?}", serde_json::to_string(&parsed.unwrap().0));

    // println!("{:?}", serde_yaml::to_string(&parsed));
}
