use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};

use serde::Serialize;

use crate::error::Result;
use crate::types::{MetricsSnapshot, ReconcilerConfig};

#[derive(Serialize)]
struct ManifestFile {
    nombre: String,
    descripcion: String,
    tamano_bytes: u64,
    lineas: u64,
}

#[derive(Serialize)]
struct ManifestConfig {
    csv_path: String,
    grpc_url: String,
    grpc_function: String,
    parametros: String,
    output_dir: String,
    delimiter: String,
    skip_header: bool,
    field_mapping: String,
    chunk_size: usize,
}

#[derive(Serialize)]
struct ManifestMetrics {
    registros_csv: u64,
    registros_grpc: u64,
    hits_100: u64,
    diferencias_parciales: u64,
    no_encontrados_csv: u64,
    no_encontrados_grpc: u64,
    registros_nuevos: u64,
    pendientes_revision: u64,
    errores: u64,
    hit_rate: f64,
}

#[derive(Serialize)]
struct CompresionInfo {
    habilitada: bool,
    nivel: u32,
    archivos_comprimidos: Vec<String>,
}

#[derive(Serialize)]
struct Manifest {
    nombre: &'static str,
    fecha: String,
    version: &'static str,
    config: ManifestConfig,
    metrics: ManifestMetrics,
    compresion: CompresionInfo,
    archivos_generados: Vec<ManifestFile>,
}

static ARCHIVOS: &[(&str, &str)] = &[
    ("correctos.csv", "Registros 100% coincidentes"),
    ("rechazos.csv", "Registros con diferencias parciales"),
    ("nuevos.csv", "Registros CSV sin identificacion en gRPC"),
    ("errores.jsonl", "Diferencias detalladas campo a campo en JSON Lines"),
    ("pendientes.csv", "Registros gRPC no existentes en CSV"),
    ("detalle.txt", "Descripcion legible de diferencias clasificadas"),
    ("reporte.txt", "Metricas finales de conciliacion"),
    ("indice_cedulas.json", "Indice de cedulas con parametros y archivo destino"),
    ("cargar_staging.sh", "Script de carga masiva a PostgreSQL"),
];

pub fn count_lines(path: &str) -> u64 {
    match std::fs::File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            reader.lines().count() as u64
        }
        Err(_) => 0,
    }
}

pub fn collect_line_counts(out_dir: &str) -> HashMap<String, u64> {
    ARCHIVOS
        .iter()
        .map(|(nombre, _)| {
            let ruta = format!("{}/{}", out_dir, nombre);
            let lineas = count_lines(&ruta);
            (nombre.to_string(), lineas)
        })
        .collect()
}

pub fn write_manifest(
    path: &str,
    config: &ReconcilerConfig,
    metrics: &MetricsSnapshot,
    pendientes_count: u64,
    nuevos_count: u64,
    total_csv: u64,
    total_grpc: u64,
    line_counts: &HashMap<String, u64>,
) -> Result<()> {
    let mut comprimidos = Vec::new();

    let archivos_con_tamano: Vec<ManifestFile> = if config.compress {
        ARCHIVOS
            .iter()
            .filter_map(|(nombre, descripcion)| {
                let zst_name = format!("{}.zst", nombre);
                let ruta = format!("{}/{}", config.output_dir, zst_name);
                match std::fs::metadata(&ruta) {
                    Ok(meta) => {
                        let zst_desc = format!("{} (comprimido zstd)", descripcion);
                        let lineas = line_counts.get(*nombre).copied().unwrap_or(0);
                        comprimidos.push(zst_name);
                        Some(ManifestFile {
                            nombre: format!("{}.zst", nombre),
                            descripcion: zst_desc,
                            tamano_bytes: meta.len(),
                            lineas,
                        })
                    }
                    Err(_) => None,
                }
            })
            .collect()
    } else {
        ARCHIVOS
            .iter()
            .map(|(nombre, descripcion)| {
                let ruta = format!("{}/{}", config.output_dir, nombre);
                let tamano = std::fs::metadata(&ruta).map(|m| m.len()).unwrap_or(0);
                let lineas = line_counts.get(*nombre).copied().unwrap_or(0);
                ManifestFile {
                    nombre: nombre.to_string(),
                    descripcion: descripcion.to_string(),
                    tamano_bytes: tamano,
                    lineas,
                }
            })
            .collect()
    };

    let num_fields = metrics.records_processed;
    let mapping_origen = match &config.field_mappings {
        Some(_) => format!("archivo externo ({} campos)", num_fields),
        None => format!("deteccion automatica ({} campos)", num_fields),
    };

    let manifest = Manifest {
        nombre: "Reconciliacion Streaming: CSV vs gRPC",
        fecha: chrono::Utc::now().to_rfc3339(),
        version: "1.0.0",
        config: ManifestConfig {
            csv_path: config.csv_path.clone(),
            grpc_url: config.grpc_url.clone(),
            grpc_function: config.grpc_function.clone(),
            parametros: config.grpc_parametros.clone(),
            output_dir: config.output_dir.clone(),
            delimiter: config.delimiter.to_string(),
            skip_header: config.skip_header,
            field_mapping: mapping_origen,
            chunk_size: config.chunk_size,
        },
        metrics: ManifestMetrics {
            registros_csv: total_csv,
            registros_grpc: total_grpc,
            hits_100: metrics.records_matched,
            diferencias_parciales: metrics.records_partial,
            no_encontrados_csv: metrics.records_not_found_csv,
            no_encontrados_grpc: metrics.records_not_found_grpc,
            registros_nuevos: nuevos_count,
            pendientes_revision: pendientes_count,
            errores: metrics.errors,
            hit_rate: metrics.hit_rate,
        },
        compresion: CompresionInfo {
            habilitada: config.compress,
            nivel: 3,
            archivos_comprimidos: comprimidos,
        },
        archivos_generados: archivos_con_tamano,
    };

    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &manifest)?;
    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MetricsSnapshot, ReconcilerConfig};

    fn dummy_config(compress: bool) -> ReconcilerConfig {
        ReconcilerConfig {
            csv_path: "/tmp/test.csv".to_string(),
            grpc_url: "http://localhost:50051".to_string(),
            grpc_function: "IPSFA_CBeneficiarios".to_string(),
            chunk_size: 10000,
            output_dir: "/tmp/manifest_test_out".to_string(),
            delimiter: ';',
            skip_header: true,
            field_mappings: None,
            grpc_parametros: "%".to_string(),
            debug: false,
            compress,
            api_url: None,
            driver: "SSSIFANB".to_string(),
        }
    }

    fn dummy_metrics() -> MetricsSnapshot {
        MetricsSnapshot {
            records_processed: 15000,
            records_matched: 14500,
            records_partial: 400,
            records_not_found_csv: 80,
            records_not_found_grpc: 20,
            errors: 0,
            hit_rate: 96.67,
            total_processing_time_ms: 0.0,
        }
    }

    fn write_test_files(dir: &str, line_map: &HashMap<&str, u64>) {
        std::fs::create_dir_all(dir).unwrap();
        for (nombre, lineas) in line_map {
            let content: String = (0..*lineas).map(|i| format!("linea {}\n", i)).collect();
            std::fs::write(format!("{}/{}", dir, nombre), content).unwrap();
        }
    }

    #[test]
    fn test_manifest_includes_zst_and_lineas_when_compress_enabled() {
        let dir = "/tmp/manifest_test_out_zst";
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).unwrap();

        let mut line_map = HashMap::new();
        for (i, (f, _)) in ARCHIVOS.iter().enumerate() {
            let lineas = (i as u64 + 1) * 3;
            let content: String = (0..lineas).map(|n| format!("linea {}\n", n)).collect();
            std::fs::write(format!("{}/{}", dir, f), content).unwrap();
            line_map.insert(*f, lineas);
            std::fs::write(format!("{}/{}.zst", dir, f), "compressed").unwrap();
        }

        let config = dummy_config(true);
        let config = ReconcilerConfig { output_dir: dir.to_string(), ..config };
        let metrics = dummy_metrics();
        let path = format!("{}/manifest.json", dir);

        let line_counts: HashMap<String, u64> = collect_line_counts(dir);

        write_manifest(&path, &config, &metrics, 80, 20, 15000, 14980, &line_counts).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["compresion"]["habilitada"], true);
        assert_eq!(parsed["compresion"]["nivel"], 3);

        let archivos = parsed["archivos_generados"].as_array().unwrap();
        assert_eq!(archivos.len(), ARCHIVOS.len());

        let all_zst = archivos.iter()
            .all(|f| f["nombre"].as_str().map(|n| n.ends_with(".zst")).unwrap_or(false));
        assert!(all_zst, "Todos los archivos deben ser .zst cuando compress=true");

        for entry in archivos {
            let nombre = entry["nombre"].as_str().unwrap();
            let original = nombre.trim_end_matches(".zst");
            let esperadas = line_map.get(original).unwrap();
            assert_eq!(entry["lineas"].as_u64().unwrap(), *esperadas, "lineas para {}", original);
        }
    }

    #[test]
    fn test_manifest_has_lineas_without_compression() {
        let dir = "/tmp/manifest_test_out_plain";
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).unwrap();

        let mut line_map: HashMap<&str, u64> = HashMap::new();
        for (i, (f, _)) in ARCHIVOS.iter().enumerate() {
            let lineas = (i as u64 + 1) * 2;
            line_map.insert(*f, lineas);
        }

        write_test_files(dir, &line_map);

        let config = dummy_config(false);
        let config = ReconcilerConfig { output_dir: dir.to_string(), ..config };
        let metrics = dummy_metrics();
        let path = format!("{}/manifest.json", dir);

        let line_counts: HashMap<String, u64> = collect_line_counts(dir);

        write_manifest(&path, &config, &metrics, 80, 20, 15000, 14980, &line_counts).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        let archivos = parsed["archivos_generados"].as_array().unwrap();
        assert_eq!(archivos.len(), ARCHIVOS.len());

        for entry in archivos {
            let nombre = entry["nombre"].as_str().unwrap();
            let esperadas = *line_map.get(nombre).unwrap();
            assert!(entry.get("lineas").is_some(), "falta lineas en {}", nombre);
            assert_eq!(entry["lineas"].as_u64().unwrap(), esperadas, "lineas para {}", nombre);
        }
    }
}
