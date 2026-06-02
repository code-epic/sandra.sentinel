use std::fs::File;
use std::io::{BufWriter, Write};

use crate::error::Result;

/// CSV con registros rechazados (PartialMatch).
/// Headers fijos: cedula + field_names del mapping.
/// Fechas se truncan automaticamente a YYYY-MM-DD.
pub struct RechazosWriter {
    writer: BufWriter<File>,
    delimiter: char,
    field_names: Vec<String>,
}

impl RechazosWriter {
    pub fn new(path: &str, field_names: Vec<String>, delimiter: char) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let mut headers = vec!["cedula".to_string()];
        headers.extend_from_slice(&field_names);
        writeln!(writer, "{}", headers.join(&delimiter.to_string()))?;

        Ok(RechazosWriter { writer, delimiter, field_names })
    }

    pub fn write_record(&mut self, cedula: &str, field_values: &[String]) -> Result<()> {
        let mut parts: Vec<String> = vec![cedula.to_string()];

        for (i, val) in field_values.iter().enumerate() {
            let field_name = self.field_names.get(i).map(|s| s.as_str()).unwrap_or("");
            let formatted = if field_name == "f_ingreso" || field_name == "f_ult_ascenso" {
                val.split('T').next().unwrap_or(val).to_string()
            } else {
                val.clone()
            };
            parts.push(formatted);
        }

        let line = parts.join(&self.delimiter.to_string());
        writeln!(self.writer, "{}", line)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
