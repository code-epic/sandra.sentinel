use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoNomina {
    #[serde(rename = "npr")]
    Npr,

    #[serde(rename = "nact")]
    Nact,

    #[serde(rename = "nrcp")]
    Nrcp,

    #[serde(rename = "nfcp")]
    Nfcp,
}

impl Default for TipoNomina {
    fn default() -> Self {
        TipoNomina::Npr
    }
}

impl TipoNomina {
    pub fn descripcion(&self) -> &'static str {
        match self {
            TipoNomina::Npr => "Nómina de Prestaciones",
            TipoNomina::Nact => "Nómina de Activos",
            TipoNomina::Nrcp => "Nómina de Retirados con Pensión",
            TipoNomina::Nfcp => "Nómina de Fallecidos con Pensión",
        }
    }

    pub fn es_titular(&self) -> bool {
        match self {
            TipoNomina::Npr | TipoNomina::Nact | TipoNomina::Nrcp => true,
            TipoNomina::Nfcp => false,
        }
    }

    pub fn usa_porcentaje(&self) -> bool {
        match self {
            TipoNomina::Nrcp | TipoNomina::Nfcp => true,
            TipoNomina::Npr | TipoNomina::Nact => false,
        }
    }

    pub fn nombre_archivo(&self, ciclo: &str) -> String {
        format!(
            "nomina_{}_{}.csv",
            match self {
                TipoNomina::Npr => "npr",
                TipoNomina::Nact => "nact",
                TipoNomina::Nrcp => "nrcp",
                TipoNomina::Nfcp => "nfcp",
            },
            ciclo
        )
    }
}

impl std::fmt::Display for TipoNomina {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoNomina::Npr => write!(f, "NPR"),
            TipoNomina::Nact => write!(f, "NACT"),
            TipoNomina::Nrcp => write!(f, "NRCP"),
            TipoNomina::Nfcp => write!(f, "NFCP"),
        }
    }
}
