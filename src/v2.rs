// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Redshirt 2 utilities.
//!
//! This module provides `Reader` and `Writer` types for reading and writing Redshirt 2-encoded
//! data, respectively.

use crate::{cursor::Cursor, error::Error, xor_bytes};
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY as SHA1, SHA1_OUTPUT_LEN};
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Read, Seek, SeekFrom, Write},
    mem,
};

const MARKER: [u8; MARKER_LEN] = *b"REDSHRT2\x00";
const MARKER_LEN: usize = 9;
const HEADER_LEN: usize = MARKER_LEN + SHA1_OUTPUT_LEN;

#[derive(Debug)]
/// Reads Redshirt 2-protected data from an input stream.
pub struct Reader<R>(Cursor<R>);

/// Writes Redshirt 2-protected data to an output stream.
pub struct Writer<W: Seek + Write> {
    dst: Option<Cursor<W>>,
    checksum: ChecksumBuilder,
}

#[derive(Clone)]
struct ChecksumBuilder(Context);

impl<R: Read + Seek> Reader<R> {
    #[inline]
    /// Creates a new reader from an input stream.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if any of the following occurs:
    ///
    /// * An I/O error occurs;
    /// * The underlying reader produces an invalid Redshirt 2 header;
    /// * The SHA-1 hash in the header does not match that of the encoded data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v2::Reader;
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("User.usr").unwrap();
    /// let reader = Reader::new(file).unwrap();
    /// ```
    pub fn new(mut src: R) -> Result<Self, Error> {
        let mut header_buf = array!(HEADER_LEN);
        src.read_exact(&mut header_buf)
            .map_err(Error::Io)
            .and_then(|_| {
                if header_buf[..MARKER_LEN] == MARKER {
                    let mut buffer = array!(16384);
                    let mut checksum = ChecksumBuilder::new();
                    loop {
                        match src.read(&mut buffer) {
                            Ok(len) => {
                                if len == 0 {
                                    break;
                                } else {
                                    checksum.update(&buffer[..len]);
                                }
                            }
                            Err(e) => {
                                if e.kind() == io::ErrorKind::Interrupted {
                                    continue;
                                } else {
                                    return Err(Error::Io(e));
                                }
                            }
                        }
                    }
                    let digest_a = &header_buf[MARKER_LEN..];
                    let digest_b = checksum.finish();
                    if digest_a == digest_b {
                        src.seek(SeekFrom::Start(HEADER_LEN as u64))
                            .map(|_| Self(Cursor::new(src)))
                            .map_err(Error::Io)
                    } else {
                        Err(Error::BadChecksum)
                    }
                } else {
                    Err(Error::BadHeader)
                }
            })
    }

    #[inline]
    /// Unwraps a `Reader`, returning its underlying reader.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v2::Reader;
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("User.usr").unwrap();
    /// let reader = Reader::new(file).unwrap();
    /// let inner = reader.into_inner();
    /// ```
    pub fn into_inner(self) -> R {
        self.0.into_inner()
    }
}

impl<R: Read> Read for Reader<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<R: Seek> Seek for Reader<R> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

impl<W: Seek + Write> Writer<W> {
    #[inline]
    /// Wraps an existing output stream and writes a Redshirt 2 header that is valid, but contains
    /// an invalid SHA-1 hash.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if writing the header fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v2::Writer;
    /// use std::io::Cursor;
    ///
    /// let mut data = [u8::default(); 30];
    /// let writer = Writer::new(Cursor::new(&mut data[..])).unwrap();
    /// ```
    pub fn new(mut dst: W) -> Result<Self, Error> {
        let mut dummy_header = array!(HEADER_LEN);
        dummy_header[..MARKER_LEN].copy_from_slice(&MARKER);
        dst.write_all(&dummy_header)
            .map(|_| Self {
                dst: Some(Cursor::new(dst)),
                checksum: ChecksumBuilder::new(),
            })
            .map_err(Error::Io)
    }

    #[inline]
    /// Writes out the SHA-1 hash of all previously encoded data, then unwraps the `Writer`.
    ///
    /// If a `Writer` is dropped without calling this method, the SHA-1 hash is written out, but the
    /// destructor will panic if an error occurs. Calling this method ensures that any such errors
    /// are safely handled.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if writing the SHA-1 hash fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v2::Writer;
    /// use std::io::Cursor;
    ///
    /// let mut data = [u8::default(); 30];
    /// let writer = Writer::new(Cursor::new(&mut data[..])).unwrap();
    /// let inner = writer.into_inner().unwrap();
    pub fn into_inner(mut self) -> Result<W, Error> {
        self.write_digest().map(Option::unwrap)
    }

    #[inline]
    fn write_digest(&mut self) -> Result<Option<W>, Error> {
        if let Some(mut dst) = self.dst.take() {
            let offset = dst.offset();
            dst.inner_mut()
                .seek(SeekFrom::Start(MARKER_LEN as u64))
                .and_then(|_| {
                    let digest = self.checksum.clone().finish();
                    let res = dst.inner_mut().write_all(&digest);
                    let _ = dst.seek(SeekFrom::Start(offset)).unwrap();
                    res
                })
                .map(|_| Some(dst.into_inner()))
                .map_err(Error::Io)
        } else {
            Ok(None)
        }
    }
}

impl<W: Debug + Seek + Write> Debug for Writer<W> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let digest = self.checksum.clone().finish();
        f.debug_struct("Writer")
            .field("dst", &self.dst)
            .field("digest", &digest)
            .finish()
    }
}

impl<W: Seek + Write> Write for Writer<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = array!(16384);
        let used = &mut buffer[..buf.len()];
        used.copy_from_slice(buf);
        xor_bytes(used);
        let dst = self.dst.as_mut().unwrap();
        dst.write_direct(used).map(|len| {
            self.checksum.update(&used[..len]);
            len
        })
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.dst.as_mut().unwrap().flush()
    }
}

impl<W: Seek + Write> Drop for Writer<W> {
    #[inline]
    /// When a `Writer` is dropped, this causes the SHA-1 hash of all previously encoded data to be
    /// written into the header.
    ///
    /// In general, you should use `Writer::into_inner` instead of relying on implict `drop` calls.
    ///
    /// # Panics
    ///
    /// Panics if writing the SHA-1 hash fails for any reason. To catch these errors, use
    /// `Writer::into_inner` instead of relying on implicit `drop` calls.
    fn drop(&mut self) {
        let _ = self.write_digest().unwrap();
    }
}

impl ChecksumBuilder {
    pub(self) fn new() -> Self {
        Self(Context::new(&SHA1))
    }

    pub(self) fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub(self) fn finish(self) -> [u8; SHA1_OUTPUT_LEN] {
        let digest = self.0.finish();
        let mut out = array!(SHA1_OUTPUT_LEN);
        out.copy_from_slice(digest.as_ref());
        for chunk in out.chunks_exact_mut(mem::size_of::<u32>()) {
            chunk.reverse();
        }
        out
    }
}

impl Debug for ChecksumBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let digest = self.0.clone().finish();
        f.debug_tuple("Checksum").field(&digest).finish()
    }
}

impl Default for ChecksumBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Reader, Writer, HEADER_LEN};
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};

    const MSG_DEC: &[u8] = b"Hello world!";
    const MSG_ENC: &[u8] = b"REDSHRT2\x00\x34\x54\x26\x2B\x4A\xBF\x29\x1D\x0B\x8E\x60\xD9\xA1\x76\xE1\x14\x7D\xDF\x05\xD4\xC8\xE5\xEC\xEC\xEF\xA0\xF7\xEF\xF2\xEC\xE4\xA1";
    const MSG_LEN: usize = 12;
    const MSG_LEN_U64: u64 = MSG_LEN as u64;
    const MSG_LEN_I64: i64 = MSG_LEN as i64;

    #[test]
    fn reader_read() {
        let mut reader = Reader::new(Cursor::new(MSG_ENC)).unwrap();
        let mut buffer = array!(MSG_LEN);
        let (left, right) = buffer.split_at_mut(MSG_LEN / 2);
        assert_eq!(
            reader.seek(SeekFrom::Current(MSG_LEN_I64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        reader.read_exact(right).unwrap();
        assert_eq!(reader.seek(SeekFrom::Current(-MSG_LEN_I64)).unwrap(), 0);
        reader.read_exact(left).unwrap();
        assert_eq!(buffer, MSG_DEC);
    }

    #[test]
    fn reader_seek_start() {
        let mut reader = Reader::new(Cursor::new(MSG_ENC)).unwrap();
        assert_eq!(reader.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(
            reader.seek(SeekFrom::Start(MSG_LEN_U64)).unwrap(),
            MSG_LEN_U64
        );
        assert_eq!(reader.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(
            reader.seek(SeekFrom::Start(MSG_LEN_U64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        assert_eq!(reader.seek(SeekFrom::Start(0)).unwrap(), 0);
    }

    #[test]
    fn reader_seek_current() {
        let mut reader = Reader::new(Cursor::new(MSG_ENC)).unwrap();
        assert_eq!(reader.seek(SeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(
            reader.seek(SeekFrom::Current(MSG_LEN_I64)).unwrap(),
            MSG_LEN_U64
        );
        assert_eq!(reader.seek(SeekFrom::Current(-MSG_LEN_I64)).unwrap(), 0);
        assert_eq!(
            reader.seek(SeekFrom::Current(MSG_LEN_I64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        assert_eq!(
            reader.seek(SeekFrom::Current(-(MSG_LEN_I64 / 2))).unwrap(),
            0
        );
    }

    #[test]
    fn reader_seek_end() {
        let mut reader = Reader::new(Cursor::new(MSG_ENC)).unwrap();
        assert_eq!(reader.seek(SeekFrom::End(-MSG_LEN_I64)).unwrap(), 0);
        assert_eq!(reader.seek(SeekFrom::End(0)).unwrap(), MSG_LEN_U64);
        assert_eq!(reader.seek(SeekFrom::End(-MSG_LEN_I64)).unwrap(), 0);
        assert_eq!(
            reader.seek(SeekFrom::End(-MSG_LEN_I64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        assert_eq!(reader.seek(SeekFrom::End(-MSG_LEN_I64)).unwrap(), 0);
    }

    #[test]
    #[should_panic]
    fn reader_seek_positive_overflow() {
        let mut reader = Reader::new(Cursor::new(MSG_ENC)).unwrap();
        let _ = reader.seek(SeekFrom::Start(u64::max_value())).unwrap();
    }

    #[test]
    #[should_panic]
    fn reader_seek_negative_overflow() {
        let mut reader = Reader::new(Cursor::new(MSG_ENC)).unwrap();
        let _ = reader.seek(SeekFrom::Current(-1)).unwrap();
    }

    #[test]
    fn writer_write() {
        let mut buffer = array!(HEADER_LEN + MSG_LEN);
        {
            let mut writer = Writer::new(Cursor::new(&mut buffer[..])).unwrap();
            writer.write_all(MSG_DEC).unwrap();
            let _ = writer.into_inner().unwrap();
        }
        assert_eq!(&buffer[..], &MSG_ENC[..]);
    }
}
