use memmap2::Mmap;
use std::fs::File;

use crate::error::Result;

pub fn mmap_file(path: &str) -> Result<Mmap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}
