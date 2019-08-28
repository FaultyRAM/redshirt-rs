// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Provides support for the Redshirt 1 and Redshirt 2 data encoding schemes.

#![deny(
    warnings,
    future_incompatible,
    rust_2018_idioms,
    rustdoc,
    unused,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_results,
    clippy::all,
    clippy::pedantic
)]

#[cfg(any(feature = "redshirt1", feature = "redshirt2"))]
macro_rules! array {
    ($len:expr) => {
        array!(_, $len)
    };
    ($tyname:ty, $len:expr) => {
        [<$tyname>::default(); $len]
    };
}

#[cfg(any(feature = "redshirt1", feature = "redshirt2"))]
mod cursor;
#[cfg(any(feature = "redshirt1", feature = "redshirt2"))]
mod error;
#[cfg(any(feature = "redshirt1", feature = "redshirt2"))]
pub use error::Error;
#[cfg(feature = "redshirt1")]
pub mod v1;
#[cfg(feature = "redshirt2")]
pub mod v2;
