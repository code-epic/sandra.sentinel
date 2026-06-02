use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;

use sandra_core::kernel::sandra::sentinel_dynamic_service_client::SentinelDynamicServiceClient;
use sandra_core::kernel::sandra::DynamicRequest;
use serde_json::Value;

use crate::compare::mapping::extract_value_from_json;
use crate::compare::traits::{compare_values, FieldComparison};
use crate::csv::index::build_index;
use crate::csv::reader::mmap_file;
use crate::error::Result;
use crate::output::{correctos, metrics, postgres};
use crate::types::{ConciliationResult, ConciliationStatus, FieldDiff, LiveMetrics, ReconcilerConfig};

pub mod cli;
pub mod compare;
pub mod csv;
pub mod error;
pub mod output;
pub mod types;

pub async fn run(config: ReconcilerConfig, quiet: bool) -> Result<()> {
    let start_time = std::time::Instant::now();

    if !quiet {
        println!("Sandra Reconciler v1.0.0 - Streaming CSV vs gRPC");
        println!("{:-<80}", "");
    }

    // PASO 1: CARGAR CSV EN ÍNDICE
    if !quiet {
        println!("[1/4] Cargando CSV: {}", config.csv_path);
    }

    let mmap = mmap_file(&config.csv_path)?;
    let csv_index = build_index(&mmap, config.delimiter, config.skip_header, 0)?;

    if !quiet {
        println!("      {} registros indexados ({} warnings)", csv_index.total_lines, csv_index.warnings.len());
    }

    // PASO 2-3: CONECTAR gRPC Y CONSUMIR STREAM
    if !quiet {
        println!("[2/4] Conectando a gRPC: {}", config.grpc_url);
    }

    let mut tasks: Vec<tokio::task::JoinHandle<Vec<Value>>> = Vec::new();
    let mut chunks = 0u64;
    let mut net_time = std::time::Duration::new(0, 0);
    let mut stream_available = false;

    match SentinelDynamicServiceClient::connect(config.grpc_url.clone()).await {
        Ok(client) => {
            let mut client = client.max_decoding_message_size(usize::MAX);
            let request = tonic::Request::new(DynamicRequest {
                funcion: config.grpc_function.clone(),
                parametros: config.grpc_parametros.clone(),
                valores: "null".to_string(),
            });

            match client.execute_dynamic(request).await {
                Ok(response) => {
                    stream_available = true;
                    let mut stream = response.into_inner();
                    let mut t_last = std::time::Instant::now();

                    if !quiet {
                        println!("[3/4] Consumiendo stream y parseando batches...");
                    }

                    while let Ok(Some(msg)) = stream.message().await {
                        net_time += t_last.elapsed();
                        chunks += 1;

                        if msg.rows.is_empty() {
                            t_last = std::time::Instant::now();
                            continue;
                        }

                        let rows_data = msg.rows;
                        let task = tokio::spawn(async move {
                            match serde_json::from_slice::<Vec<Value>>(&rows_data) {
                                Ok(items) => items,
                                Err(e) => {
                                    eprintln!("[ERROR] Error deserializando batch JSON: {}", e);
                                    Vec::new()
                                }
                            }
                        });

                        tasks.push(task);
                        t_last = std::time::Instant::now();
                    }
                }
                Err(e) => {
                    eprintln!("[WARN] Error en execute_dynamic: {}. Procesando CSV como huérfanos.", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[WARN] No se pudo conectar a gRPC: {}. Procesando CSV como huérfanos.", e);
        }
    }

    if stream_available && !quiet {
        println!("    > Descarga completada (Red: {:.2?}). {} lotes. Fusionando/Comparando...", net_time, chunks);
    }

    // PASO 4: RECOLECTAR TODOS LOS BATCHES gRPC EN MEMORIA
    let mut all_batches: Vec<Vec<Value>> = Vec::new();
    for task in tasks {
        match task.await {
            Ok(batch) => all_batches.push(batch),
            Err(e) => eprintln!("[ERROR] Error en tarea de parsing: {}", e),
        }
    }

    let total_stream_records: usize = all_batches.iter().map(|b| b.len()).sum();

    // Si no hay mapping, detectamos dinámicamente del primer batch
    let field_mappings: Vec<crate::types::FieldMapping>;
    let field_names: Vec<String>;

    if let Some(ref mappings) = config.field_mappings {
        field_mappings = mappings.clone();
        field_names = field_mappings.iter().map(|m| m.field_name.clone()).collect();
    } else {
        let first_batch = all_batches.first().ok_or_else(|| {
            crate::error::ReconcilerError::Mapping("No se recibieron datos del stream gRPC".to_string())
        })?;
        (field_mappings, field_names) = detect_dynamic_mappings(&csv_index.headers, first_batch, quiet).await?;
        if !quiet {
            println!("      Mapping detectado automáticamente ({} campos): {:?}", field_names.len(), field_names);
        }
    }

    // PASO 5: INICIALIZAR WRITERS Y PROCESAR
    let out_dir = &config.output_dir;
    std::fs::create_dir_all(out_dir)?;

    let headers_line = std::str::from_utf8(&mmap)
        .map_err(|e| crate::error::ReconcilerError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    let mut correctos_writer = correctos::init_correctos(&format!("{}/correctos.csv", out_dir), &headers_line)?;
    let mut errores_writer = output::errores::RichErrorWriter::new(&format!("{}/errores.jsonl", out_dir))?;
    let mut pendientes_writer = output::pendientes::PendienteWriter::new(&format!("{}/pendientes.csv", out_dir), field_names.clone(), config.delimiter)?;
    let mut rechazos_writer = output::rechazos::RechazosWriter::new(&format!("{}/rechazos.csv", out_dir), field_names.clone(), config.delimiter)?;
    let mut nuevos_writer = output::nuevos::NuevosWriter::new(&format!("{}/nuevos.csv", out_dir), field_names.clone(), config.delimiter)?;
    let mut detalle_writer = output::detalle::DetalleWriter::new(&format!("{}/detalle.txt", out_dir))?;
    let mut postgres_builder = postgres::PostgresBatchBuilder::new(field_names.clone());
    let mut postgres_insert_builder = output::postgres_insert::PostgresInsertBuilder::new(field_names.clone());

    let metrics = Arc::new(LiveMetrics::new());
    let mut processed_cedulas = HashSet::new();
    let mut debug_records: Vec<Value> = Vec::new();
    let mut debug_not_in_csv_printed = 0usize;
    let mut debug_in_csv_printed = 0usize;

    for batch_items in all_batches {
        for record in batch_items {
            let cedula_raw = extract_value_from_json(&record, &["cedula".to_string()]).unwrap_or_default();
            let cedula_norm: String = cedula_raw.chars().filter(|c| c.is_ascii_digit()).collect();

            if cedula_norm.is_empty() {
                eprintln!("[WARN] Registro gRPC sin cédula válida. Ignorando.");
                continue;
            }

            processed_cedulas.insert(cedula_norm.clone());

            let cedula_for_result = cedula_raw.clone();
            let (result, all_values) = if let Some(csv_record) = csv_index.inner.get(&cedula_norm) {
                let all = extract_all_field_values(&record, csv_record, &field_mappings);
                let r = compare_record(&record, &cedula_raw, csv_record, &field_mappings, &metrics);
                (r, Some(all))
            } else {
                metrics.records_not_found_csv.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let all = extract_all_field_values_from_grpc(&record, &field_mappings);
                let r = ConciliationResult {
                    cedula: cedula_for_result,
                    status: ConciliationStatus::NotFoundInCsv,
                    csv_line: None,
                    diffs: vec![],
                    processing_time: std::time::Duration::ZERO,
                    chunk_id: 0,
                };
                (r, Some(all))
            };

            if config.debug {
                if debug_records.is_empty() {
                    print_field_names_diagnostic(&record);
                }
                if debug_records.len() < 5 {
                    debug_records.push(record.clone());
                }
                let is_in_csv = csv_index.inner.contains_key(&cedula_norm);
                if !is_in_csv && debug_not_in_csv_printed < 3 {
                    print_debug_comparison(&record, &result, csv_index.inner.get(&cedula_norm));
                    debug_not_in_csv_printed += 1;
                } else if is_in_csv && debug_in_csv_printed < 7 {
                    print_debug_comparison(&record, &result, csv_index.inner.get(&cedula_norm));
                    debug_in_csv_printed += 1;
                }
            }

            match result.status {
                ConciliationStatus::FullMatch => {
                    correctos::write_correcto(&mut correctos_writer, &result)?;
                }
                ConciliationStatus::PartialMatch => {
                    if let Some(ref vals) = all_values {
                        postgres_builder.add(&cedula_raw, vals.clone());
                        rechazos_writer.write_record(&cedula_raw, vals)?;
                    }
                    errores_writer.write(&result)?;
                    detalle_writer.write_record(&result)?;
                }
                ConciliationStatus::NotFoundInCsv => {
                    if let Some(ref vals) = all_values {
                        pendientes_writer.write(&cedula_raw, &record, vals)?;
                    }
                }
                _ => {}
            }
        }
    }

    correctos_writer.flush()?;
    errores_writer.flush()?;
    pendientes_writer.flush()?;
    rechazos_writer.flush()?;
    detalle_writer.flush()?;

    // PASO 5: REGISTROS CSV NO ENCONTRADOS EN STREAM
    if !quiet {
        println!("[4/4] Verificando registros CSV no encontrados en stream...");
    }

    let mut huerfanos_count = 0;
    for (cedula, csv_record) in &csv_index.inner {
        if !processed_cedulas.contains(cedula.as_str()) {
            huerfanos_count += 1;
            metrics.records_not_found_grpc.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let csv_vals = extract_all_field_values_from_csv(csv_record, &field_mappings);
            nuevos_writer.write_record(cedula, &csv_vals)?;
            postgres_insert_builder.add(cedula, csv_vals);
        }
    }

    if huerfanos_count > 0 && !quiet {
        println!("      Detectados {} registros CSV sin match en el stream gRPC (nuevos)", huerfanos_count);
    }

    correctos_writer.flush()?;
    errores_writer.flush()?;
    pendientes_writer.flush()?;
    rechazos_writer.flush()?;
    nuevos_writer.flush()?;
    detalle_writer.flush()?;
    postgres_builder.write_to_file(&format!("{}/postgres_batch.sql", out_dir))?;
    postgres_insert_builder.write_to_file(&format!("{}/insert_batch.sql", out_dir))?;

    if config.debug && !debug_records.is_empty() {
        let _ = save_debug_sample(&debug_records, out_dir);
    }

    // PASO 6: REPORTE FINAL
    let mut final_metrics = metrics.snapshot();
    final_metrics.records_processed = final_metrics.records_matched + final_metrics.records_partial + final_metrics.records_not_found_csv;
    let pendientes_count = final_metrics.records_not_found_csv;
    let nuevos_count = final_metrics.records_not_found_grpc;

    metrics::write_metrics_report(&format!("{}/reporte.txt", out_dir), &final_metrics, pendientes_count, nuevos_count)?;

    if !quiet {
        println!("\n{:-<80}", "");
        println!("CONCILIACIÓN COMPLETADA");
        println!("  Tiempo total:    {:.2?}", start_time.elapsed());
        println!("  Registros CSV:   {}", csv_index.total_lines);
        println!("  Registros gRPC:  {}", total_stream_records);
        println!("  Chunks:          {}", chunks);
        println!("  Hits (100%):     {} ({:.2}%)", final_metrics.records_matched, final_metrics.hit_rate);
        println!("  Diferencias:     {}", final_metrics.records_partial);
        println!("  No en CSV:       {}", final_metrics.records_not_found_csv);
        println!("  Pendientes rev:  {}", pendientes_count);
        println!("  Nuevos (sin ID): {}", nuevos_count);
        println!("  Salidas en:      {}/", out_dir);
        println!("{:-<80}", "");
    }

    Ok(())
}

fn compare_record(
    record: &Value,
    cedula_raw: &str,
    csv_record: &crate::types::CsvRecord,
    field_mappings: &[crate::types::FieldMapping],
    metrics: &Arc<LiveMetrics>,
) -> ConciliationResult {
    let t_start = std::time::Instant::now();
    let mut diffs = Vec::with_capacity(field_mappings.len());

    for mapping in field_mappings.iter() {
        let csv_val = csv_record.fields.get(&mapping.csv_column).map(|s| s.as_str()).unwrap_or("");
        let grpc_val = extract_value_from_json(record, &mapping.grpc_path).unwrap_or_default();

        if grpc_val.trim().is_empty() {
            continue;
        }

        let comparison = compare_values(csv_val, &grpc_val, &mapping.strategy);

        if let FieldComparison::Mismatch { expected, actual } = comparison {
            diffs.push(FieldDiff {
                field_name: mapping.field_name.clone(),
                expected,
                actual,
            });
        }
    }

    let elapsed = t_start.elapsed();

    if diffs.is_empty() {
        metrics.records_matched.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        ConciliationResult {
            cedula: cedula_raw.to_string(),
            status: ConciliationStatus::FullMatch,
            csv_line: Some(csv_record.raw_line.to_string()),
            diffs: vec![],
            processing_time: elapsed,
            chunk_id: 0,
        }
    } else {
        metrics.records_partial.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        ConciliationResult {
            cedula: cedula_raw.to_string(),
            status: ConciliationStatus::PartialMatch,
            csv_line: Some(csv_record.raw_line.to_string()),
            diffs,
            processing_time: elapsed,
            chunk_id: 0,
        }
    }
}

fn extract_all_field_values(
    record: &Value,
    csv_record: &crate::types::CsvRecord,
    field_mappings: &[crate::types::FieldMapping],
) -> Vec<String> {
    field_mappings.iter().map(|m| {
        let grpc_val = extract_value_from_json(record, &m.grpc_path).unwrap_or_default();
        if grpc_val.trim().is_empty() {
            csv_record.fields.get(&m.csv_column).cloned().unwrap_or_default()
        } else {
            grpc_val
        }
    }).collect()
}

fn extract_all_field_values_from_grpc(
    record: &Value,
    field_mappings: &[crate::types::FieldMapping],
) -> Vec<String> {
    field_mappings.iter().map(|m| {
        extract_value_from_json(record, &m.grpc_path).unwrap_or_default()
    }).collect()
}

fn extract_all_field_values_from_csv(
    csv_record: &crate::types::CsvRecord,
    field_mappings: &[crate::types::FieldMapping],
) -> Vec<String> {
    field_mappings.iter().map(|m| {
        csv_record.fields.get(&m.csv_column).cloned().unwrap_or_default()
    }).collect()
}

async fn detect_dynamic_mappings(
    csv_headers: &[String],
    first_batch: &[Value],
    quiet: bool,
) -> Result<(Vec<crate::types::FieldMapping>, Vec<String>)> {

    let sample = first_batch.first().ok_or_else(|| {
        crate::error::ReconcilerError::Mapping("Primer batch gRPC vacío".to_string())
    })?;

    let grpc_fields = if let Value::Object(map) = sample {
        map.keys().cloned().collect::<Vec<String>>()
    } else {
        return Err(crate::error::ReconcilerError::Mapping(
            "Registro gRPC no es un objeto JSON".to_string()
        ));
    };

    let mut mappings = vec![];
    let mut field_names = vec![];

    for header in csv_headers {
        if header == "cedula" {
            continue;
        }

        // Coincidencia exacta
        let grpc_field = if grpc_fields.contains(header) {
            header.clone()
        } else {
            // Reglas de normalización comunes
            match header.as_str() {
                "grado" => "grado_id".to_string(),
                "fecha_ingreso" => "f_ingreso".to_string(),
                "f_ingreso_sistema" => "f_ingreso".to_string(),
                _ => header.clone(),
            }
        };

        if !grpc_fields.contains(&grpc_field) {
            if !quiet {
                eprintln!("[WARN] Campo '{}' del CSV no encontrado en gRPC (intentado '{}'). Omitiendo.", header, grpc_field);
            }
            continue;
        }

        let strategy = infer_strategy(&grpc_field);
        field_names.push(grpc_field.clone());
        mappings.push(crate::types::FieldMapping {
            field_name: grpc_field.clone(),
            csv_column: header.clone(),
            grpc_path: vec![grpc_field],
            strategy,
        });
    }

    if mappings.is_empty() {
        return Err(crate::error::ReconcilerError::Mapping(
            "No se pudo detectar ningún campo coincidente entre CSV y gRPC".to_string()
        ));
    }

    Ok((mappings, field_names))
}

fn infer_strategy(field_name: &str) -> crate::types::ComparisonStrategy {
    match field_name {
        "f_ingreso" | "f_ult_ascenso" | "fecha_ingreso" => crate::types::ComparisonStrategy::Date,
        "n_hijos" | "anio_reconocido" | "mes_reconocido" | "dia_reconocido" | "grado_id" => {
            crate::types::ComparisonStrategy::Numeric { epsilon: 0.0 }
        }
        _ => crate::types::ComparisonStrategy::Exact,
    }
}

fn save_debug_sample(records: &[Value], out_dir: &str) -> Result<()> {
    let debug_dir = format!("{}/debug", out_dir);
    std::fs::create_dir_all(&debug_dir)?;
    let path = format!("{}/grpc_sample.json", debug_dir);
    let file = std::fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, records)?;
    println!("  [DEBUG] Muestra gRPC guardada en: {}", path);
    Ok(())
}

fn print_debug_comparison(
    _record: &Value,
    result: &ConciliationResult,
    csv_record: Option<&crate::types::CsvRecord>,
) {
    println!("\n  [DEBUG] --- Comparación para cédula: {} (Status: {:?}) ---", result.cedula, result.status);
    if let Some(_csv) = csv_record {
        println!("  {:<25} {:<20} {:<20} {:<10}", "CAMPO", "CSV", "gRPC", "RESULTADO");
        println!("  {:-<80}", "");
        for mapping in result.diffs.iter() {
            println!("  {:<25} {:<20} {:<20} {:<10}", mapping.field_name, mapping.actual, mapping.expected, "DIFF");
        }
        if result.diffs.is_empty() {
            println!("  {:<25} {:<20} {:<20} {:<10}", "(todos)", "-", "-", "MATCH");
        }
        println!("  {:-<80}", "");
    } else {
        println!("  [DEBUG] No encontrado en CSV");
    }
}

fn print_field_names_diagnostic(record: &Value) {
    if let Value::Object(map) = record {
        let keys: Vec<&String> = map.keys().collect();
        println!("\n  [DEBUG] === NOMBRES DE CAMPOS EN JSON gRPC ({} campos) ===", keys.len());
        let mut line = String::from("  ");
        for (i, k) in keys.iter().enumerate() {
            line.push_str(&format!("{:<22}", k));
            if (i + 1) % 4 == 0 {
                println!("{}", line);
                line = String::from("  ");
            }
        }
        if !line.trim().is_empty() {
            println!("{}", line);
        }
        println!("  [DEBUG] ============================================================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_csv_index_populates_fields() {
        let csv_data = b"cedula,grado,n_hijos\n12345678,Coronel,2\n87654321,General,3";
        let index = csv::index::build_index(csv_data, ',', false, 0).unwrap();

        assert_eq!(index.inner.len(), 2);
        assert!(index.inner.contains_key("12345678"));
        assert!(index.inner.contains_key("87654321"));

        let rec1 = index.inner.get("12345678").unwrap();
        assert_eq!(rec1.fields.get("grado"), Some(&"Coronel".to_string()));
        assert_eq!(rec1.fields.get("n_hijos"), Some(&"2".to_string()));

        let rec2 = index.inner.get("87654321").unwrap();
        assert_eq!(rec2.fields.get("grado"), Some(&"General".to_string()));
        assert_eq!(rec2.fields.get("n_hijos"), Some(&"3".to_string()));
    }

    #[test]
    fn test_compare_record_exact_match() {
        let csv_record = crate::types::CsvRecord {
            fields: HashMap::from([
                ("grado".to_string(), "Coronel".to_string()),
                ("n_hijos".to_string(), "2".to_string()),
            ]),
            raw_line: "12345678,Coronel,2".to_string(),
        };

        let grpc_record = json!({
            "cedula": "12345678",
            "grado_id": "Coronel",
            "n_hijos": "2"
        });

        let mappings = vec![
            crate::types::FieldMapping {
                field_name: "grado".to_string(),
                csv_column: "grado".to_string(),
                grpc_path: vec!["grado_id".to_string()],
                strategy: crate::types::ComparisonStrategy::Exact,
            },
            crate::types::FieldMapping {
                field_name: "n_hijos".to_string(),
                csv_column: "n_hijos".to_string(),
                grpc_path: vec!["n_hijos".to_string()],
                strategy: crate::types::ComparisonStrategy::Numeric { epsilon: 0.0 },
            },
        ];

        let metrics = std::sync::Arc::new(crate::types::LiveMetrics::new());
        let result = compare_record(&grpc_record, "12345678", &csv_record, &mappings, &metrics);

        assert_eq!(result.status, crate::types::ConciliationStatus::FullMatch);
        assert!(result.diffs.is_empty());
    }

    #[test]
    fn test_compare_record_date_match() {
        let csv_record = crate::types::CsvRecord {
            fields: HashMap::from([
                ("f_ingreso".to_string(), "2010-05-01".to_string()),
            ]),
            raw_line: "12345678,2010-05-01".to_string(),
        };

        let grpc_record = json!({
            "cedula": "12345678",
            "f_ingreso": "2010-05-01T00:00:00Z"
        });

        let mappings = vec![
            crate::types::FieldMapping {
                field_name: "f_ingreso".to_string(),
                csv_column: "f_ingreso".to_string(),
                grpc_path: vec!["f_ingreso".to_string()],
                strategy: crate::types::ComparisonStrategy::Date,
            },
        ];

        let metrics = std::sync::Arc::new(crate::types::LiveMetrics::new());
        let result = compare_record(&grpc_record, "12345678", &csv_record, &mappings, &metrics);

        assert_eq!(result.status, crate::types::ConciliationStatus::FullMatch);
    }

    #[test]
    fn test_compare_record_partial_mismatch() {
        let csv_record = crate::types::CsvRecord {
            fields: HashMap::from([
                ("grado".to_string(), "Coronel".to_string()),
                ("n_hijos".to_string(), "5".to_string()),
            ]),
            raw_line: "12345678,Coronel,5".to_string(),
        };

        let grpc_record = json!({
            "cedula": "12345678",
            "grado_id": "General",
            "n_hijos": "2"
        });

        let mappings = vec![
            crate::types::FieldMapping {
                field_name: "grado".to_string(),
                csv_column: "grado".to_string(),
                grpc_path: vec!["grado_id".to_string()],
                strategy: crate::types::ComparisonStrategy::Exact,
            },
            crate::types::FieldMapping {
                field_name: "n_hijos".to_string(),
                csv_column: "n_hijos".to_string(),
                grpc_path: vec!["n_hijos".to_string()],
                strategy: crate::types::ComparisonStrategy::Numeric { epsilon: 0.0 },
            },
        ];

        let metrics = std::sync::Arc::new(crate::types::LiveMetrics::new());
        let result = compare_record(&grpc_record, "12345678", &csv_record, &mappings, &metrics);

        assert_eq!(result.status, crate::types::ConciliationStatus::PartialMatch);
        assert_eq!(result.diffs.len(), 2);
    }

    #[test]
    fn test_postgres_batch_builder_generates_proper_sql() {
        let field_names = vec!["grado_id".to_string(), "n_hijos".to_string(), "f_ingreso".to_string()];
        let mut builder = crate::output::postgres::PostgresBatchBuilder::new(field_names);
        // Agregar un registro con TODOS los campos (no solo los que difieren)
        builder.add("12345678", vec![
            "2080".to_string(),      // grado_id (diferente)
            "3".to_string(),         // n_hijos (diferente)
            "2025-01-01T00:00:00Z".to_string(), // f_ingreso (diferente)
        ]);

        let tmp_file = "/tmp/test_postgres_batch.sql";
        builder.write_to_file(tmp_file).unwrap();

        let content = std::fs::read_to_string(tmp_file).unwrap();
        // Batch UPDATE con VALUES y todos los campos
        assert!(content.contains("UPDATE beneficiarios AS b SET"));
        assert!(content.contains("grado_id = v.grado_id,"));
        assert!(content.contains("n_hijos = v.n_hijos,"));
        assert!(content.contains("f_ingreso = v.f_ingreso"));
        assert!(content.contains("FROM (VALUES"));
        assert!(content.contains("'12345678', 2080, 3, '2025-01-01'"));
        assert!(content.contains(") AS v(cedula, grado_id, n_hijos, f_ingreso)"));
        assert!(content.contains("WHERE b.cedula = v.cedula;"));
        assert!(content.contains("BEGIN;"));
        assert!(content.contains("COMMIT;"));
    }
}
