use std::collections::HashMap;

use sandra_core::kernel::sandra::sentinel_dynamic_service_client::SentinelDynamicServiceClient;
use sandra_core::kernel::sandra::DynamicRequest;
use serde::Serialize;

use crate::error::Result;

#[derive(Debug, Clone, Serialize)]
pub struct IndiceEntry {
    pub cedula: String,
    pub casos: String,
    pub linea: String,
    pub estatus: u8,
    pub detalles: HashMap<String, String>,
}

#[derive(Debug)]
pub struct IndiceWriter {
    inner: Vec<IndiceEntry>,
    parametro: String,
    field_names: Vec<String>,
}

impl IndiceWriter {
    pub fn new(parametro: &str, field_names: Vec<String>) -> Self {
        IndiceWriter {
            inner: Vec::new(),
            parametro: parametro.trim_matches('"').to_string(),
            field_names,
        }
    }

    pub fn add(&mut self, cedula: &str, casos: &str, linea: &str, values: &[String]) {
        let mut detalles = HashMap::new();
        for (i, val) in values.iter().enumerate() {
            if let Some(name) = self.field_names.get(i) {
                detalles.insert(name.clone(), val.clone());
            }
        }
        self.inner.push(IndiceEntry {
            cedula: cedula.to_string(),
            casos: casos.to_string(),
            linea: linea.to_string(),
            estatus: 0,
            detalles,
        });
    }

    pub fn write_to_file(&self, path: &str) -> Result<()> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        let sorted = {
            let mut v = self.inner.clone();
            v.sort_by(|a, b| a.cedula.cmp(&b.cedula));
            v
        };
        let output = serde_json::json!({
            "parametro": self.parametro,
            "listado": sorted,
        });
        serde_json::to_writer_pretty(writer, &output)?;
        Ok(())
    }

    pub fn build_payload(&self) -> serde_json::Value {
        let sorted = {
            let mut v = self.inner.clone();
            v.sort_by(|a, b| a.cedula.cmp(&b.cedula));
            v
        };
        serde_json::json!({
            "parametro": self.parametro,
            "listado": sorted,
        })
    }
}

pub async fn send_indice(grpc_url: &str, driver: &str, parametro: &str, objeto: &serde_json::Value) -> Result<()> {
    let param_clean = parametro.trim_matches('"');
    let donde = serde_json::json!({"parametro": param_clean}).to_string();
    let payload = serde_json::json!({
        "coleccion": "file-fideicomitentes-validate",
        "objeto": objeto,
        "donde": donde,
        "driver": driver,
        "upsert": true,
    });

    let mut client = SentinelDynamicServiceClient::connect(grpc_url.to_string())
        .await
        .map_err(|e| crate::error::ReconcilerError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, format!("Error conectando gRPC {}: {}", grpc_url, e))
        ))?
        .max_decoding_message_size(usize::MAX)
        .max_encoding_message_size(usize::MAX);

    let request = tonic::Request::new(DynamicRequest {
        funcion: "ccoleccion".to_string(),
        parametros: param_clean.to_string(),
        valores: payload.to_string(),
    });

    let response = client
        .execute_dynamic(request)
        .await
        .map_err(|e| crate::error::ReconcilerError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, format!("Error enviando indice por gRPC a {}: {}", grpc_url, e))
        ))?;

    let mut stream = response.into_inner();
    while let Ok(Some(_)) = stream.message().await {}

    Ok(())
}
