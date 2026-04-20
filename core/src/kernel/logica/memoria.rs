use serde::{Deserialize, Serialize};

// =============================================================================
// ESTRUCTURA PARA NOMINA PATRIA - FINIQUITOS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiniquitoPatria {
    /// Cédula del beneficiario (sin prefijo V/E)
    pub cedula: String,

    /// Apellidos y nombres del beneficiario
    pub apellidos: String,

    /// Número de cuenta bancaria (20 dígitos, debe empezar con 0102)
    pub numero_cuenta: String,

    /// Monto del finiquito (soporta string o number)
    #[serde(default, deserialize_with = "deserialize_string_to_f64")]
    pub monto: f64,

    /// Fecha contable del movimiento
    pub f_contable: String,

    /// Observaciones del movimiento (causal)
    pub observaciones: String,
}

/// Transformación al formato TXT del Sistema Patria
impl FiniquitoPatria {
    /// Convierte la estructura al formato de línea TXT para Patria
    ///
    /// Formato detalle (80 columnas):
    /// - Columnas 1-1: Letra de cédula (V o E)
    /// - Columnas 2-9: Cédula (8 dígitos, padded)
    /// - Columnas 10-29: Número de cuenta (20 dígitos)
    /// - Columnas 30-40: Monto (11 dígitos, 9 enteros + 2 decimales)
    /// - Columnas 41-80: Nombre (40 caracteres)
    pub fn to_line_patria(&self) -> String {
        // Campo 1: Letra de cédula
        let letra_cedula = if self.cedula.starts_with('V') || self.cedula.starts_with('E') {
            &self.cedula[0..1]
        } else {
            "V"
        };

        // Campo 2: Cédula padded a 8 dígitos
        let cedula_clean: String = self.cedula.chars().filter(|c| c.is_ascii_digit()).collect();
        let cedula_padded = format!("{:0>8}", cedula_clean);

        // Campo 3: Número de cuenta (20 dígitos)
        let cuenta_clean: String = self
            .numero_cuenta
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let cuenta_padded = format!("{:0<20}", cuenta_clean);

        // Campo 4: Monto (11 dígitos: 9 enteros + 2 decimales)
        let monto_centavos = (self.monto * 100.0).round() as i64;
        let monto_padded = format!("{:0>11}", monto_centavos);

        // Campo 5: Nombre (40 caracteres, padded con espacios)
        let nombre_padded = format!("{: <40}", self.apellidos);

        // Línea completa (80 columnas)
        format!(
            "{}{}{}{}{}",
            letra_cedula, cedula_padded, cuenta_padded, monto_padded, nombre_padded
        )
    }

    /// Valida que el registro cumpla las reglas para Patria
    pub fn es_valido(&self) -> bool {
        // Regla 1: Cuenta debe empezar con 0102
        let cuenta_ok = self.numero_cuenta.starts_with("0102");

        // Regla 2: Monto positivo
        let monto_ok = self.monto > 0.0;

        // Regla 3: Cédula válida
        let cedula_ok = !self.cedula.is_empty() && self.cedula.len() >= 7;

        cuenta_ok && monto_ok && cedula_ok
    }
}

// =============================================================================

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

    #[serde(default, deserialize_with = "deserialize_any_to_string")]
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
        deserialize_with = "deserialize_string_to_f64",
        alias = "st_profesion"
    )]
    pub st_profesion: f64,

    #[serde(default, alias = "patrones")]
    pub patterns: String,

    // Campos calculados (No obligatorios en JSON)
    #[serde(default, alias = "fecha_retiro")]
    pub f_retiro: Option<String>,

    #[serde(
        default,
        deserialize_with = "deserialize_string_to_f64",
        alias = "sueldo"
    )]
    pub sueldo_base: f64,

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

    // ALMACENAMIENTO DINÁMICO DE PRIMAS CALCULADAS
    #[serde(skip_deserializing)]
    pub calculos: Option<std::collections::HashMap<String, f64>>,

    // Campos calculados de nómina (PHP: KCalculoLote)
    #[serde(default)]
    pub sueldo_mensual: f64,

    #[serde(default)]
    pub aguinaldos: f64,

    #[serde(default)]
    pub vacaciones: f64,

    #[serde(default)]
    pub dia_vacaciones: u32,

    #[serde(default)]
    pub sueldo_integral: f64,

    #[serde(default)]
    pub asignacion_antiguedad: f64,

    #[serde(default)]
    pub garantias: f64,

    #[serde(default)]
    pub dias_adicionales: f64,

    #[serde(default)]
    pub no_depositado_banco: f64,

    // Campos de movimientos (para cálculos)
    #[serde(default)]
    pub deposito_banco: f64,

    // Campos de anticipo de garantías (distribución)
    #[serde(default)]
    pub garantia_original: f64,

    #[serde(default)]
    pub garantia_anticipo: f64,

    #[serde(default)]
    pub factor_aplicado: f64,
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
            st_profesion: 0.0,
            patterns: String::new(),
            f_retiro: None,
            sueldo_base: 0.0,
            total_asignaciones: 0.0,
            antiguedad: 0,
            antiguedad_grado: 0,
            calculos: None,
            sueldo_mensual: 0.0,
            aguinaldos: 0.0,
            vacaciones: 0.0,
            dia_vacaciones: 0,
            sueldo_integral: 0.0,
            asignacion_antiguedad: 0.0,
            garantias: 0.0,
            dias_adicionales: 0.0,
            no_depositado_banco: 0.0,
            deposito_banco: 0.0,
            garantia_original: 0.0,
            garantia_anticipo: 0.0,
            factor_aplicado: 0.0,
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

// Helper para deserializar cualquier cosa a String
fn deserialize_any_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(serde::de::Error::custom("Expected string, number or null")),
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

    #[serde(default)]
    pub neto: f64,

    #[serde(
        default,
        alias = "porcentaje",
        deserialize_with = "deserialize_string_to_f64"
    )]
    pub porcentaje: f64,

    #[serde(default)]
    pub conceptos_calculados: Option<std::collections::HashMap<String, ConceptoCalculado>>,

    #[serde(default)]
    pub total_asignaciones: f64,

    #[serde(default)]
    pub total_deducciones: f64,

    #[serde(default)]
    pub es_familiar: bool,

    #[serde(default, alias = "cedula_titular", alias = "titular")]
    pub cedula_titular: Option<String>,

    #[serde(default, alias = "parentesco")]
    pub parentesco: Option<String>,

    #[serde(default, alias = "nombre_autorizado", alias = "autorizado")]
    pub nombre_autorizado: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrimaFuncion {
    #[serde(alias = "codigo")]
    pub codigo: String,

    #[serde(alias = "nombre")]
    pub nombre: String,

    #[serde(alias = "descripcion")]
    pub descripcion: String,

    #[serde(default)]
    pub formula: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoConcepto {
    Asignacion,
    Deduccion,
}

impl From<String> for TipoConcepto {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "asignacion" | "asignación" | "1" | "asig" => TipoConcepto::Asignacion,
            "deduccion" | "deducción" | "2" | "3" | "ded" => TipoConcepto::Deduccion,
            _ => TipoConcepto::Asignacion,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptoCalculado {
    pub codigo: String,
    pub descripcion: String,
    pub tipo: TipoConcepto,
    pub valor: f64,
    pub estructura: String,
    pub cuenta: String,
    pub partida: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptoNomina {
    #[serde(default, deserialize_with = "deserialize_any_to_string")]
    pub codigo: String,

    #[serde(default, deserialize_with = "deserialize_any_to_string")]
    pub descripcion: String,

    #[serde(
        default,
        alias = "forumula",
        alias = "formula",
        deserialize_with = "deserialize_any_to_string"
    )]
    pub codigo_rhai: String,

    #[serde(default, deserialize_with = "deserialize_any_to_string")]
    pub estructura: String,

    #[serde(default, deserialize_with = "deserialize_any_to_string")]
    pub cuenta: String,

    #[serde(default, deserialize_with = "deserialize_any_to_string")]
    pub partida: String,

    #[serde(
        default,
        deserialize_with = "deserialize_any_to_u32",
        alias = "tipo",
        alias = "TIPO"
    )]
    pub tipo: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Familiar {
    #[serde(alias = "titular", alias = "cedula_titular")]
    pub titular: String,

    #[serde(alias = "cedula", alias = "id")]
    pub cedula: String,

    #[serde(default, alias = "nombres", alias = "nombre")]
    pub nombres: String,

    #[serde(default, alias = "apellidos")]
    pub apellidos: String,

    #[serde(default, alias = "sexo")]
    pub sexo: Option<String>,

    #[serde(default, alias = "f_nacimiento")]
    pub f_nacimiento: Option<String>,

    #[serde(default, alias = "parentesco")]
    pub parentesco: Option<String>,

    #[serde(default, alias = "edo_civil")]
    pub edo_civil: Option<String>,

    #[serde(default, alias = "f_defuncion")]
    pub f_defuncion: Option<String>,

    #[serde(default, alias = "autorizado")]
    pub autorizado: Option<String>,

    #[serde(default, alias = "tipo")]
    pub tipo: Option<String>,

    #[serde(default, alias = "banco")]
    pub banco: Option<String>,

    #[serde(default, alias = "numero", alias = "numero_cuenta")]
    pub numero_cuenta: Option<String>,

    #[serde(default, alias = "situacion")]
    pub situacion: Option<String>,

    #[serde(default, alias = "estatus")]
    pub estatus: Option<i32>,

    #[serde(default, alias = "porcentaje", alias = "pct")]
    pub porcentaje: f64,

    #[serde(default, alias = "nombre_autorizado")]
    pub nombre_autorizado: Option<String>,
}
