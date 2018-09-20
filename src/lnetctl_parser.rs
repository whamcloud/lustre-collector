// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::types::{LNetExport, LNetStat, LNetStats, Net, Param, Record};
use serde_yaml;

pub fn build_lnet_stats(x: &Net) -> Vec<Record> {
    x.local_nis
        .iter()
        .flat_map(|y| {
            vec![
                LNetStats::SendCount(LNetStat {
                    nid: y.nid.to_string(),
                    param: Param("send_count".to_string()),
                    value: y.statistics.send_count,
                }),
                LNetStats::RecvCount(LNetStat {
                    nid: y.nid.to_string(),
                    param: Param("recv_count".to_string()),
                    value: y.statistics.recv_count,
                }),
                LNetStats::DropCount(LNetStat {
                    nid: y.nid.to_string(),
                    param: Param("drop_count".to_string()),
                    value: y.statistics.recv_count,
                }),
            ]
        })
        .map(Record::LNetStat)
        .collect()
}

pub fn parse(x: &str) -> Result<Vec<Record>, serde_yaml::Error> {
    let y: LNetExport = serde_yaml::from_str(x)?;

    Ok(y.net.iter().flat_map(build_lnet_stats).collect())
}
