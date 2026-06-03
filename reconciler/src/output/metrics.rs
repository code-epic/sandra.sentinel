use std::fs::File;
use std::io::Write;

use crate::error::Result;
use crate::types::MetricsSnapshot;

pub fn write_metrics_report(
    path: &str,
    metrics: &MetricsSnapshot,
    pendientes_count: u64,
    nuevos_count: u64,
    compress: bool,
) -> Result<()> {
    let mut file = File::create(path)?;

    writeln!(file, "REPORTE DE CONCILIACIÓN")?;
    writeln!(file, "=======================")?;
    writeln!(file)?;
    writeln!(file, "Registros CSV:            {}", metrics.records_processed)?;
    writeln!(file, "Registros gRPC:           {}", metrics.records_processed + metrics.records_not_found_csv)?;
    writeln!(file, "Hits (100% match):        {}", metrics.records_matched)?;
    writeln!(file, "Diferencias parciales:    {}", metrics.records_partial)?;
    writeln!(file, "No encontrados en CSV:    {}", metrics.records_not_found_csv)?;
    writeln!(file, "No encontrados en gRPC:   {}", metrics.records_not_found_grpc)?;
    writeln!(file, "Nuevos (sin ID gRPC):     {}", nuevos_count)?;
    writeln!(file, "Pendientes revisión:      {}", pendientes_count)?;
    writeln!(file, "Errores:                  {}", metrics.errors)?;
    writeln!(file, "Hit rate:                 {:.2}%", metrics.hit_rate)?;
    writeln!(file)?;
    writeln!(file, "ARCHIVOS GENERADOS")?;
    writeln!(file, "  correctos.csv      -> Registros 100% coincidentes (mismo formato que origen)")?;
    writeln!(file, "  rechazos.csv       -> Registros con diferencias (mismo formato que origen)")?;
    writeln!(file, "  nuevos.csv         -> Registros CSV sin identificacion en gRPC (INSERT)")?;
    writeln!(file, "  errores.jsonl      -> Registros con diferencias detalladas campo a campo")?;
    writeln!(file, "  pendientes.csv     -> Registros gRPC no existentes en CSV (cedula,nombre,apellidos,sexo)")?;
    writeln!(file, "  detalle.txt        -> Descripcion legible de diferencias por cedula")?;
    writeln!(file, "  postgres_batch.sql -> Batch UPDATE PostgreSQL con todos los campos")?;
    writeln!(file, "  insert_batch.sql   -> Batch INSERT PostgreSQL para registros nuevos")?;
    if compress {
        writeln!(file, "  *.csv.zst          -> Archivos comprimidos zstd (nivel 3)")?;
        writeln!(file, "  *.txt.zst          -> Archivos comprimidos zstd (nivel 3)")?;
        writeln!(file, "  *.sql.zst          -> Archivos comprimidos zstd (nivel 3)")?;
        writeln!(file, "  *.jsonl.zst        -> Archivos comprimidos zstd (nivel 3)")?;
    }
    writeln!(file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MetricsSnapshot;

    #[test]
    fn test_report_with_compression() {
        let metrics = MetricsSnapshot {
            records_processed: 100,
            records_matched: 95,
            records_partial: 3,
            records_not_found_csv: 1,
            records_not_found_grpc: 1,
            errors: 0,
            hit_rate: 95.0,
            total_processing_time_ms: 0.0,
        };

        let path = "/tmp/test_report_with_zst.txt";
        write_metrics_report(path, &metrics, 1, 1, true).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("*.csv.zst"));
        assert!(content.contains("*.txt.zst"));

        let path2 = "/tmp/test_report_no_zst.txt";
        write_metrics_report(path2, &metrics, 1, 1, false).unwrap();
        let content2 = std::fs::read_to_string(path2).unwrap();
        assert!(!content2.contains("*.csv.zst"));
    }
}
