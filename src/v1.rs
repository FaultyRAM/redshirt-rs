// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Redshirt 1 utilities.

use crate::{cursor::Cursor, error::Error};
use std::io::{self, Read, Seek, SeekFrom};

const MARKER: [u8; MARKER_LEN] = *b"REDSHIRT\x00";
const MARKER_LEN: usize = 9;

#[derive(Debug)]
/// Reads Redshirt 1-protected data from an input stream.
pub struct Reader<R>(Cursor<R>);

impl<R: Read> Reader<R> {
    #[inline]
    /// Creates a new reader from an existing input stream.
    pub fn new(mut src: R) -> Result<Self, Error> {
        let mut marker_buf = array!(MARKER_LEN);
        src.read_exact(&mut marker_buf)
            .map_err(Error::Io)
            .and_then(|_| {
                if marker_buf == MARKER {
                    Ok(Self(Cursor::new(src)))
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

#[cfg(test)]
mod tests {
    use super::Reader;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    const MSG_DEC: &[u8] = b"Hello world!";
    const MSG_ENC: &[u8] = b"REDSHIRT\x00\xC8\xE5\xEC\xEC\xEF\xA0\xF7\xEF\xF2\xEC\xE4\xA1";
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
