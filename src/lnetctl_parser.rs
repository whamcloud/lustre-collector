// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    types::{
        lnet_exports::{LNetExport, Net},
        LNetStat, LNetStats, Param, Record,
    },
    LustreCollectorError,
};
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
                    value: y.statistics.drop_count,
                }),
            ]
        })
        .map(Record::LNetStat)
        .collect()
}

pub fn parse(x: &str) -> Result<Vec<Record>, LustreCollectorError> {
    let y: LNetExport = serde_yaml::from_str(x)?;

    Ok(y.net
        .map(|x| x.iter().flat_map(build_lnet_stats).collect())
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_lnet_down() {
        let x = parse(
            r#"show:
    - net:
          errno: -100
          descr: "cannot get networks: Network is down"
show:
    - route:
          errno: -100
          descr: "cannot get routes: Network is down"
show:
    - routing:
          errno: -100
          descr: "cannot get routing information: Network is down"
show:
    - peer:
          errno: -100
          descr: "cannot get peer list: Network is down"
show:
    - global:
          errno: -100
          descr: "cannot get numa_range: Unknown error -100"
global:
    max_intf: 200
    discovery: 1
    drop_asym_route: 0"#,
        )
        .unwrap();

        assert_debug_snapshot!(x);
    }

    #[test]
    fn test_lnet_export_parse() {
        let x = parse(
            r#"net:
    - net type: lo
      local NI(s):
        - nid: 0@lo
          status: up
          statistics:
              send_count: 942
              recv_count: 942
              drop_count: 0
          sent_stats:
              put: 942
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 930
              get: 0
              reply: 0
              ack: 12
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 942
              interrupts: 0
              dropped: 0
              aborted: 0
              no route: 0
              timeouts: 0
              error: 0
          tunables:
              peer_timeout: 0
              peer_credits: 0
              peer_buffer_credits: 0
              credits: 0
          dev cpt: 0
          tcp bonding: 0
          CPT: "[0]"
    - net type: tcp
      local NI(s):
        - nid: 10.73.20.11@tcp
          status: up
          interfaces:
              0: eth1
          statistics:
              send_count: 3825
              recv_count: 3736
              drop_count: 30
          sent_stats:
              put: 3821
              get: 4
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 3698
              get: 1
              reply: 3
              ack: 34
              hello: 0
          dropped_stats:
              put: 30
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 1000
              interrupts: 0
              dropped: 0
              aborted: 0
              no route: 0
              timeouts: 0
              error: 0
          tunables:
              peer_timeout: 180
              peer_credits: 8
              peer_buffer_credits: 0
              credits: 256
          dev cpt: -1
          tcp bonding: 0
          CPT: "[0]"
peer:
    - primary nid: 0@lo
      Multi-Rail: False
      peer ni:
        - nid: 0@lo
          state: NA
          max_ni_tx_credits: 0
          available_tx_credits: 0
          min_tx_credits: 0
          tx_q_num_of_buf: 0
          available_rtr_credits: 0
          min_rtr_credits: 0
          refcount: 1
          statistics:
              send_count: 0
              recv_count: 942
              drop_count: 0
          sent_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 930
              get: 0
              reply: 0
              ack: 12
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 1000
              dropped: 0
              timeout: 0
              error: 0
              network timeout: 0
    - primary nid: 10.73.20.12@tcp
      Multi-Rail: True
      peer ni:
        - nid: 10.73.20.12@tcp
          state: NA
          max_ni_tx_credits: 8
          available_tx_credits: 8
          min_tx_credits: 5
          tx_q_num_of_buf: 0
          available_rtr_credits: 8
          min_rtr_credits: 8
          refcount: 1
          statistics:
              send_count: 1628
              recv_count: 1628
              drop_count: 0
          sent_stats:
              put: 1626
              get: 2
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 1596
              get: 1
              reply: 1
              ack: 30
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 1000
              dropped: 3
              timeout: 0
              error: 7
              network timeout: 0
    - primary nid: 10.73.20.21@tcp
      Multi-Rail: True
      peer ni:
        - nid: 10.73.20.21@tcp
          state: NA
          max_ni_tx_credits: 8
          available_tx_credits: 8
          min_tx_credits: 1
          tx_q_num_of_buf: 0
          available_rtr_credits: 8
          min_rtr_credits: 8
          refcount: 1
          statistics:
              send_count: 1226
              recv_count: 1201
              drop_count: 0
          sent_stats:
              put: 1225
              get: 1
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 1198
              get: 0
              reply: 1
              ack: 2
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 1000
              dropped: 8
              timeout: 0
              error: 30
              network timeout: 0
    - primary nid: 10.73.20.22@tcp
      Multi-Rail: True
      peer ni:
        - nid: 10.73.20.22@tcp
          state: NA
          max_ni_tx_credits: 8
          available_tx_credits: 8
          min_tx_credits: 2
          tx_q_num_of_buf: 0
          available_rtr_credits: 8
          min_rtr_credits: 8
          refcount: 1
          statistics:
              send_count: 971
              recv_count: 907
              drop_count: 0
          sent_stats:
              put: 970
              get: 1
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 904
              get: 0
              reply: 1
              ack: 2
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 1000
              dropped: 8
              timeout: 0
              error: 28
              network timeout: 0
global:
    numa_range: 0
    max_intf: 200
    discovery: 1
    drop_asym_route: 0"#,
        )
        .unwrap();

        assert_debug_snapshot!(x);
    }
}
