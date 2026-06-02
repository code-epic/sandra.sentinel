use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReconcilerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("gRPC error: {0}")]
    Grpc(String),

    #[error("Invalid delimiter")]
    InvalidDelimiter,

    #[error("Mapping error: {0}")]
    Mapping(String),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

pub type Result<T> = std::result::Result<T, ReconcilerError>;
