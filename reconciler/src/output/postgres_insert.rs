use std::fs::File;
use std::io::Write;

use crate::error::Result;

/// Genera batch INSERT PostgreSQL para registros nuevos (existentes en CSV, no en gRPC).
pub struct PostgresInsertBuilder {
    field_names: Vec<String>,
    inserts: Vec<(String, Vec<String>)>,
}

impl PostgresInsertBuilder {
    pub fn new(field_names: Vec<String>) -> Self {
        PostgresInsertBuilder {
            field_names,
            inserts: vec![],
        }
    }

    pub fn add(&mut self, cedula: &str, field_values: Vec<String>) {
        self.inserts.push((cedula.to_string(), field_values));
    }

    pub fn write_to_file(&self, path: &str) -> Result<()> {
        if self.inserts.is_empty() {
            return Ok(());
        }

        let mut file = File::create(path)?;
        writeln!(&mut file, "-- Batch de inserciones generado por Sandra Reconciler")?;
        writeln!(&mut file, "-- Total registros nuevos: {}", self.inserts.len())?;
        writeln!(&mut file, "BEGIN;")?;
        writeln!(&mut file)?;

        let columns: Vec<String> = std::iter::once("cedula".to_string())
            .chain(self.field_names.iter().cloned())
            .collect();

        writeln!(&mut file, "INSERT INTO beneficiarios ({})", columns.join(", "))?;
        writeln!(&mut file, "VALUES")?;

        for (i, (cedula, values)) in self.inserts.iter().enumerate() {
            let mut row_parts: Vec<String> = vec![format!("'{}'", cedula.replace("'", "''"))];
            for (j, val) in values.iter().enumerate() {
                let field_name = self.field_names.get(j).map(|s| s.as_str()).unwrap_or("");
                row_parts.push(format_postgres_value(val, field_name));
            }
            let row = row_parts.join(", ");

            if i == self.inserts.len() - 1 {
                writeln!(&mut file, "    ({});", row)?;
            } else {
                writeln!(&mut file, "    ({}),", row)?;
            }
        }

        writeln!(&mut file)?;
        writeln!(&mut file, "COMMIT;")?;

        Ok(())
    }
}

fn format_postgres_value(val: &str, field_name: &str) -> String {
    if field_name == "f_ingreso" || field_name == "f_ult_ascenso" || field_name == "fecha_ingreso" {
        let truncated = val.split('T').next().unwrap_or(val);
        format!("'{}'", truncated.replace("'", "''"))
    } else {
        match field_name {
            "grado_id" | "n_hijos" | "anio_reconocido" | "mes_reconocido" | "dia_reconocido" => {
                if val.trim().is_empty() { "0".to_string() } else { val.to_string() }
            }
            _ => format!("'{}'", val.replace("'", "''")),
        }
    }
}
