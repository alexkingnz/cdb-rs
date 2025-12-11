use crate::{vec, Vec, ToString};
use core::cmp::max;
use core::iter;
use crate::nostd::*;

use crate::hash::hash;
use crate::uint32;


#[derive(Clone, Copy, Debug)]
struct HashPos {
    hash: u32,
    pos: u32,
}

impl HashPos {
    fn pack(&self, buf: &mut [u8]) {
        uint32::pack2(buf, self.hash, self.pos);
    }
}

fn err_toobig<T>() -> Result<T> {
    Err(Error::new("File too big".to_string()))
}

/// Base interface for making a CDB file.
///
/// # Example
///
/// TODO
///
pub struct CDBMake<'a> {
    entries: Vec<Vec<HashPos>>,
    pos: u32,
    buffer: &'a mut Vec<u8>,
}

impl<'a> CDBMake<'a> {
    /// Create a new CDB maker.
    pub fn new(buffer: &mut Vec<u8>) -> CDBMake<'_> {
        let buf = [0; 2048];
        buffer.clear();
        buffer.extend_from_slice(&buf);
        CDBMake {
            entries: iter::repeat(vec![]).take(256).collect::<Vec<_>>(),
            pos: 2048,
            buffer,
        }
    }

    fn pos_plus(&mut self, len: u32) -> Result<()> {
        if self.pos + len < len {
            err_toobig()
        } else {
            self.pos += len;
            Ok(())
        }
    }

    fn add_end(&mut self, keylen: u32, datalen: u32, hash: u32) -> Result<()> {
        self.entries[(hash & 0xff) as usize].push(HashPos {
            hash: hash,
            pos: self.pos,
        });
        self.pos_plus(8)?;
        self.pos_plus(keylen)?;
        self.pos_plus(datalen)?;
        Ok(())
    }

    fn add_begin(&mut self, keylen: u32, datalen: u32) {
        let mut buf = [0; 8];
        uint32::pack2(&mut buf[0..8], keylen, datalen);
        self.buffer.extend_from_slice(&buf);
    }

    /// Add a record to the CDB file.
    pub fn add(&mut self, key: &[u8], data: &[u8]) -> Result<()> {
        if key.len() >= 0xffffffff || data.len() >= 0xffffffff {
            return Err(Error::new("Key or data too big".to_string()));
        }
        self.add_begin(key.len() as u32, data.len() as u32);
        self.buffer.extend_from_slice(key);
        self.buffer.extend_from_slice(data);
        self.add_end(key.len() as u32, data.len() as u32, hash(&key[..]))
    }

    /// Finish writing to the CDB file and flush its contents.
    pub fn finish(mut self) -> Result<()> {
        let mut buf = [0; 8];

        let maxsize = self.entries.iter().fold(1, |acc, e| max(acc, e.len() * 2));
        let count = self.entries.iter().fold(0, |acc, e| acc + e.len());
        if maxsize + count > (0xffffffff / 8) {
            return err_toobig();
        }

        let mut table = vec![HashPos { hash: 0, pos: 0 }; maxsize];

        let mut header = [0 as u8; 2048];
        for i in 0..256 {
            let len = self.entries[i].len() * 2;
            let j = i * 8;
            uint32::pack2(&mut header[j..j + 8], self.pos, len as u32);

            for e in self.entries[i].iter() {
                let mut wh = (e.hash as usize >> 8) % len;
                while table[wh].pos != 0 {
                    wh += 1;
                    if wh == len {
                        wh = 0;
                    }
                }
                table[wh] = *e;
            }

            for hp in table.iter_mut().take(len) {
                hp.pack(&mut buf);
                self.buffer.extend_from_slice(&buf);
                self.pos_plus(8)?;
                *hp = HashPos { hash: 0, pos: 0 };
            }
        }

        self.buffer[0..2048].copy_from_slice(&header);
        Ok(())
    }
}

