//! # filebuffer
//! A wrapper around memmap for fast file reading, with no_std support.


#[cfg(feature = "std")]
use std::{slice, io, fs, os::fd::AsRawFd, path};
#[cfg(not(feature = "std"))]
use no_std_io::io;
#[cfg(not(feature = "std"))]
use crate::slice;
use core::mem::transmute;
use core::ops::Deref;
use core::ptr;
use libc;

#[allow(dead_code)]
pub struct FileBuffer {
    page_size: usize,
    buffer: *const u8,
    length: usize,
}
impl FileBuffer {
    #[cfg(feature = "std")]
    pub fn open<P: AsRef<path::Path>>(filename: P) -> io::Result<FileBuffer> {
        let mut open_opts = fs::OpenOptions::new();
        open_opts.read(true);
        let file = open_opts.open(filename)?;
        let fd = file.as_raw_fd();
        FileBuffer::from_filedes(fd)
    }
    pub fn from_filedes(fd: libc::c_int) -> io::Result<FileBuffer> {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
        let length = unsafe {
            let stat=[0u8;size_of::<libc::stat>()];
            let mut stat=transmute(stat);
            if libc::fstat(fd, &mut stat) == 0 {
                Ok(stat.st_size as usize)
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "Unable to access CDB file"))
            }
        }?;
        if length == 0 {
            return Ok(FileBuffer{length: 0, buffer: ptr::null(), page_size});
        }
        let buffer = unsafe {
            let p = libc::mmap(
                ptr::null_mut(),
                length as usize,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                fd,
                0
            );
            if p == libc::MAP_FAILED {
                Err(io::Error::new(io::ErrorKind::InvalidData, "Unable to map CDB file"))
            } else {
                Ok(p as *const u8)
            }
        }?;
        Ok(FileBuffer{length, buffer, page_size})
    }
    pub fn len(&self) -> usize {
        self.length
    }
}
impl Drop for FileBuffer {
    fn drop(&mut self) {
        if self.buffer != ptr::null() {
            unsafe {
                if libc::munmap(self.buffer as *mut libc::c_void, self.length) == libc::MAP_FAILED as libc::c_int {
                    panic!("Unable to munmap");
                }
            }
        }
    }
}

impl Deref for FileBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        if self.buffer == ptr::null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.buffer, self.length) }
        }
    }
}

impl AsRef<[u8]> for FileBuffer {
    fn as_ref(&self) -> &[u8] {
        self.deref()
    }
}
