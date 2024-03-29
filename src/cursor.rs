// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

use crate::xor_bytes;
use std::{
    convert::TryFrom,
    io::{self, Read, Seek, SeekFrom, Write},
    ops::Deref,
};

const BUFFER_LEN: usize = 16384;

#[derive(Debug)]
pub(crate) struct Cursor<T> {
    inner: T,
    base: Option<u64>,
    offset: u64,
}

pub(crate) struct Chunk {
    bytes: [u8; BUFFER_LEN],
    len: usize,
}

impl<T> Cursor<T> {
    #[inline]
    pub(crate) const fn new(inner: T) -> Self {
        Self {
            inner,
            base: None,
            offset: 0,
        }
    }

    #[cfg(feature = "redshirt2")]
    #[inline]
    pub(crate) const fn offset(&self) -> u64 {
        self.offset
    }

    #[cfg(feature = "redshirt2")]
    #[inline]
    pub(crate) fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    #[inline]
    pub(crate) fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Write> Cursor<T> {
    #[inline]
    pub(crate) fn write_chunk(&mut self, buf: &[u8]) -> io::Result<Chunk> {
        let mut temp = array!(BUFFER_LEN);
        if let Some(chunk) = buf.chunks(temp.len()).next() {
            let used = &mut temp[..chunk.len()];
            used.copy_from_slice(chunk);
            xor_bytes(used);
            self.inner.write(used).map(|len| {
                self.offset += u64::try_from(len).unwrap();
                Chunk::new(temp, len)
            })
        } else {
            Ok(Chunk::new(temp, 0))
        }
    }
}

impl<T: Read> Read for Cursor<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner.read(buf) {
            Ok(len) => {
                xor_bytes(&mut buf[..len]);
                self.offset += u64::try_from(len).unwrap();
                Ok(len)
            }
            Err(e) => Err(e),
        }
    }
}

impl<T: Seek> Seek for Cursor<T> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        #[inline]
        fn overflow_error() -> io::Error {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )
        }

        let base = if let Some(v) = self.base {
            v
        } else {
            let v = self.inner.seek(SeekFrom::Current(0))? - self.offset;
            self.base = Some(v);
            v
        };

        match pos {
            SeekFrom::Start(n) => n
                .checked_add(base)
                .ok_or_else(overflow_error)
                .and_then(|v| self.inner.seek(SeekFrom::Start(v))),
            SeekFrom::Current(n) => {
                let offset_big = i128::from(self.offset);
                let n_big = i128::from(n);
                let rel = offset_big + n_big;
                if rel >= 0 {
                    self.inner.seek(SeekFrom::Current(n))
                } else {
                    Err(overflow_error())
                }
            }
            SeekFrom::End(n) => self.inner.seek(SeekFrom::End(n)).and_then(|v| {
                if v >= base {
                    Ok(v)
                } else {
                    let _ = self.inner.seek(SeekFrom::Start(self.offset)).unwrap();
                    Err(overflow_error())
                }
            }),
        }
        .map(|v| {
            self.offset = v - base;
            self.offset
        })
    }
}

impl<T: Write> Write for Cursor<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_chunk(buf).map(|chunk| chunk.len())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Chunk {
    #[inline]
    pub(self) const fn new(bytes: [u8; BUFFER_LEN], len: usize) -> Self {
        Self { bytes, len }
    }
}

impl AsRef<[u8]> for Chunk {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl Deref for Chunk {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
