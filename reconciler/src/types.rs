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
    pub pg_host: String,
    pub pg_port: String,
    pub pg_user: String,
    pub pg_database: String,
    pub pg_password: String,
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
    /// Valor leído del CSV.
    pub expected: String,
    /// Valor leído de gRPC (SSSIFANB).
    pub actual: String,
}

impl FieldDiff {
    /// Categoría a la que pertenece el campo que difiere.
    ///
    /// Fuente única de verdad: la usan tanto `detalle.txt` como el índice
    /// persistido en MongoDB para que la razón del rechazo sea idéntica en
    /// ambos artefactos.
    pub fn categoria(&self) -> Categoria {
        Categoria::from_field_name(&self.field_name)
    }
}

/// Catálogo REAL y definitivo que consume el frontend Angular de auditoría
/// de discrepancias (`auditoria-discrepancias.component.ts`).
///
/// Contiene los IDs de las carpetas/motivos que navega la UI. El reconciler
/// solo produce las seis primeras (categorías de campo, ver [`Categoria`]);
/// las cinco últimas son categorías por estatus que llegan del stream
/// `LISTAR_TMP_COMPONENTE` (`status_id` / `componente_id`) y NO deben ser
/// emitidas por el motor de cálculo. `TODOS` y `SIN DISCREPANCIAS` son
/// pseudo-carpetas de UI y tampoco son motivos.
pub const CATALOGO_UI: [&str; 11] = [
    "FECHAS",
    "GRADOS",
    "TIEMPO_RECONOCIDO",
    "FAMILIA",
    "PROFESION",
    "OTROS",
    "NUEVO",
    "REINCORPORADOS",
    "PARALIZADOS",
    "CAMBIO_COMPONENTE",
    "RETIRADOS",
];

/// Subconjunto de [`CATALOGO_UI`] que NO proviene de diferencias de campo:
/// son categorías por estatus emitidas por `LISTAR_TMP_COMPONENTE`. Se
/// mantienen documentadas aquí para que quede explícito que el reconciler
/// nunca las produce (no hay variantes muertas en [`Categoria`]).
pub const CATALOGO_POR_ESTATUS: [&str; 5] = [
    "NUEVO",
    "REINCORPORADOS",
    "PARALIZADOS",
    "CAMBIO_COMPONENTE",
    "RETIRADOS",
];

/// Categoría de un campo con diferencias.
///
/// Los IDs coinciden EXACTAMENTE con los de la UI. El orden de declaración de
/// las variantes es el orden canónico de presentación y del campo
/// `grupo_validacion` del motivo: `FECHAS`, `GRADOS`, `TIEMPO_RECONOCIDO`,
/// `FAMILIA`, `PROFESION`, `OTROS`. El `Ord` derivado garantiza que
/// `categorias` sea determinista y ya ordenado según la UI.
///
/// Catálogo completo de la UI: ver [`CATALOGO_UI`]. Las categorías
/// `NUEVO`, `REINCORPORADOS`, `PARALIZADOS`, `CAMBIO_COMPONENTE` y
/// `RETIRADOS` son por estatus (`LISTAR_TMP_COMPONENTE`) y NO las emite el
/// reconciler; por eso no existen como variantes de este enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub enum Categoria {
    #[serde(rename = "FECHAS")]
    Fechas,
    #[serde(rename = "GRADOS")]
    Grados,
    #[serde(rename = "TIEMPO_RECONOCIDO")]
    TiempoReconocido,
    #[serde(rename = "FAMILIA")]
    Familia,
    #[serde(rename = "PROFESION")]
    Profesion,
    #[serde(rename = "OTROS")]
    Otros,
}

impl Categoria {
    /// Nombre exacto que se emite en JSON, en `detalle.txt` y en el campo
    /// `grupo_validacion` del motivo.
    pub fn as_str(self) -> &'static str {
        match self {
            Categoria::Fechas => "FECHAS",
            Categoria::Grados => "GRADOS",
            Categoria::TiempoReconocido => "TIEMPO_RECONOCIDO",
            Categoria::Familia => "FAMILIA",
            Categoria::Profesion => "PROFESION",
            Categoria::Otros => "OTROS",
        }
    }

    /// Clasifica un campo por su nombre (tal como aparece en
    /// `FieldMapping.field_name`).
    pub fn from_field_name(field_name: &str) -> Self {
        match field_name {
            "grado_id" | "grado" => Categoria::Grados,
            "n_hijos" => Categoria::Familia,
            "f_ingreso" | "f_ult_ascenso" | "fecha_ingreso" => Categoria::Fechas,
            "anio_reconocido" | "mes_reconocido" | "dia_reconocido" => Categoria::TiempoReconocido,
            "st_profesion" => Categoria::Profesion,
            _ => Categoria::Otros,
        }
    }
}

impl std::fmt::Display for Categoria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Normaliza una fecha al formato `YYYY-MM-DD`.
///
/// Fuente única de verdad para todo artefacto de salida que deba exponer
/// fechas homologadas (`detalles`, `motivo.campos[]`, `detalle.txt`,
/// `rechazos.csv`, `nuevos.csv`, `pendientes.csv`, `postgres_batch.sql` e
/// `insert_batch.sql`).
///
/// - Si el valor empieza por una fecha válida `YYYY-MM-DD` (aunque venga
///   con `T`, `Z` o fracciones de segundo), devuelve exactamente esos 10
///   caracteres.
/// - Si viene vacío o no es parseable, devuelve el valor sin cambios.
///   Nunca inventa valores ni rellena con `0`.
pub fn normalizar_fecha(valor: &str) -> String {
    if valor.trim().is_empty() {
        return valor.to_string();
    }
    match valor.get(..10) {
        Some(candidate) if chrono::NaiveDate::parse_from_str(candidate, "%Y-%m-%d").is_ok() => {
            candidate.to_string()
        }
        _ => valor.to_string(),
    }
}

/// Indica si un campo del mapping representa una fecha.
///
/// Se mantiene alineado con [`Categoria::Fechas`] y agrega `f_ingreso_sistema`,
/// que también llega desde gRPC con hora (`T00:00:00Z`).
pub fn es_campo_fecha(field_name: &str) -> bool {
    matches!(
        field_name,
        "f_ingreso" | "f_ult_ascenso" | "fecha_ingreso" | "f_ingreso_sistema"
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizar_fecha_recorta_timestamp_iso() {
        assert_eq!(normalizar_fecha("2023-07-05T00:00:00Z"), "2023-07-05");
        assert_eq!(
            normalizar_fecha("2023-07-05T23:59:59.123456Z"),
            "2023-07-05"
        );
    }

    #[test]
    fn test_normalizar_fecha_es_idempotente() {
        assert_eq!(normalizar_fecha("2023-07-05"), "2023-07-05");
        let una_vez = normalizar_fecha("2023-07-05T00:00:00Z");
        assert_eq!(normalizar_fecha(&una_vez), "2023-07-05");
    }

    #[test]
    fn test_normalizar_fecha_vacio_no_inventa() {
        assert_eq!(normalizar_fecha(""), "");
        assert_eq!(normalizar_fecha("   "), "   ");
    }

    #[test]
    fn test_normalizar_fecha_invalida_devuelve_tal_cual() {
        assert_eq!(normalizar_fecha("no-es-fecha"), "no-es-fecha");
        assert_eq!(normalizar_fecha("05/07/2023"), "05/07/2023");
        // Fecha con mes/día fuera de rango: no se debe inventar ni truncar.
        assert_eq!(normalizar_fecha("2023-13-45"), "2023-13-45");
        assert_eq!(normalizar_fecha("2023-07-05extra"), "2023-07-05");
    }

    #[test]
    fn test_es_campo_fecha_cubre_los_cuatro_nombres() {
        for field in [
            "f_ingreso",
            "f_ult_ascenso",
            "fecha_ingreso",
            "f_ingreso_sistema",
        ] {
            assert!(es_campo_fecha(field), "{} debe ser fecha", field);
        }
        for field in ["grado", "grado_id", "n_hijos", "st_profesion", "otro"] {
            assert!(!es_campo_fecha(field), "{} no debe ser fecha", field);
        }
    }

    /// Guardián: todos los campos fecha del catálogo pertenecen a FECHAS.
    #[test]
    fn test_campos_fecha_coinciden_con_categoria_fechas() {
        for field in ["f_ingreso", "f_ult_ascenso", "fecha_ingreso"] {
            assert_eq!(Categoria::from_field_name(field), Categoria::Fechas);
            assert!(es_campo_fecha(field));
        }
    }
}
