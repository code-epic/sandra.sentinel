use std::fs::File;
use std::io::{BufWriter, Write};

use crate::error::Result;
use crate::types::{es_campo_fecha, normalizar_fecha};

/// CSV con registros existentes en CSV pero no en gRPC (nuevos beneficiarios).
/// Incluye TODOS los campos del mapping + cedula.
pub struct NuevosWriter {
    writer: BufWriter<File>,
    delimiter: char,
    field_names: Vec<String>,
}

impl NuevosWriter {
    pub fn new(path: &str, field_names: Vec<String>, delimiter: char) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let mut headers = vec!["cedula".to_string()];
        headers.extend_from_slice(&field_names);
        writeln!(writer, "{}", headers.join(&delimiter.to_string()))?;

        Ok(NuevosWriter {
            writer,
            delimiter,
            field_names,
        })
    }

    pub fn write_record(&mut self, cedula: &str, field_values: &[String]) -> Result<()> {
        let mut parts: Vec<String> = vec![cedula.to_string()];

        for (i, val) in field_values.iter().enumerate() {
            let field_name = self.field_names.get(i).map(|s| s.as_str()).unwrap_or("");
            let formatted = if es_campo_fecha(field_name) {
                normalizar_fecha(val)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Regresión: `fecha_ingreso` también se trunca a `YYYY-MM-DD`.
    #[test]
    fn test_nuevos_trunca_fecha_ingreso() {
        let path = std::env::temp_dir().join("reconciler_test_nuevos_fecha_ingreso.csv");
        let path = path.to_str().unwrap();
        let mut writer = NuevosWriter::new(
            path,
            vec!["grado".to_string(), "fecha_ingreso".to_string()],
            ';',
        )
        .unwrap();
        writer
            .write_record(
                "11439805",
                &["30".to_string(), "1996-07-05T00:00:00Z".to_string()],
            )
            .unwrap();
        writer.flush().unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("11439805;30;1996-07-05"));
        assert!(!content.contains("T00:00:00Z"));
        let _ = std::fs::remove_file(path);
    }
}
