// Copyright 2025 Alex King
// SPDX-License-Identifier: LGPL-3.0-or-later
#[cfg(feature = "std")]
use std::{io, path};
#[cfg(not(feature = "std"))]
use no_std_io::io;
#[cfg(not(feature = "std"))]
use crate::{Vec, vec};
use crate::filebuffer::FileBuffer;
#[cfg(not(feature = "std"))]
use libc;

use crate::hash::hash;
use crate::uint32;

pub use io::Result;

/// Memory-mapped CDB reader.
///
/// # Example
///
/// ```
/// #[cfg(not(feature = "std"))]
/// let cdb = {
///     use libc;
///     let fd = unsafe { libc::open(c"tests/test1.cdb".as_ptr() as *const libc::c_char, libc::O_RDONLY) };
///     if fd == -1 {panic!("Unable to open file tests/test1.cdb")}
///     tumu_cdb::CDB::from_filedes(fd).unwrap()
/// };
/// #[cfg(feature = "std")]
/// let cdb = tumu_cdb::CDB::open("tests/test1.cdb").unwrap();
///
/// for result in cdb.find(b"one") {
///     println!("{:?}", result);
/// }
/// ```
pub struct CDB {
    file: FileBuffer,
    size: usize,
}

fn err_badfile<T>() -> Result<T> {
    Err(io::Error::new(io::ErrorKind::Other, "Invalid file format"))
}

impl CDB {
    /// Opens the named file and returns the CDB reader.
    ///
    /// # Examples
    ///
    /// ```
    /// let cdb = tumu_cdb::CDB::open("tests/test1.cdb").unwrap();
    /// ```
    #[cfg(feature = "std")]
    pub fn open<P: AsRef<path::Path>>(filename: P) -> Result<CDB> {
        let file = FileBuffer::open(&filename)?;
        if file.len() < 2048 + 8 + 8 || file.len() > 0xffffffff {
            return err_badfile();
        }
        let size = file.len();
        Ok(CDB { file, size })
    }
    #[cfg(not(feature = "std"))]
    pub fn from_filedes(fd: libc::c_int) -> Result<CDB> {
        let file = FileBuffer::from_filedes(fd)?;
        if file.len() < 2048 + 8 + 8 || file.len() > 0xffffffff {
            return err_badfile();
        }
        let size = file.len();
        Ok(CDB { file, size })
    }
    pub fn copy_from_slice(s: &[u8]) -> Result<CDB> {
        let file = FileBuffer::copy_from_slice(s)?;
        let size = s.len();
        Ok(CDB { file, size })
    }

    fn read(&self, len: usize, pos: u32) -> Option<&[u8]> {
        let pos = pos as usize;
        self.file.get(pos..pos + len)
    }

    fn hash_table(&self, khash: u32) -> (u32, u32, u32) {
        let x = ((khash as usize) & 0xff) << 3;
        let (hpos, hslots) = uint32::unpack2(&self.file[x..x + 8]);
        let kpos = if hslots > 0 {
            hpos + (((khash >> 8) % hslots) << 3)
        } else {
            0
        };
        (hpos, hslots, kpos)
    }

    /// Match if key is present at pos
    fn match_key(&self, key: &[u8], pos: u32) -> bool {
        let len = key.len();
        self.read(len, pos).map(|x| x == key).unwrap_or(false)

    }

    /// Find the first record with the named key.
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(not(feature = "std"))]
    /// let cdb = {
    ///     use libc;
    ///     let fd = unsafe { libc::open(c"tests/test1.cdb".as_ptr() as *const libc::c_char, libc::O_RDONLY) };
    ///     if fd == -1 {panic!("Unable to open file tests/test1.cdb")}
    ///     tumu_cdb::CDB::from_filedes(fd).unwrap()
    /// };
    /// #[cfg(feature = "std")]
    /// let cdb = tumu_cdb::CDB::open("tests/test1.cdb").unwrap();
    /// if let Some(record) = cdb.get(b"one") {
    ///     println!("{:?}", record);
    /// }
    /// ```
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.find(key).next()
    }

    /// Find all records with the named key. The returned iterator
    /// produces each value associated with the key.
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(not(feature = "std"))]
    /// let cdb = {
    ///     use libc;
    ///     let fd = unsafe { libc::open(c"tests/test1.cdb".as_ptr() as *const libc::c_char, libc::O_RDONLY) };
    ///     if fd == -1 {panic!("Unable to open file tests/test1.cdb")}
    ///     tumu_cdb::CDB::from_filedes(fd).unwrap()
    /// };
    /// #[cfg(feature = "std")]
    /// let cdb = tumu_cdb::CDB::open("tests/test1.cdb").unwrap();
    ///
    /// for result in cdb.find(b"one") {
    ///     println!("{:?}", result);
    /// }
    /// ```
    pub fn find(&self, key: &[u8]) -> CDBValueIter<'_> {
        CDBValueIter::find(self, key)
    }

    /// Iterate over all the `(key, value)` pairs in the database.
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(not(feature = "std"))]
    /// let cdb = {
    ///     use libc;
    ///     let fd = unsafe { libc::open(c"tests/test1.cdb".as_ptr() as *const libc::c_char, libc::O_RDONLY) };
    ///     if fd == -1 {panic!("Unable to open file tests/test1.cdb")}
    ///     tumu_cdb::CDB::from_filedes(fd).unwrap()
    /// };
    /// #[cfg(feature = "std")]
    /// let cdb = tumu_cdb::CDB::open("tests/test1.cdb").unwrap();
    /// for result in cdb.iter() {
    ///     let (key, value) = result.unwrap();
    ///     println!("{:?} => {:?}", key, value);
    /// }
    /// ````
    pub fn iter(&self) -> CDBKeyValueIter<'_> {
        CDBKeyValueIter::start(&self)
    }
}

/// Type alias for [`CDBValueiter`](struct.CDBValueIter.html)
pub type CDBIter<'a> = CDBValueIter<'a>;

/// Iterator over a set of records in the CDB with the same key.
///
/// See [`CDB::find`](struct.CDB.html#method.find)
pub struct CDBValueIter<'a> {
    cdb: &'a CDB,
    key: Vec<u8>,
    khash: u32,
    kloop: u32,
    kpos: u32,
    hpos: u32,
    hslots: u32,
}

impl<'a> CDBValueIter<'a> {
    fn find(cdb: &'a CDB, key: &[u8]) -> Self {
        let khash = hash(key);
        let (hpos, hslots, kpos) = cdb.hash_table(khash);

        CDBValueIter {
            cdb: cdb,
            key: key.into_iter().map(|x| *x).collect(),
            khash: khash,
            kloop: 0,
            kpos: kpos,
            hpos: hpos,
            hslots: hslots,
        }
    }

}

impl<'a> Iterator for CDBValueIter<'a> {
    type Item = &'a[u8];
    fn next(&mut self) -> Option<Self::Item> {
        while self.kloop < self.hslots {
            //let mut buf = [0 as u8; 8];
            //let kpos = self.kpos;
            //iter_try!(self.cdb.read(&mut buf, kpos));
            let Some(p) = self.cdb.read(8, self.kpos) else { return None };
            let (khash, pos) = uint32::unpack2(p);
            if pos == 0 {
                return None;
            }
            self.kloop += 1;
            self.kpos += 8;
            if self.kpos == self.hpos + (self.hslots << 3) {
                self.kpos = self.hpos;
            }
            if khash == self.khash {
                let Some(p) = self.cdb.read(8, pos) else { return None };
                let (klen, dlen) = uint32::unpack2(p);
                if klen as usize == self.key.len() {
                    if self.cdb.match_key(&self.key[..], pos + 8) {
                        let dpos = pos + 8 + self.key.len() as u32;
                        return self.cdb.read(dlen as usize, dpos);
                    }
                }
            }
        }
        None
    }
}

/// Iterator over all the records in the CDB.
///
/// See [`CDB::iter`](struct.CDB.html#method.iter)
pub struct CDBKeyValueIter<'a> {
    cdb: &'a CDB,
    pos: u32,
    data_end: u32,
}

impl<'a> CDBKeyValueIter<'a> {
    fn start(cdb: &'a CDB) -> Self {
        let data_end = uint32::unpack(&cdb.file[0..4]).min(cdb.size as u32);
        Self {
            cdb,
            pos: 2048,
            data_end,
        }
    }
}

impl<'a> Iterator for CDBKeyValueIter<'a> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 8 >= self.data_end {
            None
        } else {
            let (klen, dlen) =
                uint32::unpack2(&self.cdb.file[self.pos as usize..self.pos as usize + 8]);
            if self.pos + klen + dlen >= self.data_end {
                Some(err_badfile())
            } else {
                let kpos = (self.pos + 8) as usize;
                let dpos = kpos + klen as usize;
                let mut key = vec![0; klen as usize];
                let mut value = vec![0; dlen as usize];
                // Copied from CDB::read
                key.copy_from_slice(&self.cdb.file[kpos..kpos + klen as usize]);
                value.copy_from_slice(&self.cdb.file[dpos..dpos + dlen as usize]);
                self.pos += 8 + klen + dlen;
                Some(Ok((key, value)))
            }
        }
    }
}
