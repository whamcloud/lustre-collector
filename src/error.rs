// Copyright (c) 2020 DDN. All rights reserved.
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file.

use combine::error::StringStreamError;
use std::{io, str};

#[derive(Debug)]
pub enum LustreCollectorError {
    IoError(io::Error),
    SerdeJsonError(serde_json::error::Error),
    SerdeYamlError(serde_yaml::Error),
    StringStreamError(StringStreamError),
    Utf8Error(str::Utf8Error),
}

impl std::fmt::Display for LustreCollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            LustreCollectorError::IoError(ref err) => write!(f, "{}", err),
            LustreCollectorError::SerdeJsonError(ref err) => write!(f, "{}", err),
            LustreCollectorError::SerdeYamlError(ref err) => write!(f, "{}", err),
            LustreCollectorError::StringStreamError(ref err) => write!(f, "{}", err),
            LustreCollectorError::Utf8Error(ref err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for LustreCollectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            LustreCollectorError::IoError(ref err) => Some(err),
            LustreCollectorError::SerdeJsonError(ref err) => Some(err),
            LustreCollectorError::SerdeYamlError(ref err) => Some(err),
            LustreCollectorError::StringStreamError(ref err) => Some(err),
            LustreCollectorError::Utf8Error(ref err) => Some(err),
        }
    }
}

impl From<serde_json::error::Error> for LustreCollectorError {
    fn from(err: serde_json::error::Error) -> Self {
        LustreCollectorError::SerdeJsonError(err)
    }
}

impl From<serde_yaml::Error> for LustreCollectorError {
    fn from(err: serde_yaml::Error) -> Self {
        LustreCollectorError::SerdeYamlError(err)
    }
}

impl From<io::Error> for LustreCollectorError {
    fn from(err: io::Error) -> Self {
        LustreCollectorError::IoError(err)
    }
}

impl From<str::Utf8Error> for LustreCollectorError {
    fn from(err: str::Utf8Error) -> Self {
        LustreCollectorError::Utf8Error(err)
    }
}

impl From<StringStreamError> for LustreCollectorError {
    fn from(err: StringStreamError) -> Self {
        LustreCollectorError::StringStreamError(err)
    }
}
