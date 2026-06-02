use std::fs::File;
use std::io::Write;

use crate::error::Result;
use crate::types::MetricsSnapshot;

pub fn write_metrics_report(
    path: &str,
    metrics: &MetricsSnapshot,
    pendientes_count: u64,
    nuevos_count: u64,
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
    writeln!(file, "  correctos.csv      → Registros 100% coincidentes (mismo formato que origen)")?;
    writeln!(file, "  rechazos.csv       → Registros con diferencias (mismo formato que origen)")?;
    writeln!(file, "  nuevos.csv         → Registros CSV sin identificación en gRPC (INSERT)")?;
    writeln!(file, "  errores.jsonl      → Registros con diferencias detalladas campo a campo")?;
    writeln!(file, "  pendientes.csv     → Registros gRPC no existentes en CSV (cedula,nombre,apellidos,sexo)")?;
    writeln!(file, "  detalle.txt        → Descripción legible de diferencias por cédula")?;
    writeln!(file, "  postgres_batch.sql → Batch UPDATE PostgreSQL con todos los campos")?;
    writeln!(file, "  insert_batch.sql   → Batch INSERT PostgreSQL para registros nuevos")?;
    writeln!(file)?;

    Ok(())
}
