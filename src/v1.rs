// Copyright (c) 2019 FaultyRAM
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option. This file may not be copied,
// modified, or distributed except according to those terms.

//! Redshirt 1 utilities.
//!
//! This module provides `Reader` and `Writer` types for reading and writing Redshirt 1-encoded
//! data, respectively.

use crate::{cursor::Cursor, error::Error};
use std::io::{self, Read, Seek, SeekFrom, Write};

const MARKER: [u8; MARKER_LEN] = *b"REDSHIRT\x00";
const MARKER_LEN: usize = 9;

#[derive(Debug)]
/// Reads Redshirt 1-protected data from an input stream.
pub struct Reader<R>(Cursor<R>);

#[derive(Debug)]
/// Writes Redshirt 1-protected data to an output stream.
pub struct Writer<W>(Cursor<W>);

impl<R: Read> Reader<R> {
    #[inline]
    /// Creates a new reader from an existing input stream.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if an I/O error occurs, or the underlying reader produces an invalid
    /// Redshirt 1 header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v1::Reader;
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("data.dat").unwrap();
    /// let reader = Reader::new(file).unwrap();
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v1::Reader;
    /// use std::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("data.dat").unwrap();
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

impl<W: Write> Writer<W> {
    #[inline]
    /// Wraps an existing output stream and writes a valid Redshirt 1 header.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if writing the Redshirt 1 header fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v1::Writer;
    ///
    /// let mut data = [u8::default(); 10];
    /// let writer = Writer::new(&mut data[..]).unwrap();
    /// ```
    pub fn new(mut dst: W) -> Result<Self, Error> {
        dst.write_all(&MARKER)
            .map(|_| Self(Cursor::new(dst)))
            .map_err(Error::Io)
    }

    #[inline]
    /// Unwraps a `Writer`, returning its underlying writer.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use redshirt::v1::Writer;
    ///
    /// let mut data = [u8::default(); 10];
    /// let writer = Writer::new(&mut data[..]).unwrap();
    /// let inner = writer.into_inner();
    /// ```
    pub fn into_inner(self) -> W {
        self.0.into_inner()
    }
}

impl<W: Write> Write for Writer<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: Seek> Seek for Writer<W> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::{Reader, Writer, MARKER_LEN};
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};

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

    #[test]
    fn writer_write() {
        let mut buffer = array!(MARKER_LEN + MSG_LEN);
        let mut writer = Writer::new(Cursor::new(&mut buffer[..])).unwrap();
        let (left, right) = MSG_DEC.split_at(MSG_LEN / 2);
        assert_eq!(
            writer.seek(SeekFrom::Current(MSG_LEN_I64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        writer.write_all(right).unwrap();
        assert_eq!(writer.seek(SeekFrom::Current(-MSG_LEN_I64)).unwrap(), 0);
        writer.write_all(left).unwrap();
        assert_eq!(buffer, MSG_ENC);
    }

    #[test]
    fn writer_seek_start() {
        let mut buffer = array!(MARKER_LEN + MSG_LEN);
        let mut writer = Writer::new(Cursor::new(&mut buffer[..])).unwrap();
        assert_eq!(writer.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(
            writer.seek(SeekFrom::Start(MSG_LEN_U64)).unwrap(),
            MSG_LEN_U64
        );
        assert_eq!(writer.seek(SeekFrom::Start(0)).unwrap(), 0);
        assert_eq!(
            writer.seek(SeekFrom::Start(MSG_LEN_U64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        assert_eq!(writer.seek(SeekFrom::Start(0)).unwrap(), 0);
    }

    #[test]
    fn writer_seek_current() {
        let mut buffer = array!(MARKER_LEN + MSG_LEN);
        let mut writer = Writer::new(Cursor::new(&mut buffer[..])).unwrap();
        assert_eq!(writer.seek(SeekFrom::Current(0)).unwrap(), 0);
        assert_eq!(
            writer.seek(SeekFrom::Current(MSG_LEN_I64)).unwrap(),
            MSG_LEN_U64
        );
        assert_eq!(writer.seek(SeekFrom::Current(-MSG_LEN_I64)).unwrap(), 0);
        assert_eq!(
            writer.seek(SeekFrom::Current(MSG_LEN_I64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        assert_eq!(
            writer.seek(SeekFrom::Current(-(MSG_LEN_I64 / 2))).unwrap(),
            0
        );
    }

    #[test]
    fn writer_seek_end() {
        let mut buffer = array!(MARKER_LEN + MSG_LEN);
        let mut writer = Writer::new(Cursor::new(&mut buffer[..])).unwrap();
        assert_eq!(writer.seek(SeekFrom::End(-MSG_LEN_I64)).unwrap(), 0);
        assert_eq!(writer.seek(SeekFrom::End(0)).unwrap(), MSG_LEN_U64);
        assert_eq!(writer.seek(SeekFrom::End(-MSG_LEN_I64)).unwrap(), 0);
        assert_eq!(
            writer.seek(SeekFrom::End(-MSG_LEN_I64 / 2)).unwrap(),
            MSG_LEN_U64 / 2
        );
        assert_eq!(writer.seek(SeekFrom::End(-MSG_LEN_I64)).unwrap(), 0);
    }

    #[test]
    #[should_panic]
    fn writer_seek_positive_overflow() {
        let mut buffer = array!(MARKER_LEN + MSG_LEN);
        let mut writer = Reader::new(Cursor::new(&mut buffer[..])).unwrap();
        let _ = writer.seek(SeekFrom::Start(u64::max_value())).unwrap();
    }

    #[test]
    #[should_panic]
    fn writer_seek_negative_overflow() {
        let mut buffer = array!(MARKER_LEN + MSG_LEN);
        let mut writer = Reader::new(Cursor::new(&mut buffer[..])).unwrap();
        let _ = writer.seek(SeekFrom::Current(-1)).unwrap();
    }
}
