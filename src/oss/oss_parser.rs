// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    oss::{ldlm_parser, obdfilter_parser},
    types::Record,
};
use combine::{choice, error::ParseError, Parser, Stream};

pub fn params() -> Vec<String> {
    let mut a = obdfilter_parser::obd_params();
    a.extend(ldlm_parser::ldlm_params());

    a
}

pub fn parse<I>() -> impl Parser<I, Output = Record>
where
    I: Stream<Token = char>,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    choice((obdfilter_parser::parse(), ldlm_parser::parse()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::many;
    use insta::assert_debug_snapshot;

    #[test]
    fn test_params() {
        let x = r#"obdfilter.fs-OST0000.stats=
snapshot_time             1535148988.363769785 secs.nsecs
write_bytes               9 samples [bytes] 98303 4194304 33554431
create                    4 samples [reqs]
statfs                    42297 samples [reqs]
get_info                  2 samples [reqs]
connect                   6 samples [reqs]
reconnect                 1 samples [reqs]
disconnect                4 samples [reqs]
statfs                    46806 samples [reqs]
preprw                    9 samples [reqs]
commitrw                  9 samples [reqs]
ping                      8229 samples [reqs]
obdfilter.fs-OST0000.brw_stats=
snapshot_time:         1535148988.364041639 (secs.nsecs)

                           read      |     write
pages per bulk r/w     rpcs  % cum % |  rpcs        % cum %
32:		         0   0   0   |    1  11  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    0   0  11
256:		         0   0   0   |    0   0  11
512:		         0   0   0   |    0   0  11
1K:		         0   0   0   |    8  88 100

                           read      |     write
discontiguous pages    rpcs  % cum % |  rpcs        % cum %
0:		         0   0   0   |    6  66  66
1:		         0   0   0   |    3  33 100

                           read      |     write
discontiguous blocks   rpcs  % cum % |  rpcs        % cum %
0:		         0   0   0   |    9 100 100

                           read      |     write
disk fragmented I/Os   ios   % cum % |  ios         % cum %
1:		         0   0   0   |    1  11  11
2:		         0   0   0   |    0   0  11
3:		         0   0   0   |    0   0  11
4:		         0   0   0   |    8  88 100

                           read      |     write
disk I/Os in flight    ios   % cum % |  ios         % cum %
1:		         0   0   0   |    2   6   6
2:		         0   0   0   |    1   3   9
3:		         0   0   0   |    1   3  12
4:		         0   0   0   |    1   3  15
5:		         0   0   0   |    3   9  24
6:		         0   0   0   |    3   9  33
7:		         0   0   0   |    3   9  42
8:		         0   0   0   |    3   9  51
9:		         0   0   0   |    1   3  54
10:		         0   0   0   |    1   3  57
11:		         0   0   0   |    1   3  60
12:		         0   0   0   |    1   3  63
13:		         0   0   0   |    2   6  69
14:		         0   0   0   |    2   6  75
15:		         0   0   0   |    2   6  81
16:		         0   0   0   |    2   6  87
17:		         0   0   0   |    1   3  90
18:		         0   0   0   |    1   3  93
19:		         0   0   0   |    1   3  96
20:		         0   0   0   |    1   3 100

                           read      |     write
I/O time (1/1000s)     ios   % cum % |  ios         % cum %
4:		         0   0   0   |    1  11  11
8:		         0   0   0   |    0   0  11
16:		         0   0   0   |    0   0  11
32:		         0   0   0   |    0   0  11
64:		         0   0   0   |    0   0  11
128:		         0   0   0   |    3  33  44
256:		         0   0   0   |    5  55 100

                           read      |     write
disk I/O size          ios   % cum % |  ios         % cum %
128K:		         0   0   0   |    1   3   3
256K:		         0   0   0   |    0   0   3
512K:		         0   0   0   |    0   0   3
1M:		         0   0   0   |   32  96 100
obdfilter.fs-OST0000.filesfree=327382
obdfilter.fs-OST0000.filestotal=327680
obdfilter.fs-OST0000.fstype=osd-ldiskfs
obdfilter.fs-OST0000.kbytesavail=4486468
obdfilter.fs-OST0000.kbytesfree=4764996
obdfilter.fs-OST0000.kbytestotal=4831716
obdfilter.fs-OST0000.num_exports=2
obdfilter.fs-OST0000.tot_dirty=0
obdfilter.fs-OST0000.tot_granted=8666816
obdfilter.fs-OST0000.tot_pending=0
"#;

        let result: (Vec<_>, _) = many(parse()).parse(x).unwrap();

        assert_debug_snapshot!(result)
    }
}
