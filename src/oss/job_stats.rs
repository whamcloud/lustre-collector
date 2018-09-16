// Copyright (c) 2018 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::{
    error::{ParseError, StreamError},
    optional,
    parser::{
        char::{alpha_num, newline},
        repeat::take_until,
    },
    stream::{Stream, StreamErrorFor},
    try, Parser,
};

use serde_yaml;

use types::{JobStatOst, JobStatsOst};

/*
obdfilter.fs-OST0001.job_stats=job_stats:
obdfilter.fs-OST0003.job_stats=
job_stats:
- job_id:          cp.0
  snapshot_time:   1537063675
  read_bytes:      { samples:         256, unit: bytes, min: 4194304, max: 4194304, sum:      1073741824 }
  write_bytes:     { samples:         256, unit: bytes, min: 4194304, max: 4194304, sum:      1073741824 }
  getattr:         { samples:           0, unit:  reqs }
  setattr:         { samples:           0, unit:  reqs }
  punch:           { samples:           0, unit:  reqs }
  sync:            { samples:           0, unit:  reqs }
  destroy:         { samples:           0, unit:  reqs }
  create:          { samples:           0, unit:  reqs }
  statfs:          { samples:           0, unit:  reqs }
  get_info:        { samples:           0, unit:  reqs }
  set_info:        { samples:           0, unit:  reqs }
  quotactl:        { samples:           0, unit:  reqs }
*/

pub fn parse<I>() -> impl Parser<Input = I, Output = Option<Vec<JobStatOst>>>
where
    I: Stream<Item = char>,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        optional(newline()), // If Jobstats are present, the whole yaml blob will be on a newline
        take_until(try((newline(), alpha_num()))),
    )
        .skip(newline())
        .and_then(|(_, x): (_, String)| {
            serde_yaml::from_str(&x)
                .map(|x: JobStatsOst| x.job_stats)
                .map_err(StreamErrorFor::<I>::other)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_yaml_deserialize() {
        let x = r#"job_stats:
- job_id:          cp.0
  snapshot_time:   1537070542
  read_bytes:      { samples:         256, unit: bytes, min: 4194304, max: 4194304, sum:      1073741824 }
  write_bytes:     { samples:           0, unit: bytes, min:       0, max:       0, sum:               0 }
  getattr:         { samples:           0, unit:  reqs }
  setattr:         { samples:           0, unit:  reqs }
  punch:           { samples:           0, unit:  reqs }
  sync:            { samples:           0, unit:  reqs }
  destroy:         { samples:           0, unit:  reqs }
  create:          { samples:           0, unit:  reqs }
  statfs:          { samples:           0, unit:  reqs }
  get_info:        { samples:           0, unit:  reqs }
  set_info:        { samples:           0, unit:  reqs }
  quotactl:        { samples:           0, unit:  reqs }"#;

        assert_eq!(
            serde_yaml::from_str::<JobStatsOst>(x).unwrap(),
            JobStatsOst { job_stats: None }
        )
    }
}
