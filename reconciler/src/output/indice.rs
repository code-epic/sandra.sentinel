use std::collections::HashMap;

use sandra_core::kernel::sandra::sentinel_dynamic_service_client::SentinelDynamicServiceClient;
use sandra_core::kernel::sandra::DynamicRequest;
use serde::Serialize;

use crate::error::Result;
use crate::types::{es_campo_fecha, normalizar_fecha, Categoria, FieldDiff};

/// Motivo exacto de un rechazo, con la misma semántica que `detalle.txt`.
#[derive(Debug, Clone, Serialize)]
pub struct Motivo {
    /// Categorías deduplicadas en orden canónico de la UI: `FECHAS`,
    /// `GRADOS`, `TIEMPO_RECONOCIDO`, `FAMILIA`, `PROFESION`, `OTROS`.
    /// Dicho orden es el de declaración de [`Categoria`] y su `Ord` derivado.
    pub categorias: Vec<Categoria>,
    /// Un elemento por cada `FieldDiff`, conservando el orden de
    /// `field_mappings`.
    pub campos: Vec<MotivoCampo>,
}

impl Motivo {
    /// Construye el motivo a partir de los diffs. Devuelve `None` cuando no
    /// hay diferencias (no se debe serializar `motivo`).
    pub fn from_diffs(diffs: &[FieldDiff]) -> Option<Self> {
        if diffs.is_empty() {
            return None;
        }

        let mut categorias: Vec<Categoria> = diffs.iter().map(|d| d.categoria()).collect();
        categorias.sort();
        categorias.dedup();

        let campos = diffs
            .iter()
            .map(|d| MotivoCampo {
                campo: d.field_name.clone(),
                // Las fechas se emiten igual que la línea CSV (`YYYY-MM-DD`),
                // no como el timestamp ISO que devuelve gRPC.
                valor_csv: if es_campo_fecha(&d.field_name) {
                    normalizar_fecha(&d.expected)
                } else {
                    d.expected.clone()
                },
                valor_sssifanb: if es_campo_fecha(&d.field_name) {
                    normalizar_fecha(&d.actual)
                } else {
                    d.actual.clone()
                },
            })
            .collect();

        Some(Motivo {
            categorias,
            campos,
        })
    }
}

/// Detalle de un campo que difiere: `valor_csv` = `FieldDiff.expected`,
/// `valor_sssifanb` = `FieldDiff.actual`.
#[derive(Debug, Clone, Serialize)]
pub struct MotivoCampo {
    pub campo: String,
    pub valor_csv: String,
    pub valor_sssifanb: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndiceEntry {
    pub cedula: String,
    pub casos: String,
    pub linea: String,
    /// 0 = rechazo (`PartialMatch`). No cambia de tipo por compatibilidad.
    pub estatus: u8,
    pub detalles: HashMap<String, String>,
    /// Solo presente en rechazos con diferencias.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motivo: Option<Motivo>,
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
        self.push_entry(cedula, casos, linea, values, None);
    }

    /// Agrega un rechazo incluyendo el motivo exacto (`diffs`). Si `diffs`
    /// está vacío, el campo `motivo` no se serializa.
    pub fn add_rechazo(
        &mut self,
        cedula: &str,
        linea: &str,
        values: &[String],
        diffs: &[FieldDiff],
    ) {
        let motivo = Motivo::from_diffs(diffs);
        self.push_entry(cedula, "rechazos", linea, values, motivo);
    }

    fn push_entry(
        &mut self,
        cedula: &str,
        casos: &str,
        linea: &str,
        values: &[String],
        motivo: Option<Motivo>,
    ) {
        let mut detalles = HashMap::new();
        for (i, val) in values.iter().enumerate() {
            if let Some(name) = self.field_names.get(i) {
                // Las fechas se persisten en `YYYY-MM-DD`; el resto de campos
                // conserva el valor tal cual llega de gRPC.
                let value = if es_campo_fecha(name) {
                    normalizar_fecha(val)
                } else {
                    val.clone()
                };
                detalles.insert(name.clone(), value);
            }
        }
        self.inner.push(IndiceEntry {
            cedula: cedula.to_string(),
            casos: casos.to_string(),
            linea: linea.to_string(),
            estatus: 0,
            detalles,
            motivo,
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

pub async fn send_indice(
    grpc_url: &str,
    driver: &str,
    parametro: &str,
    objeto: &serde_json::Value,
) -> Result<()> {
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
        .map_err(|e| {
            crate::error::ReconcilerError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error conectando gRPC {}: {}", grpc_url, e),
            ))
        })?
        .max_decoding_message_size(usize::MAX)
        .max_encoding_message_size(usize::MAX);

    let request = tonic::Request::new(DynamicRequest {
        funcion: "ccoleccion".to_string(),
        parametros: param_clean.to_string(),
        valores: payload.to_string(),
    });

    let response = client.execute_dynamic(request).await.map_err(|e| {
        crate::error::ReconcilerError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error enviando indice por gRPC a {}: {}", grpc_url, e),
        ))
    })?;

    let mut stream = response.into_inner();
    while let Ok(Some(_)) = stream.message().await {}

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CATALOGO_POR_ESTATUS, CATALOGO_UI};
    use serde_json::json;

    fn field_names() -> Vec<String> {
        crate::types::default_field_mappings()
            .into_iter()
            .map(|m| m.field_name)
            .collect()
    }

    fn valores() -> Vec<String> {
        vec![
            "30",                   // grado (gRPC)
            "4",                    // n_hijos
            "1996-07-05T00:00:00Z", // f_ingreso
            "2023-07-05T00:00:00Z", // f_ult_ascenso
            "35",                   // st_profesion
            "0",                    // anio_reconocido
            "0",                    // mes_reconocido
            "0",                    // dia_reconocido
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn diff(field: &str, csv: &str, grpc: &str) -> FieldDiff {
        FieldDiff {
            field_name: field.to_string(),
            expected: csv.to_string(),
            actual: grpc.to_string(),
        }
    }

    /// Caso real cédula 11439805: solo difiere GRADOS (CSV=1040, SSSIFANB=30).
    #[test]
    fn test_rechazo_solo_grado_expone_motivo() {
        let linea = "11439805;1040;4;1996-07-05;2023-07-05;35;0;0;0";
        let mut writer = IndiceWriter::new("\"3\"", field_names());
        writer.add_rechazo(
            "11439805",
            linea,
            &valores(),
            &[diff("grado", "1040", "30")],
        );

        let payload = writer.build_payload();

        // Contrato hacia atrás intacto.
        assert_eq!(payload["parametro"], "3");
        assert_eq!(payload["listado"].as_array().unwrap().len(), 1);

        let entry = &payload["listado"][0];
        assert_eq!(entry["cedula"], "11439805");
        assert_eq!(entry["casos"], "rechazos");
        assert_eq!(entry["linea"], linea);
        assert_eq!(entry["estatus"], json!(0));
        assert_eq!(entry["detalles"]["grado"], "30");

        let motivo = &entry["motivo"];
        assert_eq!(motivo["categorias"], json!(["GRADOS"]));
        assert!(motivo.get("grupo_validacion").is_none());
        assert!(motivo.get("resumen").is_none());

        let campo = &motivo["campos"][0];
        assert_eq!(campo["campo"], "grado");
        assert_eq!(campo["valor_csv"], "1040");
        assert_eq!(campo["valor_sssifanb"], "30");
        assert!(campo.get("categoria").is_none());
        assert!(campo.get("columna_csv").is_none());
        assert!(campo.get("valor_grpc").is_none());
    }

    /// Múltiples categorías: se deduplican y ordenan según el catálogo de la
    /// UI (`FECHAS, GRADOS, TIEMPO_RECONOCIDO, FAMILIA, PROFESION, OTROS`),
    /// mientras `campos` conserva el orden de los diffs.
    #[test]
    fn test_rechazo_multicategoria_ordena_y_deduplica() {
        let mut writer = IndiceWriter::new("3", field_names());
        let diffs = vec![
            diff("f_ingreso", "1996-07-05", "1997-01-01T00:00:00Z"),
            diff("n_hijos", "4", "3"),
            diff("grado", "1040", "30"),
            // Segunda diferencia de FECHAS: no debe duplicar la categoría.
            diff("f_ult_ascenso", "2023-07-05", "2022-01-01T00:00:00Z"),
            diff("anio_reconocido", "0", "2"),
        ];
        writer.add_rechazo("12345678", "12345678;...", &valores(), &diffs);

        let motivo = &writer.build_payload()["listado"][0]["motivo"];

        assert_eq!(
            motivo["categorias"],
            json!(["FECHAS", "GRADOS", "TIEMPO_RECONOCIDO", "FAMILIA"])
        );

        let campos = motivo["campos"].as_array().unwrap();
        assert_eq!(campos.len(), 5);
        // `campos` respeta el orden de entrada (field_mappings / diffs).
        assert_eq!(campos[0]["campo"], "f_ingreso");
        assert_eq!(campos[1]["campo"], "n_hijos");
        assert_eq!(campos[2]["campo"], "grado");
        assert_eq!(campos[3]["campo"], "f_ult_ascenso");
        assert_eq!(campos[4]["campo"], "anio_reconocido");
    }

    /// Sin diffs no se serializa `motivo`.
    #[test]
    fn test_rechazo_sin_diffs_no_serializa_motivo() {
        let mut writer = IndiceWriter::new("3", field_names());
        writer.add_rechazo("12345678", "linea", &valores(), &[]);

        let entry = &writer.build_payload()["listado"][0];
        assert!(entry.get("motivo").is_none());
    }

    /// Los casos distintos de "rechazos" conservan el esquema legacy sin
    /// `motivo`.
    #[test]
    fn test_add_no_rechazo_mantiene_esquema_legacy() {
        let mut writer = IndiceWriter::new("\"3\"", field_names());
        writer.add("12345678", "correctos", "linea", &valores());

        let payload = writer.build_payload();
        assert_eq!(payload["parametro"], "3");
        assert_eq!(payload["listado"].as_array().unwrap().len(), 1);

        let entry = &payload["listado"][0];
        for key in ["cedula", "casos", "linea", "estatus", "detalles"] {
            assert!(entry.get(key).is_some(), "falta la clave {}", key);
        }
        assert!(entry.get("motivo").is_none());
    }

    /// La clasificación compartida cubre todos los campos conocidos con los
    /// IDs exactos de la UI.
    #[test]
    fn test_categoria_mapea_campos_conocidos() {
        assert_eq!(Categoria::from_field_name("grado"), Categoria::Grados);
        assert_eq!(Categoria::from_field_name("grado_id"), Categoria::Grados);
        assert_eq!(Categoria::from_field_name("n_hijos"), Categoria::Familia);
        assert_eq!(Categoria::from_field_name("f_ingreso"), Categoria::Fechas);
        assert_eq!(
            Categoria::from_field_name("f_ult_ascenso"),
            Categoria::Fechas
        );
        assert_eq!(
            Categoria::from_field_name("fecha_ingreso"),
            Categoria::Fechas
        );
        assert_eq!(
            Categoria::from_field_name("anio_reconocido"),
            Categoria::TiempoReconocido
        );
        assert_eq!(
            Categoria::from_field_name("mes_reconocido"),
            Categoria::TiempoReconocido
        );
        assert_eq!(
            Categoria::from_field_name("dia_reconocido"),
            Categoria::TiempoReconocido
        );
        assert_eq!(
            Categoria::from_field_name("st_profesion"),
            Categoria::Profesion
        );
        assert_eq!(Categoria::from_field_name("otro_campo"), Categoria::Otros);

        assert_eq!(Categoria::Grados.as_str(), "GRADOS");
        assert_eq!(Categoria::Fechas.as_str(), "FECHAS");
        assert_eq!(Categoria::TiempoReconocido.as_str(), "TIEMPO_RECONOCIDO");
    }

    /// Guardián del catálogo: todo `Categoria::as_str()` pertenece al
    /// catálogo de la UI y su serialización serde coincide exactamente.
    /// Las categorías por estatus están documentadas y NO son producibles
    /// por `from_field_name`.
    #[test]
    fn test_catalogo_ui_y_categorias_por_estatus() {
        assert_eq!(CATALOGO_UI.len(), 11);
        assert_eq!(CATALOGO_POR_ESTATUS.len(), 5);

        let categorias = [
            Categoria::Fechas,
            Categoria::Grados,
            Categoria::TiempoReconocido,
            Categoria::Familia,
            Categoria::Profesion,
            Categoria::Otros,
        ];
        for categoria in categorias {
            let id = categoria.as_str();
            assert!(
                CATALOGO_UI.contains(&id),
                "{} no está en el catálogo de la UI",
                id
            );
            // El JSON debe ser idéntico a `as_str` (sin transformación en TS).
            assert_eq!(
                serde_json::to_string(&categoria).unwrap(),
                format!("\"{}\"", id)
            );
        }

        // Las categorías por estatus vienen de `LISTAR_TMP_COMPONENTE`; el
        // reconciler nunca las emite.
        for id in CATALOGO_POR_ESTATUS {
            assert!(CATALOGO_UI.contains(&id), "{} falta en CATALOGO_UI", id);
            assert_ne!(
                Categoria::from_field_name(id).as_str(),
                id,
                "{} no debe ser producible por from_field_name",
                id
            );
        }
    }

    /// Caso real cédula 11439805: `detalles` persiste las fechas igual que la
    /// línea CSV (`YYYY-MM-DD`), no como timestamp ISO de gRPC.
    #[test]
    fn test_detalles_fechas_normalizadas_sin_timestamp() {
        let linea = "11439805;1040;4;1996-07-05;2023-07-05;35;0;0;0";
        let mut writer = IndiceWriter::new("3", field_names());
        writer.add_rechazo(
            "11439805",
            linea,
            &valores(),
            &[diff("grado", "1040", "30")],
        );

        let payload = writer.build_payload();
        let entry = &payload["listado"][0];

        // Fechas homologadas a YYYY-MM-DD, sin `T`.
        assert_eq!(entry["detalles"]["f_ingreso"], "1996-07-05");
        assert_eq!(entry["detalles"]["f_ult_ascenso"], "2023-07-05");
        assert!(!entry["detalles"]["f_ingreso"]
            .as_str()
            .unwrap()
            .contains('T'));
        assert!(!entry["detalles"]["f_ult_ascenso"]
            .as_str()
            .unwrap()
            .contains('T'));

        // El resto de campos conserva el valor gRPC crudo.
        assert_eq!(entry["detalles"]["grado"], "30");
        assert_eq!(entry["detalles"]["n_hijos"], "4");
        assert_eq!(entry["detalles"]["st_profesion"], "35");

        // `linea` ya viene en YYYY-MM-DD y no se toca.
        assert_eq!(entry["linea"], linea);

        // Contrato del motivo intacto.
        assert_eq!(entry["motivo"]["categorias"], json!(["GRADOS"]));
    }

    /// `motivo.campos[]` normaliza las fechas en `valor_csv` y `valor_sssifanb`.
    #[test]
    fn test_motivo_campos_fecha_normalizados() {
        let mut writer = IndiceWriter::new("3", field_names());
        writer.add_rechazo(
            "12345678",
            "12345678;1040;4;1996-07-05;2023-07-05;35;0;0;0",
            &valores(),
            &[diff("f_ingreso", "1996-07-05", "1997-01-01T00:00:00Z")],
        );

        let campo = &writer.build_payload()["listado"][0]["motivo"]["campos"][0];
        assert_eq!(campo["campo"], "f_ingreso");
        assert_eq!(campo["valor_csv"], "1996-07-05");
        assert_eq!(campo["valor_sssifanb"], "1997-01-01");
        assert!(!campo["valor_sssifanb"].as_str().unwrap().contains('T'));
    }

    /// Los campos no fecha de `motivo.campos[]` no se transforman.
    #[test]
    fn test_motivo_campos_no_fecha_intactos() {
        let mut writer = IndiceWriter::new("3", field_names());
        writer.add_rechazo(
            "12345678",
            "linea",
            &valores(),
            &[diff("grado", "1040", "30")],
        );

        let campo = &writer.build_payload()["listado"][0]["motivo"]["campos"][0];
        assert_eq!(campo["valor_csv"], "1040");
        assert_eq!(campo["valor_sssifanb"], "30");
    }
}
