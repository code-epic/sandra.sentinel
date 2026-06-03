use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReconcilerConfig {
    pub csv_path: String,
    pub grpc_url: String,
    pub grpc_function: String,
    pub chunk_size: usize,
    pub output_dir: String,
    pub delimiter: char,
    pub skip_header: bool,
    pub field_mappings: Option<Vec<FieldMapping>>,
    pub grpc_parametros: String,
    pub debug: bool,
    pub compress: bool,
    pub api_url: Option<String>,
    pub driver: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldMapping {
    pub field_name: String,
    pub csv_column: String,
    pub grpc_path: Vec<String>,
    pub strategy: ComparisonStrategy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ComparisonStrategy {
    Exact,
    Normalized,
    Cedula,
    Numeric { epsilon: f64 },
    Date,
}

#[derive(Debug)]
pub struct CsvRecord {
    pub fields: HashMap<String, String>,
    pub raw_line: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConciliationResult {
    pub cedula: String,
    pub status: ConciliationStatus,
    pub csv_line: Option<String>,
    pub diffs: Vec<FieldDiff>,
    #[serde(skip)]
    pub processing_time: Duration,
    pub chunk_id: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ConciliationStatus {
    FullMatch,
    PartialMatch,
    NotFoundInCsv,
    NotFoundInGrpc,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldDiff {
    pub field_name: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug)]
pub struct LiveMetrics {
    pub records_processed: AtomicU64,
    pub records_matched: AtomicU64,
    pub records_partial: AtomicU64,
    pub records_not_found_csv: AtomicU64,
    pub records_not_found_grpc: AtomicU64,
    pub errors: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub records_processed: u64,
    pub records_matched: u64,
    pub records_partial: u64,
    pub records_not_found_csv: u64,
    pub records_not_found_grpc: u64,
    pub errors: u64,
    pub hit_rate: f64,
    pub total_processing_time_ms: f64,
}

impl LiveMetrics {
    pub fn new() -> Self {
        LiveMetrics {
            records_processed: AtomicU64::new(0),
            records_matched: AtomicU64::new(0),
            records_partial: AtomicU64::new(0),
            records_not_found_csv: AtomicU64::new(0),
            records_not_found_grpc: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let processed = self.records_processed.load(Ordering::Relaxed);
        let matched = self.records_matched.load(Ordering::Relaxed);
        let partial = self.records_partial.load(Ordering::Relaxed);
        let not_found_csv = self.records_not_found_csv.load(Ordering::Relaxed);
        let not_found_grpc = self.records_not_found_grpc.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);

        let hit_rate = if processed > 0 {
            (matched as f64 / processed as f64) * 100.0
        } else {
            0.0
        };

        MetricsSnapshot {
            records_processed: processed,
            records_matched: matched,
            records_partial: partial,
            records_not_found_csv: not_found_csv,
            records_not_found_grpc: not_found_grpc,
            errors,
            hit_rate,
            total_processing_time_ms: 0.0,
        }
    }
}

pub fn default_field_mappings() -> Vec<FieldMapping> {
    vec![
        FieldMapping {
            field_name: "grado".to_string(),
            csv_column: "grado".to_string(),
            grpc_path: vec!["grado_id".to_string()],
            strategy: ComparisonStrategy::Exact,
        },
        FieldMapping {
            field_name: "n_hijos".to_string(),
            csv_column: "n_hijos".to_string(),
            grpc_path: vec!["n_hijos".to_string()],
            strategy: ComparisonStrategy::Numeric { epsilon: 0.0 },
        },
        FieldMapping {
            field_name: "f_ingreso".to_string(),
            csv_column: "f_ingreso".to_string(),
            grpc_path: vec!["f_ingreso".to_string()],
            strategy: ComparisonStrategy::Date,
        },
        FieldMapping {
            field_name: "f_ult_ascenso".to_string(),
            csv_column: "f_ult_ascenso".to_string(),
            grpc_path: vec!["f_ult_ascenso".to_string()],
            strategy: ComparisonStrategy::Date,
        },
        FieldMapping {
            field_name: "st_profesion".to_string(),
            csv_column: "st_profesion".to_string(),
            grpc_path: vec!["st_profesion".to_string()],
            strategy: ComparisonStrategy::Exact,
        },
        FieldMapping {
            field_name: "anio_reconocido".to_string(),
            csv_column: "anio_reconocido".to_string(),
            grpc_path: vec!["anio_reconocido".to_string()],
            strategy: ComparisonStrategy::Numeric { epsilon: 0.0 },
        },
        FieldMapping {
            field_name: "mes_reconocido".to_string(),
            csv_column: "mes_reconocido".to_string(),
            grpc_path: vec!["mes_reconocido".to_string()],
            strategy: ComparisonStrategy::Numeric { epsilon: 0.0 },
        },
        FieldMapping {
            field_name: "dia_reconocido".to_string(),
            csv_column: "dia_reconocido".to_string(),
            grpc_path: vec!["dia_reconocido".to_string()],
            strategy: ComparisonStrategy::Numeric { epsilon: 0.0 },
        },
    ]
}
