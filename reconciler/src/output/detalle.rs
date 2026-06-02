use std::fs::File;
use std::io::Write;

use crate::error::Result;
use crate::types::ConciliationResult;

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

            let mut grado = vec![];
            let mut hijos = vec![];
            let mut fechas = vec![];
            let mut reconocimiento = vec![];
            let mut profesion = vec![];
            let mut otros = vec![];

            for diff in &result.diffs {
                let line = format!(
                    "    - {}: CSV={}, gRPC={}",
                    diff.field_name, diff.expected, diff.actual
                );

                match diff.field_name.as_str() {
                    "grado_id" | "grado" => grado.push(line),
                    "n_hijos" => hijos.push(line),
                    "f_ingreso" | "f_ult_ascenso" | "fecha_ingreso" => fechas.push(line),
                    "anio_reconocido" | "mes_reconocido" | "dia_reconocido" => reconocimiento.push(line),
                    "st_profesion" => profesion.push(line),
                    _ => otros.push(line),
                }
            }

            if !grado.is_empty() {
                writeln!(self.file, "    [GRADO]")?;
                for l in grado { writeln!(self.file, "{}", l)?; }
            }
            if !hijos.is_empty() {
                writeln!(self.file, "    [N_HIJOS]")?;
                for l in hijos { writeln!(self.file, "{}", l)?; }
            }
            if !fechas.is_empty() {
                writeln!(self.file, "    [FECHAS]")?;
                for l in fechas { writeln!(self.file, "{}", l)?; }
            }
            if !reconocimiento.is_empty() {
                writeln!(self.file, "    [RECONOCIMIENTO]")?;
                for l in reconocimiento { writeln!(self.file, "{}", l)?; }
            }
            if !profesion.is_empty() {
                writeln!(self.file, "    [PROFESION]")?;
                for l in profesion { writeln!(self.file, "{}", l)?; }
            }
            if !otros.is_empty() {
                writeln!(self.file, "    [OTROS]")?;
                for l in otros { writeln!(self.file, "{}", l)?; }
            }
        }

        if let Some(ref line) = result.csv_line {
            writeln!(self.file, "  Linea CSV origen: {}", line)?;
        }

        writeln!(self.file)?;
        Ok(())
    }
}
