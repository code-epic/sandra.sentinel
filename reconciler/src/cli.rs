use clap::Parser;
use std::path::PathBuf;

use crate::types::{FieldMapping, ReconcilerConfig};
use crate::error::Result;

#[derive(Parser, Debug)]
#[command(name = "sandra-reconciler")]
#[command(about = "Conciliación de alta velocidad: CSV vs streaming gRPC")]
pub struct ReconcilerCli {
    #[arg(long)]
    pub csv_path: PathBuf,

    #[arg(long, default_value = "http://localhost:50051")]
    pub grpc_url: String,

    #[arg(long, default_value = "IPSFA_CBeneficiarios")]
    pub grpc_function: String,

    #[arg(long, default_value = "\"%\"")]
    pub grpc_parametros: String,

    #[arg(long, default_value_t = 10000)]
    pub chunk_size: usize,

    #[arg(long, default_value = "./reconcile-out")]
    pub output_dir: PathBuf,

    #[arg(long, default_value = ";")]
    pub delimiter: String,

    #[arg(long)]
    pub skip_header: bool,

    #[arg(long)]
    pub field_mapping: Option<PathBuf>,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short, long)]
    pub debug: bool,
}

impl ReconcilerCli {
    pub fn into_config(self) -> Result<ReconcilerConfig> {
        if !self.csv_path.exists() {
            return Err(crate::error::ReconcilerError::FileNotFound(
                format!("CSV no encontrado: {}", self.csv_path.display())
            ));
        }

        let field_mappings = if let Some(path) = self.field_mapping {
            if !path.exists() {
                return Err(crate::error::ReconcilerError::FileNotFound(
                    format!("Mapping JSON no encontrado: {}", path.display())
                ));
            }
            let mappings_json = std::fs::read_to_string(&path)
                .map_err(|e| crate::error::ReconcilerError::Io(e))?;
            let mappings: Vec<FieldMapping> = serde_json::from_str(&mappings_json)
                .map_err(|e| {
                    eprintln!("\n[ERROR] Fallo al parsear el mapping JSON: {}", path.display());
                    eprintln!("        Linea: {}, Columna: {}", e.line(), e.column());
                    eprintln!("        Formato esperado para 'strategy':");
                    eprintln!("          {{ \"type\": \"Exact\" }}");
                    eprintln!("          {{ \"type\": \"Numeric\", \"epsilon\": 0.0 }}");
                    eprintln!("          {{ \"type\": \"Date\" }}");
                    eprintln!("        (Los campos de struct variant van planos, sin wrapper 'config')\n");
                    crate::error::ReconcilerError::Json(e)
                })?;
            Some(mappings)
        } else {
            None
        };

        Ok(ReconcilerConfig {
            csv_path: self.csv_path.to_string_lossy().to_string(),
            grpc_url: self.grpc_url,
            grpc_function: self.grpc_function,
            chunk_size: self.chunk_size,
            output_dir: self.output_dir.to_string_lossy().to_string(),
            delimiter: self.delimiter.chars().next().unwrap_or(';'),
            skip_header: self.skip_header,
            field_mappings,
            grpc_parametros: self.grpc_parametros,
            debug: self.debug,
        })
    }
}
