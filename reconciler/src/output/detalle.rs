use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

use crate::error::Result;
use crate::types::{es_campo_fecha, normalizar_fecha, Categoria, ConciliationResult};

pub struct DetalleWriter {
    file: File,
}

impl DetalleWriter {
    pub fn new(path: &str) -> Result<Self> {
        let mut file = File::create(path)?;
        writeln!(file, "DETALLE DE DIFERENCIAS - SANDRA RECONCILER")?;
        writeln!(file, "==========================================")?;
        writeln!(file)?;
        Ok(DetalleWriter { file })
    }

    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        Ok(())
    }

    pub fn write_record(&mut self, result: &ConciliationResult) -> Result<()> {
        writeln!(self.file, "CEDULA: {}", result.cedula)?;
        writeln!(self.file, "  Status: {:?}", result.status)?;

        if !result.diffs.is_empty() {
            writeln!(self.file, "  Campos con diferencias:")?;

            // BTreeMap<Categoria, _> respeta el orden canónico del enum
            // (FECHAS, GRADOS, TIEMPO_RECONOCIDO, FAMILIA, PROFESION, OTROS),
            // idéntico al de `motivo.categorias` en el JSON. Las etiquetas
            // impresas son los mismos IDs que consume la UI Angular.
            let mut por_categoria: BTreeMap<Categoria, Vec<String>> = BTreeMap::new();

            for diff in &result.diffs {
                // Se normalizan las fechas para que `detalle.txt` coincida
                // exactamente con `motivo.campos[]` del índice JSON.
                let (expected, actual) = if es_campo_fecha(&diff.field_name) {
                    (
                        normalizar_fecha(&diff.expected),
                        normalizar_fecha(&diff.actual),
                    )
                } else {
                    (diff.expected.clone(), diff.actual.clone())
                };
                let line = format!(
                    "    - {}: CSV={}, gRPC={}",
                    diff.field_name, expected, actual
                );
                por_categoria
                    .entry(diff.categoria())
                    .or_default()
                    .push(line);
            }

            for (categoria, lines) in por_categoria {
                writeln!(self.file, "    [{}]", categoria.as_str())?;
                for l in lines {
                    writeln!(self.file, "{}", l)?;
                }
            }
        }

        if let Some(ref line) = result.csv_line {
            writeln!(self.file, "  Linea CSV origen: {}", line)?;
        }

        writeln!(self.file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ConciliationStatus, FieldDiff};

    /// `detalle.txt` usa el mismo formato de fecha que `motivo.campos[]`.
    #[test]
    fn test_detalle_normaliza_fechas_como_motivo() {
        let path = std::env::temp_dir().join("reconciler_test_detalle_fechas.txt");
        let path = path.to_str().unwrap();
        let mut writer = DetalleWriter::new(path).unwrap();
        let result = ConciliationResult {
            cedula: "11439805".to_string(),
            status: ConciliationStatus::PartialMatch,
            csv_line: Some("11439805;1040;4;1996-07-05;2023-07-05;35;0;0;0".to_string()),
            diffs: vec![FieldDiff {
                field_name: "f_ingreso".to_string(),
                expected: "1996-07-05".to_string(),
                actual: "1997-01-01T00:00:00Z".to_string(),
            }],
            processing_time: std::time::Duration::ZERO,
            chunk_id: 0,
        };
        writer.write_record(&result).unwrap();
        writer.flush().unwrap();

        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("f_ingreso: CSV=1996-07-05, gRPC=1997-01-01"));
        assert!(!content.contains("T00:00:00Z"));
        let _ = std::fs::remove_file(path);
    }
}
