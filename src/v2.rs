// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Redshirt 2 utilities.

use crate::{cursor::Cursor, error::Error};
use ring::digest::{Context, SHA1_FOR_LEGACY_USE_ONLY as SHA1, SHA1_OUTPUT_LEN};
use std::{
    fmt::{self, Debug, Formatter},
    io::{self, Read, Seek, SeekFrom},
    mem,
};

const MARKER: [u8; MARKER_LEN] = *b"REDSHRT2\x00";
const MARKER_LEN: usize = 9;
const HEADER_LEN: usize = MARKER_LEN + SHA1_OUTPUT_LEN;

#[derive(Debug)]
/// Reads Redshirt 2-protected data from an input stream.
pub struct Reader<R>(Cursor<R>);

#[derive(Clone)]
struct ChecksumBuilder(Context);

impl<R: Read + Seek> Reader<R> {
    #[inline]
    /// Creates a new reader from an input stream.
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
    use super::Reader;
    use std::io::{Cursor, Read, Seek, SeekFrom};

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
}
