use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScfError {
    #[error("Error de E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error de parsing: {0}")]
    Parse(String),

    #[error("Columna fuera de rango: {0} (línea {1})")]
    ColumnOutOfBounds(usize, usize),

    #[error("Archivo no encontrado: {0}")]
    FileNotFound(String),

    #[error("Argumento inválido: {0}")]
    InvalidArgument(String),
}

pub type Result<T> = std::result::Result<T, ScfError>;
