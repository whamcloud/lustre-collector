// Copyright (c) 2019 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use clap::{arg_enum, value_t, App, Arg};
use lustre_collector::{
    error::LustreCollectorError, mgs::mgs_fs_parser, parse_lctl_output, parse_lnetctl_output,
    parse_mgs_fs_output, parser, types::Record,
};
use std::{
    error::Error as _,
    process::{exit, Command},
    str, thread,
};

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum Format {
        Json,
        Yaml
    }
}

fn get_lctl_output() -> Result<Vec<u8>, LustreCollectorError> {
    let r = Command::new("lctl")
        .arg("get_param")
        .args(parser::params())
        .output()?;

    Ok(r.stdout)
}

fn get_lctl_mgs_fs_output() -> Result<Vec<u8>, LustreCollectorError> {
    let r = Command::new("lctl")
        .arg("get_param")
        .arg("-N")
        .args(mgs_fs_parser::params())
        .output()?;

    Ok(r.stdout)
}

fn main() {
    let variants = &Format::variants()
        .iter()
        .map(|x| x.to_ascii_lowercase())
        .collect::<Vec<_>>();

    let matches = App::new("lustre_collector")
        .version("0.2.14")
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

    let handle = thread::spawn(move || -> Result<Vec<Record>, LustreCollectorError> {
        let lctl_output = get_lctl_output()?;
        let lctl_record = parse_lctl_output(&lctl_output)?;

        Ok(lctl_record)
    });

    let mgs_fs_handle = thread::spawn(move || -> Result<Vec<Record>, LustreCollectorError> {
        let lctl_output = get_lctl_mgs_fs_output()?;
        let lctl_record = parse_mgs_fs_output(&lctl_output)?;

        Ok(lctl_record)
    });

    let lnetctl_output = Command::new("lnetctl")
        .arg("export")
        .output()
        .expect("failed to get lnetctl stats");

    let lnetctl_stats =
        str::from_utf8(&lnetctl_output.stdout).expect("while converting lnetctl stdout from utf8");

    let mut lnet_record = parse_lnetctl_output(lnetctl_stats).expect("while parsing lnetctl stats");

    let mut lctl_record = handle.join().unwrap().unwrap();

    let mut mgs_fs_record = mgs_fs_handle.join().unwrap().unwrap();

    lctl_record.append(&mut lnet_record);
    lctl_record.append(&mut mgs_fs_record);

    let r = match format {
        Format::Json => {
            serde_json::to_string(&lctl_record).map_err(LustreCollectorError::SerdeJsonError)
        }
        Format::Yaml => {
            serde_yaml::to_string(&lctl_record).map_err(LustreCollectorError::SerdeYamlError)
        }
    };

    match r {
        Ok(x) => println!("{}", x),
        Err(ref e) => {
            eprintln!("error: {}", e);

            let mut cause = e.source();

            while let Some(e) = cause {
                eprintln!("caused by: {}", e);

                cause = e.source();
            }

            exit(1);
        }
    }
}
