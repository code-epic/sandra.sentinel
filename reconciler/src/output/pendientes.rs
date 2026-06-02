use std::fs::File;
use std::io::{BufWriter, Write};

use crate::compare::mapping::extract_value_from_json;
use crate::error::Result;
use serde_json::Value;

/// CSV con registros gRPC que no existen en el CSV (pendientes de revisión).
/// Incluye datos básicos (nombre, apellidos, sexo) + TODOS los campos del mapping.
pub struct PendienteWriter {
    writer: BufWriter<File>,
    delimiter: char,
    field_names: Vec<String>,
}

impl PendienteWriter {
    pub fn new(path: &str, field_names: Vec<String>, delimiter: char) -> Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let mut headers = vec!["cedula".to_string(), "nombre".to_string(), "apellidos".to_string(), "sexo".to_string()];
        headers.extend_from_slice(&field_names);
        writeln!(writer, "{}", headers.join(&delimiter.to_string()))?;

        Ok(PendienteWriter { writer, delimiter, field_names })
    }

    pub fn write(&mut self, cedula: &str, grpc_record: &Value, field_values: &[String]) -> Result<()> {
        let nombre = extract_value_from_json(grpc_record, &["nombre".to_string()]).unwrap_or_default();
        let apellidos = extract_value_from_json(grpc_record, &["apellidos".to_string()]).unwrap_or_default();
        let sexo = extract_value_from_json(grpc_record, &["sexo".to_string()]).unwrap_or_default();

        let mut parts: Vec<String> = vec![
            cedula.to_string(),
            nombre.replace(',', ";"),
            apellidos.replace(',', ";"),
            sexo.replace(',', ";"),
        ];

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
