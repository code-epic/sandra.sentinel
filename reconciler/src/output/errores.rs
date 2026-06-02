use std::fs::File;
use std::io::{BufWriter, Write};

use crate::error::Result;
use crate::types::ConciliationResult;

#[derive(serde::Serialize)]
struct RichErrorRecord {
    cedula: String,
    status: String,
    csv_line: Option<String>,
    diffs: Vec<RichDiff>,
    total_diffs: usize,
}

#[derive(serde::Serialize)]
struct RichDiff {
    campo: String,
    valor_csv: String,
    valor_grpc: String,
}

pub struct RichErrorWriter {
    writer: BufWriter<File>,
}

impl RichErrorWriter {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::create(path)?;
        Ok(RichErrorWriter {
            writer: BufWriter::new(file),
        })
    }

    pub fn write(&mut self, result: &ConciliationResult) -> Result<()> {
        let diffs: Vec<RichDiff> = result.diffs.iter().map(|d| RichDiff {
            campo: d.field_name.clone(),
            valor_csv: d.expected.clone(),
            valor_grpc: d.actual.clone(),
        }).collect();

        let record = RichErrorRecord {
            cedula: result.cedula.clone(),
            status: format!("{:?}", result.status),
            csv_line: result.csv_line.clone(),
            diffs,
            total_diffs: result.diffs.len(),
        };

        let line = serde_json::to_string(&record)?;
        writeln!(self.writer, "{}", line)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
