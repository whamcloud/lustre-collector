// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

#[derive(Debug, PartialEq, Serialize)]
/// The hostname cooresponding to these stats.
pub struct Host(pub String);

#[derive(Debug, PartialEq, Serialize)]
/// The Lustre target cooresponding to these stats.
pub struct Target(pub String);

#[derive(Debug, PartialEq, Serialize)]
/// The name of the stat.
pub struct Param(pub String);

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Stat {
    pub name: String,
    pub units: String,
    pub samples: u64,
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub sum: Option<u64>,
    pub sumsquare: Option<u64>,
}

#[derive(PartialEq, Debug, Serialize)]
/// A Stat specific to a host.
pub struct HostStat<T> {
    pub host: Option<Host>,
    pub param: Param,
    pub value: T,
}

#[derive(PartialEq, Debug, Serialize)]
/// Stats collected via obdfilter.
pub struct ObdFilterStat {
    pub host: Option<Host>,
    pub param: Param,
    pub target: Target,
    pub value: ObdFilterStats,
}

#[derive(PartialEq, Debug, Serialize)]
pub struct BrwStatsBucket {
    pub name: u64,
    pub read: u64,
    pub write: u64,
}

#[derive(PartialEq, Debug, Serialize)]
pub struct BrwStats {
    pub name: String,
    pub unit: String,
    pub buckets: Vec<BrwStatsBucket>,
}

#[derive(PartialEq, Debug, Serialize)]
pub enum HostStats {
    MemUsedMax(HostStat<u64>),
    MemUsed(HostStat<u64>),
    LNetMemUsed(HostStat<u64>),
    Health(HostStat<String>),
}

/// The obdfilter stats currently collected
#[derive(PartialEq, Debug, Serialize)]
pub enum ObdFilterStats {
    /// Operations per OST. Read and write data is particularly interesting
    Stats(Vec<Stat>),
    BrwStats(Vec<BrwStats>),
    /// Available inodes
    FilesFree(u64),
    /// Total inodes
    FilesTotal(u64),
    /// Type of target
    FsType(String),
    /// Available disk space
    BytesAvail(u64),
    /// Free disk space
    BytesFree(u64),
    /// Total disk space
    BytesTotal(u64),
    NumExports(u64),
    TotDirty(u64),
    TotGranted(u64),
    TotPending(u64),
}

#[derive(PartialEq, Debug, Serialize)]
pub enum Stats {
    HostStats(HostStats),
    TargetStats(ObdFilterStat),
}
