use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directiva {
    // Campos llave
    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "cod_grado",
        alias = "codigo_grado"
    )]
    pub grado_id: u32,

    #[serde(default, alias = "descripcion", alias = "nombre")]
    pub grado: String, // Ej: "SARGENTO SEGUNDO"

    // Factores de Cálculo
    #[serde(default, deserialize_with = "deserialize_any_to_u32")]
    pub antiguedad: u32, // Años de servicio necesarios

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "sueldo",
        alias = "monto"
    )]
    pub sueldo_base: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "ut",
        alias = "unidad_tributaria"
    )]
    pub unidad_tributaria: f64,

    // Metadatos
    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "anio",
        alias = "vigencia"
    )]
    pub anio: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concepto {
    pub cedula: String,

    #[serde(alias = "descripcion")]
    pub nombre: String,

    #[serde(default)]
    pub formula: String,

    #[serde(default)]
    pub tipo: String, // Asignación / Deducción

    #[serde(default, alias = "monto")]
    pub valor: f64,

    #[serde(default, alias = "sueldo_base")]
    pub sueldo_base: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base {
    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "cod_grado",
        alias = "codigo_grado"
    )]
    pub grado_id: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "cod_componente",
        alias = "codigo_componente"
    )]
    pub componente_id: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "numero_hijos",
        alias = "cantidad_hijos"
    )]
    pub n_hijos: u32,

    #[serde(default, alias = "f_ingreso")]
    pub fecha_ingreso: Option<String>,

    #[serde(default, alias = "fecha_ultimo_ascenso", alias = "f_ascenso")]
    pub f_ult_ascenso: Option<String>,

    // Reconocimiento
    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "anos_reconocidos",
        alias = "aa_recon"
    )]
    pub anio_reconocido: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "meses_reconocidos",
        alias = "mm_recon"
    )]
    pub mes_reconocido: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "dias_reconocidos",
        alias = "dd_recon"
    )]
    pub dia_reconocido: u32,

    // Estatus
    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "sin_ascenso"
    )]
    pub st_no_ascenso: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_null_as_string",
        alias = "cod_profesion"
    )]
    pub st_profesion: String,

    #[serde(default, alias = "patrones")]
    pub patterns: String,

    // Campos calculados (No obligatorios en JSON)
    #[serde(default, alias = "fecha_retiro")]
    pub f_retiro: Option<String>,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "sueldo",
        alias = "sueldo_mensual"
    )]
    pub sueldo_base: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "p_antiguedad"
    )]
    pub prima_antiguedad: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "p_hijos"
    )]
    pub prima_hijos: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "total_asig"
    )]
    pub total_asignaciones: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "tiempo_servicio"
    )]
    pub antiguedad: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "antiguedad_grado"
    )]
    pub antiguedad_grado: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movimiento {
    pub cedula: String,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "capital_banco",
        alias = "capital"
    )]
    pub cap_banco: f64,

    #[serde(default, deserialize_with = "deserialize_string_to_f64")]
    pub anticipo: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "fecha_capital_banco",
        alias = "f_cap_banco"
    )]
    pub fcap_banco: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "diferencia_asignacion_anticipo"
    )]
    pub dif_asi_anti: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "anticipo_retroactivo",
        alias = "retroactivo"
    )]
    pub anticipor: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "deposito_adicional"
    )]
    pub dep_adicional: f64,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "deposito_garantia"
    )]
    pub dep_garantia: f64,

    #[serde(default, alias = "f_ult_modificacion", alias = "updated_at")]
    pub ultima_modificacion: Option<String>,
}

impl Default for Movimiento {
    fn default() -> Self {
        Movimiento {
            cedula: String::new(),
            cap_banco: 0.0,
            anticipo: 0.0,
            fcap_banco: 0.0,
            dif_asi_anti: 0.0,
            anticipor: 0.0,
            dep_adicional: 0.0,
            dep_garantia: 0.0,
            ultima_modificacion: None,
        }
    }
}

impl Default for Base {
    fn default() -> Self {
        Base {
            grado_id: 0,
            componente_id: 0,
            n_hijos: 0,
            fecha_ingreso: None,
            f_ult_ascenso: None,
            anio_reconocido: 0,
            mes_reconocido: 0,
            dia_reconocido: 0,
            st_no_ascenso: 0,
            st_profesion: String::new(),
            patterns: String::new(),
            f_retiro: None,
            sueldo_base: 0.0,
            prima_antiguedad: 0.0,
            prima_hijos: 0.0,
            total_asignaciones: 0.0,
            antiguedad: 0,
            antiguedad_grado: 0,
        }
    }
}

// Helper para deserializar strings numéricos a f64
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => s.parse::<f64>().map_err(serde::de::Error::custom),
        serde_json::Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("Invalid number")),
        serde_json::Value::Null => Ok(0.0),
        _ => Err(serde::de::Error::custom(
            "Expected string or number for f64",
        )),
    }
}

// Helper para deserializar cualquier cosa a u32
fn deserialize_any_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                Ok(i as u32)
            } else if let Some(f) = n.as_f64() {
                Ok(f as u32)
            } else {
                Err(serde::de::Error::custom("Invalid number for u32"))
            }
        }
        serde_json::Value::String(s) => {
            if let Ok(i) = s.parse::<u32>() {
                Ok(i)
            } else if let Ok(f) = s.parse::<f64>() {
                Ok(f as u32)
            } else {
                Err(serde::de::Error::custom("Invalid string for u32"))
            }
        }
        serde_json::Value::Null => Ok(0),
        _ => Err(serde::de::Error::custom(
            "Expected number, string or null for u32",
        )),
    }
}

// Helper para convertir nulls o numeros a String vacío o texto
fn deserialize_null_as_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Ok(String::new()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beneficiario {
    // Identificación
    #[serde(alias = "id", alias = "cip")]
    pub cedula: String,

    #[serde(default, alias = "nombre_primero", alias = "nombres_completos")]
    pub nombres: String,

    #[serde(default, alias = "apellido_primero", alias = "apellidos_completos")]
    pub apellidos: String,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "cod_componente",
        alias = "codigo_componente"
    )]
    pub componente_id: u32,

    #[serde(default, alias = "f_ingreso_sis", alias = "fecha_ingreso_sistema")]
    pub f_ingreso_sistema: Option<String>,

    #[serde(default, alias = "fecha_ultimo_ascenso", alias = "f_ascenso")]
    pub f_ult_ascenso: Option<String>,

    #[serde(default, alias = "fecha_retiro")]
    pub f_retiro: Option<String>,

    #[serde(default, alias = "fecha_retiro_efectiva")]
    pub f_retiro_efectiva: Option<String>,

    // Datos Personales
    #[serde(default, alias = "estado_civil")]
    pub edo_civil: Option<String>,

    #[serde(default, alias = "genero")]
    pub sexo: Option<String>,

    // Estatus y Control
    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "cod_status",
        alias = "estatus_id"
    )]
    pub status_id: u32,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "sin_ascenso"
    )]
    pub st_no_ascenso: u32,

    #[serde(default, alias = "cat")]
    pub categoria: Option<String>,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "estatus",
        alias = "estado"
    )]
    pub status: u32,

    // Datos Bancarios
    #[serde(default, alias = "cuenta_bancaria", alias = "n_cuenta")]
    pub numero_cuenta: String,

    // Auditoría
    #[serde(default, alias = "fecha_creacion", alias = "created_at")]
    pub f_creacion: Option<String>,

    #[serde(default, alias = "usuario_creacion", alias = "created_by")]
    pub usr_creacion: Option<String>,

    #[serde(default, alias = "fecha_ultima_modificacion", alias = "updated_at")]
    pub f_ult_modificacion: Option<String>,

    #[serde(default, alias = "usuario_modificacion", alias = "updated_by")]
    pub usr_modificacion: Option<String>,

    #[serde(default, alias = "observacion_modificacion")]
    pub observ_ult_modificacion: Option<String>,

    #[serde(default, alias = "motivo_paraliz")]
    pub motivo_paralizacion: Option<String>,

    #[serde(default, alias = "fecha_reincorporacion")]
    pub f_reincorporacion: Option<String>,

    #[serde(default, alias = "patrones")]
    pub patterns: String,

    // Relaciones (Calculados o Cargados aparte)
    #[serde(default)]
    pub base: Base,

    #[serde(default)]
    pub movimientos: Movimiento,
    // Relaciones (Calculados o Cargados aparte)
    #[serde(default)]
    pub asignaciones: Vec<Concepto>,

    #[serde(default)]
    pub deducciones: Vec<Movimiento>,
}
