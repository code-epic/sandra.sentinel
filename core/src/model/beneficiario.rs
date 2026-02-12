use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::componente::Componente;
use crate::model::grado::Grado;

// Estatus extendido basado en reglas de negocio (PACE)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Estatus {
    Activo = 201,
    Retirado = 202,
    Fallecido = 203,
    RetiroConPension = 204,
    RetiroSinPension = 205,
    Paralizado = 206, // Estado especial para control de pagos
                      // Otros códigos deben mapearse aquí
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sexo {
    Mm, // Masculino (Legacy M)
    Ff, // Femenino (Legacy F)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoCivil {
    Soltero,
    Casado,
    Divorciado,
    Viudo,
    Concubino,
    Otro,
}

/// Agrupa los tiempos de servicio y reconocimientos legales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HojaDeTiempo {
    // Fechas Críticas
    pub fecha_ingreso: NaiveDate,
    pub fecha_ingreso_sistema: NaiveDate,
    pub fecha_ultimo_ascenso: NaiveDate,
    pub fecha_retiro: Option<NaiveDate>,
    pub fecha_retiro_efectiva: Option<NaiveDate>,
    pub fecha_reincorporacion: Option<NaiveDate>,

    // Tiempos Reconocidos (Servicio previo fuera del componente)
    pub anos_reconocidos: u32,
    pub meses_reconocidos: u32,
    pub dias_reconocidos: u32,

    // Tiempos Calculados (Resultados del Kernel)
    pub tiempo_servicio: u32,  // Años efectivos para cálculo
    pub antiguedad_grado: u32, // Años en el grado actual
}

/// Agrupa la información bancaria y variables monetarias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HojaFinanciera {
    // Banca
    pub numero_cuenta: String,

    // Variables de Sueldo (Caché del cálculo)
    pub sueldo_base: Decimal,
    pub sueldo_global: Decimal,
    pub sueldo_integral: Decimal,

    // Alicuotas y Bonos calculados
    pub aguinaldos: Decimal,
    pub vacaciones: Decimal,
    pub prima_t_servicio: Decimal, // Prima Tiempo Servicio
    pub prima_no_ascenso: Decimal,
    pub prima_especial: Decimal,
    pub prima_profesionalizacion: Decimal,

    // Asignación de Antigüedad (La "Prestación")
    pub asignacion_antiguedad: Decimal,

    // Control de Pagos
    pub no_depositado_banco: Decimal, // Deudas
}

/// Representa al Beneficiario Principal (Afiliado/Militar)
/// Mapeo directo y mejorado de MBeneficiario.php
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beneficiario {
    // Identidad
    pub id: String, // UUID o Cédula como key
    pub cedula: String,
    pub nombres: String,
    pub apellidos: String,
    pub sexo: Sexo,
    pub estado_civil: EstadoCivil,
    pub numero_hijos: u32,

    // Carrera Militar
    pub componente: Componente,
    pub grado: Grado, // Contiene ID y Código
    pub estatus: Estatus,
    pub estatus_descripcion: Option<String>,

    // Flags de Carrera
    pub st_no_ascenso: bool,         // ¿Tiene congelado el ascenso?
    pub st_profesionalizacion: bool, // ¿Cobra prima profesional?

    // Módulos de Datos
    pub tiempo: HojaDeTiempo,
    pub financiera: HojaFinanciera,

    // Meta-datos de Auditoría
    pub usuario_creador: String,
    pub fecha_creacion: NaiveDate,
    pub usuario_modificacion: Option<String>,
    pub fecha_ultima_modificacion: Option<NaiveDate>,
    pub observacion: Option<String>,
    pub motivo_paralizacion: Option<String>,
    // Relaciones (Placeholder para futuros Vectores)
    // pub historial_movimientos: Vec<Movimiento>,
    // pub historial_sueldos: Vec<HistorialSueldo>,
    // pub medidas_judiciales: Vec<MedidaJudicial>,
}

impl Beneficiario {
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombres, self.apellidos)
    }

    /// Determina si el beneficiario está activo para efectos de nómina
    pub fn en_nomina(&self) -> bool {
        match self.estatus {
            Estatus::Activo | Estatus::Paralizado => true,
            _ => false,
        }
    }
}
