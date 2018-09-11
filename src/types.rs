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
    pub param: Param,
    pub value: T,
}

#[derive(PartialEq, Debug, Serialize)]
pub enum TargetVariant {
    OST,
    MGT,
}

#[derive(PartialEq, Debug, Serialize)]
/// Stats specific to a target.
pub struct TargetStat<T> {
    pub kind: TargetVariant,
    pub param: Param,
    pub target: Target,
    pub value: T,
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
#[serde(untagged)]
pub enum HostStats {
    MemusedMax(HostStat<u64>),
    Memused(HostStat<u64>),
    LNetMemUsed(HostStat<u64>),
    HealthCheck(HostStat<String>),
}

/// The target stats currently collected
#[derive(PartialEq, Debug, Serialize)]
#[serde(untagged)]
pub enum TargetStats {
    /// Operations per OST. Read and write data is particularly interesting
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
}

#[derive(PartialEq, Debug, Serialize)]
#[serde(untagged)]
pub enum Record {
    Host(HostStats),
    Target(TargetStats),
}
