// Copyright 2025 Alex King
// SPDX-License-Identifier: LGPL-3.0-or-later
//
//! # A `Vec<u8>` with file like Read and Write, with `#![no_std]` support.
//!
//! A wrapper around a `Vec<u8>` that allows file like behavior (e.g. Read,
//! Write and Seek traits) in a `#![no_std]` environment (using traits from the
//! `no_std_io` crate.)
//!
//! This is intended to be equivalent to an `io::Cursor<Vec<u8>>` from the
//! standard library.  If the `std` feature it on, it builds on standard
//! library types and errors, else it builds on `no_std_io::io` types and
//! errors.

#![allow(dead_code)]
/// `Vec<u8>` wrapper with `File`-like interface.
#[cfg(feature = "std")]
pub use vecbuf_std::VecBuf;
#[cfg(not(feature = "std"))]
pub use vecbuf_nostd::VecBuf;

#[cfg(not(feature = "std"))]
mod vecbuf_nostd {
    extern crate alloc;
    use alloc::vec::Vec;
    use core::cmp::min;
    use core::fmt::Debug;
    use no_std_io::io;

    #[derive(Debug)]
    pub struct VecBuf {
        inner: Vec<u8>,
        pos: usize,
    }
    impl VecBuf {
        /// Constructs a new, empty `VecBuf`
        pub const fn new() -> VecBuf {
            VecBuf { pos: 0, inner: Vec::new() }
        }
        /// Returns the wrapped `Vec<u8>`, consuming self.
        pub fn into_inner(self) -> Vec<u8> {
            self.inner
        }
        pub const fn get_ref(&self) -> &Vec<u8> {
            &self.inner
        }
        pub const fn get_mut(&mut self) -> &mut Vec<u8> {
            &mut self.inner
        }
        /// Returns the current position of this cursor.
        pub const fn position(&self) -> u64 {
            self.pos as u64
        }
    }
    impl io::Read for VecBuf {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
            let readsize = min(buf.len(), self.inner.len() - self.pos);
            buf.copy_from_slice(&self.inner[self.pos..self.pos+readsize]);
            self.pos += readsize;
            Ok(readsize)
        }
    }
    impl io::Write for VecBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let inlen = min(buf.len(), self.inner.len() - self.pos);
            self.inner[self.pos..self.pos+inlen].copy_from_slice(&buf[..inlen]);
            self.inner.extend(&buf[inlen..]);
            self.pos+=buf.len();
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }
    impl io::Seek for VecBuf {
        fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
            let (base_pos, offset) = match style {
                io::SeekFrom::Start(n) => { 
                    if n > usize::MAX as u64 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "seek to overflowing position"
                        ))
                    } else {
                        self.pos = n as usize;
                        return Ok(n);
                    }
                },
                io::SeekFrom::End(n) => (self.inner.len(), n),
                io::SeekFrom::Current(n) => (self.pos, n),
            };
            match base_pos.checked_add_signed(offset as isize) {
                Some(n) => {
                    self.pos = n;
                    Ok(self.pos as u64)
                }
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid seek to a negative or overflowing position",
                )),
            }
        }
    }
}

#[cfg(feature = "std")]
mod vecbuf_std {
    extern crate std;
    use std::io;
    use std::vec::Vec;
    use core::fmt::Debug;

    #[derive(Debug)]
    pub struct VecBuf(io::Cursor<Vec<u8>>);
    impl VecBuf {
        /// Constructs a new, empty `VecBuf`
        pub const fn new() -> VecBuf {
            VecBuf(io::Cursor::new(Vec::new()))
        }
        /// Returns the wrapped `Vec<u8>`, consuming self.
        pub fn into_inner(self) -> Vec<u8> {
            self.0.into_inner()
        }
        pub const fn get_ref(&self) -> &Vec<u8> {
            self.0.get_ref()
        }
        pub const fn get_mut(&mut self) -> &mut Vec<u8> {
            self.0.get_mut()
        }
        /// Returns the current position of this cursor.
        pub const fn position(&self) -> u64 {
            self.0.position()
        }
    }
    impl io::Read for VecBuf {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
            self.0.read(buf)
        }
    }
    impl io::Write for VecBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }
    impl io::Seek for VecBuf {
        fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
            self.0.seek(pos)
        }
    }
}
