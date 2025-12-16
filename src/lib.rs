// Copyright 2025 Alex King
// SPDX-License-Identifier: LGPL-3.0-or-later
//
//! This crate provides support for reading and writing 32 bit
//! [CDB](https://cbd.cr.yp.to/) files. A CDB is a "constant
//! database" that acts as an on-disk associative array mapping keys to
//! values, allowing multiple values for each key. It provides for fast
//! lookups and low overheads. A constant database has no provision for
//! updating, only rewriting from scratch.
//!
//! This version is notable because it is usable in (some) `#![no_std]`
//! environments.  
//! 
//!
//! # Examples
//!
//! Reading a set of records:
//!
//! ```
//! # #[cfg(not(feature = "std"))]
//! # fn main() {}
//! # #[cfg(feature = "std")]
//! fn main() -> std::io::Result<()> {
//!     let cdb = tumu_cdb::CDB::open("tests/test1.cdb")?;
//!
//!     for result in cdb.find(b"one") {
//!         println!("{:?}", result);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Creating a database with safe atomic updating:
//!
//! ```no_run
//! # #[cfg(not(feature = "std"))]
//! # fn main() {}
//! # #[cfg(feature = "std")]
//! fn main() -> std::io::Result<()> {
//!     let mut cdb = tumu_cdb::CDBWriter::create("temporary.cdb")?;
//!     cdb.add(b"one", b"Hello, ")?;
//!     cdb.add(b"one", b"world!\n")?;
//!     cdb.add(b"two", &[1, 2, 3, 4])?;
//!     cdb.finish()?;
//!     Ok(())
//! }
//! ```
//!
//! Reading a set of records (no_std):
//!
//! ```
//! # #[cfg(feature = "std")]
//! # fn main() {}
//! # #[cfg(not(feature = "std"))]
//! fn main() {
//!     let cdb = {
//!         use std::os::fd::IntoRawFd;
//!         use std::fs::File;
//!         let file = File::open("tests/test1.cdb").unwrap();
//!         tumu_cdb::CDB::from_filedes(file.into_raw_fd()).unwrap()
//!     };
//!
//!     for result in cdb.find(b"one") {
//!         println!("{:?}", result);
//!     }
//! };
//! ```
//!
//! Creating a database (using no_std):
//!
//! ```
//! # #[cfg(feature = "std")]
//! # fn main() {}
//! # #[cfg(not(feature = "std"))]
//! fn main() {
//!     let mut f = tumu_cdb::vecbuf::VecBuf::new();
//!     let mut cdb = tumu_cdb::CDBMake::new(f).unwrap();
//!     cdb.add(b"one", b"Hello, ").unwrap();
//!     cdb.add(b"one", b"world!\n").unwrap();
//!     cdb.add(b"two", &[1, 2, 3, 4]).unwrap();
//!     let v: tumu_cdb::vecbuf::VecBuf = cdb.finish().unwrap();
//!     let _v: Vec<u8> = v.into_inner();
//!     // arrange to write v to a file
//! }
//! ```
//!
//! # References
//!
//!  * [D. J. Bernstein's original software](https://cdb.cr.yp.to/)
//!  * [Bruce Guenter's rust implementation](https://github.com/bruceg/cdb-rs).  This version was originally based on Bruce's public domain work.
//!  * [Constant Database (cdb) Internals](https://www.unixuser.org/~euske/doc/cdbinternals/index.html)
//!  * [Wikipedia](https://en.wikipedia.org/wiki/Cdb_(software))
#![cfg_attr(not(feature = "std"), no_std)]

mod hash;
pub mod filebuffer;
mod reader;
mod uint32;
mod writer;
pub mod vecbuf;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
pub use alloc::{vec, 
    vec::Vec as Vec,
    slice,
    string::String as String,
    string::ToString as ToString};

pub use crate::reader::{CDB, CDBIter, CDBKeyValueIter, CDBValueIter};
#[cfg(not(feature = "std"))]
pub use crate::writer::CDBMake;
#[cfg(feature = "std")]
pub use crate::writer::{CDBMake, CDBWriter};
