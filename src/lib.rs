// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Provides support for the Redshirt 1 and Redshirt 2 data encoding schemes.
//!
//! This crate provides utilities for reading and writing Redshirt 1- or Redshirt 2-encoded data.
//! The Redshirt encoding schemes are used in *Uplink*, a 2001 computer hacking simulation game
//! developed by Introversion Software.
//!
//! # Reading Redshirt data
//!
//! redshirt provides `v1::Reader` for reading Redshirt 1 streams, and `v2::Reader` for reading
//! Redshirt 2 streams:
//!
//! ```no_run
//! use redshirt::v1::Reader;
//! use std::{fs::OpenOptions, io::Read};
//!
//! fn main() {
//!     let file = OpenOptions::new().read(true).open("data.dat").unwrap();
//!     let mut reader = Reader::new(file).unwrap();
//!     let mut buffer = [u8::default(); 4];
//!     reader.read_exact(&mut buffer).unwrap();
//!     println!("{:#?}", buffer);
//! }
//! ```
//!
//! Both types offer the same features: support for reading and seeking via the standard `Read` and
//! `Seek` traits, and destructuring to the underlying reader via `Reader::into_inner`.
//!
//! # Writing Redshirt data
//!
//! redshirt provides `v1::Writer` for writing Redshirt 1 streams, and `v2::Writer` for writing
//! Redshirt 2 streams:
//!
//! ```no_run
//! use redshirt::v1::Writer;
//! use std::{fs::OpenOptions, io::Write};
//!
//! fn main() {
//!     let file = OpenOptions::new().write(true).open("data.dat").unwrap();
//!     let mut writer = Writer::new(file).unwrap();
//!     let data = b"foobar";
//!     writer.write_all(&data[..]).unwrap();
//! }
//! ```
//!
//! Both types support writing via the standard `Write` trait, and destructuring to the underlying
//! writer via `Writer::into_inner`. `v1::Writer` also supports seeking via the standard `Seek`
//! trait. (See below for why `v2::Writer` doesn't support seeking.)
//!
//! ## `v2::Writer` additional notes
//!
//! Redshirt 2 stores a [SHA-1] hash of the encoded data in the header. This means that using
//! `v2::Writer` (which writes Redshirt 2 data) has two implications:
//!
//! * Seeking isn't supported, because it's costly to implement; the data would need to be re-read,
//!   and possibly stored in heap memory, in order to generate a correct hash.
//! * Currently the SHA-1 hash is finalised and written into the header either when the `v2::Writer`
//!   is dropped, or when `v2::Writer::into_inner` is called.
//!   **The `drop` call will panic if an error occurs**, so it's highly recommended that you call
//!   `into_inner`, which returns a `Result<T, Error>` instead:
//!
//! ```no_run
//! use redshirt::v2::Writer;
//! use std::{fs::OpenOptions, io::Write};
//!
//! fn main() {
//!     let file = OpenOptions::new().write(true).open("User.usr").unwrap();
//!     let mut writer = Writer::new(file).unwrap();
//!     let data = b"foobar";
//!     writer.write_all(&data[..]).unwrap();
//!     let _ = writer.into_inner().unwrap(); // Triggers a panic if writing the checksum fails.
//! }
//! ```
//!
//! [SHA-1]: https://en.wikipedia.org/wiki/SHA-1

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
#[inline]
pub(crate) fn xor_bytes(bytes: &mut [u8]) {
    for n in bytes {
        *n ^= 0b1000_0000;
    }
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
