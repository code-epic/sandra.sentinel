use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifiesto {
    pub nombre: String,
    pub ciclo: String,

    #[serde(default)]
    pub descripcion: String,

    #[serde(default = "default_autor")]
    pub autor: String,

    #[serde(default = "default_fecha")]
    pub fecha: String,

    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub parametros_globales: HashMap<String, String>,
    #[serde(default)]
    pub cargas: HashMap<String, CargaConfig>,
}

fn default_autor() -> String {
    "Sistema".to_string()
}
fn default_fecha() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
fn default_version() -> String {
    "1.0.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CargaConfig {
    pub sql_filter: Option<String>, // WHERE clause o similar para SQL
    pub limit: Option<u32>,         // Límite de registros
    pub parametros_extra: Option<String>, // JSON string extra si se requiere
}

impl Manifiesto {
    pub fn cargar_desde_archivo(ruta: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contenido = fs::read_to_string(ruta)?;
        let manifiesto: Manifiesto = serde_json::from_str(&contenido)?;
        Ok(manifiesto)
    }

    pub fn default_mock() -> Self {
        let mut cargas: HashMap<String, CargaConfig> = HashMap::new();

        // Configuración por defecto segura (Directiva 81 / Activos 201)
        cargas.insert(
            "IPSFA_CPrimasFunciones".to_string(),
            CargaConfig {
                sql_filter: Some("f.oidd = 81".to_string()),
                limit: None,
                parametros_extra: None,
            },
        );
        cargas.insert(
            "IPSFA_CDirectiva".to_string(),
            CargaConfig {
                sql_filter: Some("dd.directiva_sueldo_id = 81 and dd.sueldo_base > 0".to_string()),
                limit: None,
                parametros_extra: None,
            },
        );
        cargas.insert(
            "IPSFA_CConceptos".to_string(),
            CargaConfig {
                sql_filter: Some("directiva_sueldo_id = 81".to_string()),
                limit: None,
                parametros_extra: None,
            },
        );
        cargas.insert(
            "IPSFA_CBase".to_string(),
            CargaConfig {
                sql_filter: Some("status_id = 201".to_string()),
                limit: None,
                parametros_extra: None,
            },
        );
        cargas.insert(
            "IPSFA_CBeneficiarios".to_string(),
            CargaConfig {
                sql_filter: Some("bnf.status_id = 201".to_string()),
                limit: None,
                parametros_extra: None,
            },
        );

        Manifiesto {
            nombre: "Ejecución Default (Segura)".to_string(),
            ciclo: "DEFAULT_2026".to_string(),
            descripcion: "Ejecución automática con filtros de seguridad (Directiva 81 / Activos)"
                .to_string(),
            autor: "Sistema (Default)".to_string(),
            fecha: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.0".to_string(),
            parametros_globales: HashMap::new(),
            cargas,
        }
    }
}

impl Default for Manifiesto {
    fn default() -> Self {
        Self::default_mock()
    }
}
