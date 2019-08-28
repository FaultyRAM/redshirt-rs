// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use std::{
    error,
    fmt::{self, Display, Formatter},
    io,
};

#[derive(Debug)]
/// Represents errors that may occur when working with Redshirt-encoded data.
pub enum Error {
    /// An I/O error occurred.
    Io(io::Error),
    /// The Redshirt 1/Redshirt 2 header contains invalid data.
    BadHeader,
    /// The checksum specified in the Redshirt 2 header does not match the checksum of the encoded
    /// data.
    BadChecksum,
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(inner) => Display::fmt(inner, f),
            Error::BadHeader => f.write_str("bad header"),
            Error::BadChecksum => f.write_str("bad checksum"),
        }
    }
}

impl error::Error for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(inner) => Some(inner),
            _ => None,
        }
    }
}
