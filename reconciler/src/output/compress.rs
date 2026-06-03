use std::fs::File;
use std::io::{BufReader, Read, Write};

use crate::error::{ReconcilerError, Result};

static OUTPUT_FILES: &[&str] = &[
    "correctos.csv",
    "rechazos.csv",
    "nuevos.csv",
    "errores.jsonl",
    "pendientes.csv",
    "detalle.txt",
    "postgres_batch.sql",
    "insert_batch.sql",
    "reporte.txt",
    "indice_cedulas.json",
    "cargar_staging.sh",
];

pub fn compress_outputs(out_dir: &str) -> Result<Vec<String>> {
    let mut compressed = Vec::new();

    for filename in OUTPUT_FILES {
        let src = format!("{}/{}", out_dir, filename);
        let dst = format!("{}/{}.zst", out_dir, filename);

        if !std::fs::metadata(&src).map(|m| m.is_file()).unwrap_or(false) {
            continue;
        }

        let input_file = File::open(&src).map_err(|e| {
            ReconcilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("No se pudo abrir '{}' para compresion: {}", src, e),
            ))
        })?;
        let reader = BufReader::new(input_file);
        let output_file = File::create(&dst).map_err(|e| {
            ReconcilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("No se pudo crear '{}': {}", dst, e),
            ))
        })?;
        let mut encoder = zstd::stream::write::Encoder::new(output_file, 3)
            .map_err(|e| ReconcilerError::Io(e))?;

        let mut buffer = Vec::new();
        let mut reader = reader;
        reader.read_to_end(&mut buffer).map_err(|e| {
            ReconcilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error leyendo '{}': {}", src, e),
            ))
        })?;
        encoder.write_all(&buffer).map_err(|e| {
            ReconcilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error comprimiendo '{}': {}", src, e),
            ))
        })?;
        encoder.finish().map_err(|e| ReconcilerError::Io(e))?;

        // Eliminar original post-compresion
        std::fs::remove_file(&src).map_err(|e| {
            ReconcilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("No se pudo eliminar '{}' tras compresion: {}", src, e),
            ))
        })?;

        compressed.push(filename.to_string());
    }

    Ok(compressed)
}

pub fn compress_file(src: &str, dst: &str) -> Result<()> {
    let input_file = File::open(src)?;
    let reader = BufReader::new(input_file);
    let output_file = File::create(dst)?;
    let mut encoder = zstd::stream::write::Encoder::new(output_file, 3)?;

    let mut buffer = Vec::new();
    let mut reader = reader;
    reader.read_to_end(&mut buffer)?;
    encoder.write_all(&buffer)?;
    encoder.finish()?;

    Ok(())
}
