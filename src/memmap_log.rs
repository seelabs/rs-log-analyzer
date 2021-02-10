// Memory mapped log file

// The log file will be read-only memmapped.
// Lines in the log file may be iterated through with the `lines()` function.
// This returns a `str`, not a `String`. This `str` points directly into the memmapped file.

use memmap::MmapOptions;

use std::fs::File;
use std::slice;
use std::str;

pub struct MemmapLog {
    mmap: memmap::Mmap,
}

impl MemmapLog {
    pub fn new(path: &std::path::PathBuf) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(MemmapLog { mmap })
    }

    // TODO should probably mark this as unsafe
    pub fn as_str(&self) -> &str {
        let ptr: *const u8 = self.mmap.as_ptr();
        let len = self.mmap.len();

        // the log file is a multi-gigabyte files. Don't check for valid utf8
        // TODO: benchmark valid utf8 check
        let s = unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            str::from_utf8_unchecked(slice)
        };
        s
    }
}
