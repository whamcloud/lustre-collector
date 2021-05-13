// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::types::{JobStatOst, JobStatsOst};
use combine::{
    attempt,
    error::{ParseError, StreamError},
    optional,
    parser::{
        char::{alpha_num, newline},
        repeat::take_until,
    },
    stream::{Stream, StreamErrorFor},
    Parser,
};

pub fn parse<I>() -> impl Parser<I, Output = Option<Vec<JobStatOst>>>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    (
        optional(newline()), // If Jobstats are present, the whole yaml blob will be on a newline
        take_until(attempt((newline(), alpha_num()))),
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
    use crate::types::{BytesStat, ReqsStat};
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

        let expected = JobStatsOst {
            job_stats: Some(vec![JobStatOst {
                job_id: "cp.0".to_string(),
                snapshot_time: 1_537_070_542,
                read_bytes: BytesStat {
                    samples: 256,
                    unit: "bytes".to_string(),
                    min: 4_194_304,
                    max: 4_194_304,
                    sum: 1_073_741_824,
                },
                write_bytes: BytesStat {
                    samples: 0,
                    unit: "bytes".to_string(),
                    min: 0,
                    max: 0,
                    sum: 0,
                },
                getattr: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                setattr: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                punch: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                sync: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                destroy: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                create: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                statfs: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                get_info: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                set_info: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
                quotactl: ReqsStat {
                    samples: 0,
                    unit: "reqs".to_string(),
                },
            }]),
        };

        assert_eq!(serde_yaml::from_str::<JobStatsOst>(x).unwrap(), expected)
    }
}
