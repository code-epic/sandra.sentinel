use std::fs::File;
use std::io::{BufWriter, Write};

use crate::error::Result;
use crate::types::ConciliationResult;

pub fn init_correctos(path: &str, headers_line: &str) -> Result<BufWriter<File>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "{}", headers_line)?;
    Ok(writer)
}

pub fn write_correcto(writer: &mut BufWriter<File>, result: &ConciliationResult) -> Result<()> {
    if let Some(ref line) = result.csv_line {
        writeln!(writer, "{}", line)?;
    }
    Ok(())
}
