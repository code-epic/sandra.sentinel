use std::fs::File;
use std::io::Write;

use crate::error::Result;

/// Batch UPDATE único en PostgreSQL que actualiza TODOS los campos
/// para cada cédula con diferencias, usando VALUES multi-fila.
/// Mucho más rápido que N UPDATEs individuales.
pub struct PostgresBatchBuilder {
    field_names: Vec<String>,
    updates: Vec<(String, Vec<String>)>, // (cedula, [val_campo1, val_campo2, ...])
}

impl PostgresBatchBuilder {
    pub fn new(field_names: Vec<String>) -> Self {
        PostgresBatchBuilder {
            field_names,
            updates: vec![],
        }
    }

    /// Agrega un registro con diferencias. Debe venir con TODOS los campos
    /// del mapping (campos que difieren + campos que coinciden).
    pub fn add(&mut self, cedula: &str, all_field_values: Vec<String>) {
        self.updates.push((cedula.to_string(), all_field_values));
    }

    pub fn write_to_file(&self, path: &str) -> Result<()> {
        if self.updates.is_empty() {
            return Ok(());
        }

        let mut file = File::create(path)?;
        writeln!(&mut file, "-- Batch de actualizaciones generado por Sandra Reconciler")?;
        writeln!(&mut file, "-- Total registros con diferencias: {}", self.updates.len())?;
        writeln!(&mut file, "BEGIN;")?;
        writeln!(&mut file)?;

        // Nombres de columnas separados por coma
        let _columns = self.field_names.join(", ");

        writeln!(&mut file, "UPDATE beneficiarios AS b SET")?;

        // Generar SET clause: columna = v.columna
        for (i, field) in self.field_names.iter().enumerate() {
            if i == self.field_names.len() - 1 {
                writeln!(&mut file, "    {} = v.{}", field, field)?;
            } else {
                writeln!(&mut file, "    {} = v.{},", field, field)?;
            }
        }

        writeln!(&mut file, "FROM (VALUES")?;

        for (i, (cedula, values)) in self.updates.iter().enumerate() {
            let mut row_parts: Vec<String> = vec![format!("'{}'", cedula.replace("'", "''"))];
            for (j, val) in values.iter().enumerate() {
                let field_name = &self.field_names[j];
                row_parts.push(format_postgres_value(val, field_name));
            }
            let row = row_parts.join(", ");

            if i == self.updates.len() - 1 {
                writeln!(&mut file, "    ({})", row)?;
            } else {
                writeln!(&mut file, "    ({}),", row)?;
            }
        }

        // cedula + field_names
        let mut all_cols = vec!["cedula".to_string()];
        all_cols.extend(self.field_names.clone());
        writeln!(&mut file, ") AS v({})", all_cols.join(", "))?;
        writeln!(&mut file, "WHERE b.cedula = v.cedula;")?;
        writeln!(&mut file)?;
        writeln!(&mut file, "COMMIT;")?;

        Ok(())
    }
}

fn format_postgres_value(val: &str, field_name: &str) -> String {
    // Truncar fechas a YYYY-MM-DD
    if field_name == "f_ingreso" || field_name == "f_ult_ascenso" || field_name == "fecha_ingreso" {
        let truncated = val.split('T').next().unwrap_or(val);
        return format!("'{}'", truncated.replace("'", "''"));
    }

    // Numéricos enteros: sin comillas
    match field_name {
        "grado_id" | "n_hijos" | "anio_reconocido" | "mes_reconocido" | "dia_reconocido" => {
            if val.trim().is_empty() {
                "0".to_string()
            } else {
                val.to_string()
            }
        }
        _ => {
            // Strings con comillas
            format!("'{}'", val.replace("'", "''"))
        }
    }
}
