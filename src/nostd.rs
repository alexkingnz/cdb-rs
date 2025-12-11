use crate::{slice, String, ToString};
use core::error::Error as CoreError;
use core::fmt::Display as CoreDisplay;
use core::mem::transmute;
use core::ops::Deref;
use core::ptr;
use core::result::Result as CoreResult;
use libc;

#[derive(Debug)]
pub struct Error(String);

impl Error {
    pub fn new(s: String) -> Error {
        Error(s)
    }
}

impl CoreDisplay for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl CoreError for Error {}

pub type Result<T> = CoreResult<T, Error>;

#[allow(dead_code)]
pub struct FileBuffer {
    page_size: usize,
    buffer: *const u8,
    length: usize,
}
impl FileBuffer {
    pub fn from_filedes(fd: libc::c_int) -> Result<FileBuffer> {
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize };
        let length = unsafe {
            let stat=[0u8;size_of::<libc::stat>()];
            let mut stat=transmute(stat);
            if libc::fstat(fd, &mut stat) == 0 {
                Ok(stat.st_size as usize)
            } else {
                Err(Error::new("Unable to access CDB file".to_string()))
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
                Err(Error::new("Unable to map CDB file".to_string()))
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

