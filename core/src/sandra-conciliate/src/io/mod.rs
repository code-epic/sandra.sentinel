use crate::error::{Result, ScfError};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

#[allow(dead_code)]
pub struct FileReader {
    delimiter: char,
    skip_header: bool,
}

impl FileReader {
    pub fn new(delimiter: char, skip_header: bool) -> Self {
        Self {
            delimiter,
            skip_header,
        }
    }

    pub fn read_lines<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<impl Iterator<Item = Result<String>> + '_> {
        let file = File::open(path.as_ref())
            .map_err(|_| ScfError::FileNotFound(path.as_ref().display().to_string()))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.skip_header {
            let _ = lines.next();
        }

        Ok(lines.map(|line| line.map_err(ScfError::from)))
    }

    pub fn read_all<P: AsRef<Path>>(&self, path: P) -> Result<Vec<String>> {
        let file = File::open(path.as_ref())
            .map_err(|_| ScfError::FileNotFound(path.as_ref().display().to_string()))?;

        let reader = BufReader::new(file);
        let lines: Vec<String> = if self.skip_header {
            reader.lines().skip(1).filter_map(|l| l.ok()).collect()
        } else {
            reader.lines().filter_map(|l| l.ok()).collect()
        };

        Ok(lines)
    }
}

#[allow(dead_code)]
pub struct FileWriter {
    delimiter: char,
}

impl FileWriter {
    pub fn new(delimiter: char) -> Self {
        Self { delimiter }
    }

    pub fn create<P: AsRef<Path>>(&self, path: P) -> Result<BufWriter<File>> {
        let file = File::create(path.as_ref())?;
        Ok(BufWriter::new(file))
    }
}

pub fn create_output_dir(path: &str) -> Result<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn count_lines<P: AsRef<Path>>(path: P) -> Result<usize> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let metadata = fs::metadata(path.as_ref())?;
    Ok(metadata.len())
}
