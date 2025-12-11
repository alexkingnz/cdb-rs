//! This crate provides support for reading and writing
//! [CDB](https://cr.yp.to/cdb.html) files. A CDB is a "constant
//! database" that acts as an on-disk associative array mapping keys to
//! values, allowing multiple values for each key. It provides for fast
//! lookups and low overheads. A constant database has no provision for
//! updating, only rewriting from scratch.
//!
//! # Examples
//!
//! Reading a set of records:
//!
//! ```
//! #[cfg(feature = "std")]
//! let cdb = cdb::CDB::open("tests/test1.cdb").unwrap();
//! #[cfg(not(feature = "std"))]
//! let cdb = {
//!     use std::os::fd::IntoRawFd;
//!     use std::fs::File;
//!     // Note: you would normally get a file descriptor another way in a
//!     // no_std environment.  The next example below uses libc as an example.
//!     let file = File::open("tests/test1.cdb").unwrap();
//!     cdb::CDB::from_filedes(file.into_raw_fd()).unwrap()
//! };
//!
//! for result in cdb.find(b"one") {
//!     println!("{:?}", result.unwrap());
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
//!     let mut cdb = cdb::CDBWriter::create("temporary.cdb")?;
//!     cdb.add(b"one", b"Hello, ")?;
//!     cdb.add(b"one", b"world!\n")?;
//!     cdb.add(b"two", &[1, 2, 3, 4])?;
//!     cdb.finish()?;
//!     Ok(())
//! }
//! ```
//!
//! Creating a database (using no_std):
//!
//! ```
//! # #[cfg(feature = "std")]
//! # fn main() {}
//! # #[cfg(not(feature = "std"))]
//! fn main() {
//!     let mut v = Vec::new();
//!     let mut cdb = cdb::CDBMake::new(&mut v);
//!     cdb.add(b"one", b"Hello, ").unwrap();
//!     cdb.add(b"one", b"world!\n").unwrap();
//!     cdb.add(b"two", &[1, 2, 3, 4]).unwrap();
//!     cdb.finish().unwrap();
//! }
//! ```
//!
//! # References
//!
//!  * [D. J. Bernstein's original software](https://cr.yp.to/cdb.html)
//!  * [Constant Database (cdb) Internals](https://www.unixuser.org/~euske/doc/cdbinternals/index.html)
//!  * [Wikipedia](https://en.wikipedia.org/wiki/Cdb_(software))
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(target_family = "windows", not(feature = "std")))]
compile_error!("The Standard library must be enabled for Windows.");


mod hash;
#[cfg(not(feature = "std"))]
mod nostd;
mod reader;
mod uint32;
#[cfg_attr(not(feature = "std"), path = "writer_nostd.rs")]
mod writer;

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
