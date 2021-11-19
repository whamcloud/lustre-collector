// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    types::{lnet_exports::Net, LNetStat, LNetStats, Param, Record},
    LustreCollectorError,
};

#[derive(serde::Serialize, serde::Deserialize)]
struct LnetNetStats {
    net: Option<Vec<Net>>,
}

pub(crate) fn build_lnet_stats(x: &Net) -> Vec<Record> {
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
    let x = x.trim();

    if x.is_empty() {
        return Ok(vec![]);
    }

    let y: LnetNetStats = serde_yaml::from_str(x)?;

    Ok(y.net
        .map(|x| x.iter().flat_map(build_lnet_stats).collect())
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_empty_input() {
        let xs = parse(" ").unwrap();

        assert_eq!(xs, vec![]);
    }

    #[test]
    fn test_lnet_down() {
        let x = parse(
            r#"show:
    - net:
          errno: -100
          descr: "cannot get networks: Network is down"
"#,
        )
        .unwrap();

        assert_debug_snapshot!(x);
    }

    #[test]
    fn test_lnet_parse2() {
        let x = parse(
            r#"net:
    - net type: lo
      local NI(s):
        - nid: 0@lo
          status: up
          statistics:
              send_count: 0
              recv_count: 0
              drop_count: 0
          sent_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              fatal_error: 0
              health value: 0
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
          CPT: "[0,1,2,3,4]"
    - net type: o2ib
      local NI(s):
        - nid: 172.16.0.24@o2ib
          status: up
          interfaces:
              0: ib0
          statistics:
              send_count: 0
              recv_count: 0
              drop_count: 0
          sent_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              fatal_error: 0
              health value: 1000
              interrupts: 0
              dropped: 0
              aborted: 0
              no route: 0
              timeouts: 0
              error: 0
          tunables:
              peer_timeout: 180
              peer_credits: 32
              peer_buffer_credits: 0
              credits: 256
              peercredits_hiw: 16
              map_on_demand: 0
              concurrent_sends: 64
              fmr_pool_size: 512
              fmr_flush_trigger: 384
              fmr_cache: 1
              ntx: 512
              conns_per_peer: 1
          lnd tunables:
          dev cpt: -1
          CPT: "[0,1,2,3,4]"
        - nid: 172.16.0.28@o2ib
          status: up
          interfaces:
              0: ib1
          statistics:
              send_count: 0
              recv_count: 0
              drop_count: 0
          sent_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          dropped_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              fatal_error: 0
              health value: 1000
              interrupts: 0
              dropped: 0
              aborted: 0
              no route: 0
              timeouts: 0
              error: 0
          tunables:
              peer_timeout: 180
              peer_credits: 32
              peer_buffer_credits: 0
              credits: 256
              peercredits_hiw: 16
              map_on_demand: 0
              concurrent_sends: 64
              fmr_pool_size: 512
              fmr_flush_trigger: 384
              fmr_cache: 1
              ntx: 512
              conns_per_peer: 1
          lnd tunables:
          dev cpt: -1
          CPT: "[0,1,2,3,4]""#,
        )
        .unwrap();

        assert_debug_snapshot!(x);
    }

    #[test]
    fn test_lnet_net_parse() {
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
"#,
        )
        .unwrap();

        assert_debug_snapshot!(x);
    }

    #[test]
    fn test_lnet_export_parse_no_bonding() {
        let x = parse(
            r#"net:
    - net type: lo
      local NI(s):
        - nid: 0@lo
          status: up
          statistics:
              send_count: 9
              recv_count: 8
              drop_count: 1
          sent_stats:
              put: 9
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 8
              get: 0
              reply: 0
              ack: 0
              hello: 0
          dropped_stats:
              put: 1
              get: 0
              reply: 0
              ack: 0
              hello: 0
          health stats:
              health value: 800
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
          CPT: "[0,1,2,3,4]"
    - net type: tcp
      local NI(s):
        - nid: 10.36.4.130@tcp
          status: up
          statistics:
              send_count: 0
              recv_count: 0
              drop_count: 0
          sent_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          received_stats:
              put: 0
              get: 0
              reply: 0
              ack: 0
              hello: 0
          dropped_stats:
              put: 0
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
          CPT: "[0,1,2,3,4]"
"#,
        )
        .unwrap();

        assert_debug_snapshot!(x);
    }
}
