// Copyright (c) 2020 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use std::{fmt, ops::Deref};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// The hostname cooresponding to these stats.
pub struct Host(pub String);

impl Deref for Host {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// The Lustre target cooresponding to these stats.
pub struct Target(pub String);

impl Deref for Target {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
/// The name of the stat.
pub struct Param(pub String);

impl Deref for Param {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReqsStat {
    pub samples: i64,
    pub unit: String,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct BytesStat {
    pub samples: i64,
    pub unit: String,
    pub min: i64,
    pub max: i64,
    pub sum: i64,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct JobStatsOst {
    pub job_stats: Option<Vec<JobStatOst>>,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct JobStatOst {
    pub job_id: String,
    pub snapshot_time: i64,
    pub read_bytes: BytesStat,
    pub write_bytes: BytesStat,
    pub getattr: ReqsStat,
    pub setattr: ReqsStat,
    pub punch: ReqsStat,
    pub sync: ReqsStat,
    pub destroy: ReqsStat,
    pub create: ReqsStat,
    pub statfs: ReqsStat,
    pub get_info: ReqsStat,
    pub set_info: ReqsStat,
    pub quotactl: ReqsStat,
}

pub mod lnet_exports {
    use std::collections::HashMap;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct LocalNiS {
        pub nid: String,
        pub status: String,
        pub statistics: LNetStatistics,
        pub sent_stats: Stats,
        pub received_stats: Stats,
        pub dropped_stats: Stats,
        #[serde(rename = "health stats")]
        pub health_stats: HealthStats,
        pub tunables: Tunables,
        #[serde(rename = "dev cpt")]
        pub dev_cpt: i64,
        #[serde(rename = "tcp bonding")]
        pub tcp_bonding: i64,
        #[serde(rename = "CPT")]
        pub cpt: String,
        pub interfaces: Option<HashMap<i64, String>>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Stats {
        pub put: i64,
        pub get: i64,
        pub reply: i64,
        pub ack: i64,
        pub hello: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct HealthStats {
        #[serde(rename = "health value")]
        health_value: i64,
        interrupts: i64,
        dropped: i64,
        aborted: i64,
        #[serde(rename = "no route")]
        no_route: i64,
        timeouts: i64,
        error: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct HealthStatsPeer {
        #[serde(rename = "health value")]
        health_value: i64,
        dropped: i64,
        timeout: i64,
        error: i64,
        #[serde(rename = "network timeout")]
        network_timeout: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Net {
        #[serde(rename = "net type")]
        pub net_type: String,
        #[serde(rename = "local NI(s)")]
        pub local_nis: Vec<LocalNiS>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Global {
        numa_range: Option<i64>,
        max_intf: i64,
        discovery: i64,
        drop_asym_route: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Peer {
        #[serde(rename = "primary nid")]
        pub primary_nid: String,
        #[serde(rename = "Multi-Rail")]
        pub multi_rail: String,
        #[serde(rename = "peer ni")]
        pub peer_ni: Vec<PeerNi>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PeerNi {
        nid: String,
        state: String,
        max_ni_tx_credits: i64,
        available_tx_credits: i64,
        min_tx_credits: i64,
        tx_q_num_of_buf: i64,
        available_rtr_credits: i64,
        min_rtr_credits: i64,
        refcount: i64,
        statistics: LNetStatistics,
        sent_stats: Stats,
        received_stats: Stats,
        dropped_stats: Stats,
        #[serde(rename = "health stats")]
        health_stats: HealthStatsPeer,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct LNetExport {
        pub net: Option<Vec<Net>>,
        pub peer: Option<Vec<Peer>>,
        pub global: Option<Global>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct LNetStatistics {
        pub send_count: i64,
        pub recv_count: i64,
        pub drop_count: i64,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Tunables {
        pub peer_timeout: i64,
        pub peer_credits: i64,
        pub peer_buffer_credits: i64,
        pub credits: i64,
    }
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Stat {
    pub name: String,
    pub units: String,
    pub samples: u64,
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub sum: Option<u64>,
    pub sumsquare: Option<u64>,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
/// A Stat specific to a host.
pub struct HostStat<T> {
    pub param: Param,
    pub value: T,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum TargetVariant {
    OST,
    MGT,
    MDT,
}

impl fmt::Display for TargetVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TargetVariant::OST => write!(f, "OST"),
            TargetVariant::MGT => write!(f, "MGT"),
            TargetVariant::MDT => write!(f, "MDT"),
        }
    }
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
/// Stats specific to a target.
pub struct TargetStat<T> {
    pub kind: TargetVariant,
    pub param: Param,
    pub target: Target,
    pub value: T,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
/// Stats specific to a LNet Nid.
pub struct LNetStat<T> {
    pub nid: String,
    pub param: Param,
    pub value: T,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct BrwStatsBucket {
    pub name: u64,
    pub read: u64,
    pub write: u64,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct BrwStats {
    pub name: String,
    pub unit: String,
    pub buckets: Vec<BrwStatsBucket>,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum HostStats {
    MemusedMax(HostStat<u64>),
    Memused(HostStat<u64>),
    LNetMemUsed(HostStat<u64>),
    HealthCheck(HostStat<String>),
}

/// A Stat specific to a node.
#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeStat<T> {
    pub param: Param,
    pub value: T,
}
/// Top level node stats (not directly Lustre related)
#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum NodeStats {
    CpuUser(NodeStat<u64>),
    CpuSystem(NodeStat<u64>),
    CpuIowait(NodeStat<u64>),
    CpuTotal(NodeStat<u64>),
    MemTotal(NodeStat<u64>),
    MemFree(NodeStat<u64>),
    SwapTotal(NodeStat<u64>),
    SwapFree(NodeStat<u64>),
}

/// The target stats currently collected
#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum TargetStats {
    /// Operations per OST. Read and write data is particularly interesting
    JobStatsOst(TargetStat<Option<Vec<JobStatOst>>>),
    Stats(TargetStat<Vec<Stat>>),
    BrwStats(TargetStat<Vec<BrwStats>>),
    /// Available inodes
    FilesFree(TargetStat<u64>),
    /// Total inodes
    FilesTotal(TargetStat<u64>),
    /// Type of target
    FsType(TargetStat<String>),
    /// Available disk space
    BytesAvail(TargetStat<u64>),
    /// Free disk space
    BytesFree(TargetStat<u64>),
    /// Total disk space
    BytesTotal(TargetStat<u64>),
    NumExports(TargetStat<u64>),
    TotDirty(TargetStat<u64>),
    TotGranted(TargetStat<u64>),
    TotPending(TargetStat<u64>),
    ContendedLocks(TargetStat<u64>),
    ContentionSeconds(TargetStat<u64>),
    CtimeAgeLimit(TargetStat<u64>),
    EarlyLockCancel(TargetStat<u64>),
    LockCount(TargetStat<u64>),
    LockTimeouts(TargetStat<u64>),
    LockUnusedCount(TargetStat<u64>),
    LruMaxAge(TargetStat<u64>),
    LruSize(TargetStat<u64>),
    MaxNolockBytes(TargetStat<u64>),
    MaxParallelAst(TargetStat<u64>),
    ResourceCount(TargetStat<u64>),
    ThreadsMin(TargetStat<u64>),
    ThreadsMax(TargetStat<u64>),
    ThreadsStarted(TargetStat<u64>),
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum LNetStats {
    SendCount(LNetStat<i64>),
    RecvCount(LNetStat<i64>),
    DropCount(LNetStat<i64>),
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Record {
    Host(HostStats),
    LNetStat(LNetStats),
    Node(NodeStats),
    Target(TargetStats),
}
