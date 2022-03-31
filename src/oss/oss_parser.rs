// Copyright (c) 2021 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use crate::{
    oss::{ldlm_parser, obdfilter_parser},
    types::Record,
};
use combine::{choice, error::ParseError, Parser, Stream};

pub(crate) fn params() -> Vec<String> {
    let mut a = obdfilter_parser::obd_params();
    a.extend(ldlm_parser::ldlm_params());

    a
}

pub(crate) fn parse<I>() -> impl Parser<I, Output = Record>
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
obdfilter.fs-OST0000.num_exports=2
obdfilter.fs-OST0000.tot_dirty=0
obdfilter.fs-OST0000.tot_granted=8666816
obdfilter.fs-OST0000.tot_pending=0
"#;

        let result: (Vec<_>, _) = many(parse()).parse(x).unwrap();

        assert_debug_snapshot!(result)
    }
}
