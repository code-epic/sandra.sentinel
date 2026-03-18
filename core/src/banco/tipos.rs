use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoArchivo {
    Apertura,
    Aporte,
    Retiro,
    Mixto,
}

impl Default for TipoArchivo {
    fn default() -> Self {
        TipoArchivo::Aporte
    }
}

impl TipoArchivo {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "apertura" => Some(TipoArchivo::Apertura),
            "aporte" => Some(TipoArchivo::Aporte),
            "retiro" => Some(TipoArchivo::Retiro),
            "mixto" => Some(TipoArchivo::Mixto),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TipoArchivo::Apertura => "apertura",
            TipoArchivo::Aporte => "aporte",
            TipoArchivo::Retiro => "retiro",
            TipoArchivo::Mixto => "mixto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Banco {
    Venezuela,
    Banfanb,
    Bicentenario,
    Mercantil,
    Provincial,
    Tesoro,
    Desconocido,
}

impl Default for Banco {
    fn default() -> Self {
        Banco::Desconocido
    }
}

impl Banco {
    pub fn from_codigo(codigo: &str) -> Self {
        match codigo {
            "0102" => Banco::Venezuela,
            "0177" => Banco::Banfanb,
            "0175" => Banco::Bicentenario,
            "0105" => Banco::Mercantil,
            "0108" => Banco::Provincial,
            "0163" => Banco::Tesoro,
            _ => Banco::Desconocido,
        }
    }

    pub fn codigo(&self) -> &'static str {
        match self {
            Banco::Venezuela => "0102",
            Banco::Banfanb => "0177",
            Banco::Bicentenario => "0175",
            Banco::Mercantil => "0105",
            Banco::Provincial => "0108",
            Banco::Tesoro => "0163",
            Banco::Desconocido => "0000",
        }
    }

    pub fn nombre(&self) -> &'static str {
        match self {
            Banco::Venezuela => "Banco de Venezuela",
            Banco::Banfanb => "Banfanb",
            Banco::Bicentenario => "Bicentenario",
            Banco::Mercantil => "Mercantil",
            Banco::Provincial => "Provincial",
            Banco::Tesoro => "Banco del Tesoro",
            Banco::Desconocido => "Desconocido",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampoBanco {
    pub plan: String,
    pub nac: String,
    pub cedula: String,
    pub nombre: Option<String>,
    pub edocivil: Option<String>,
    pub monto: f64,
    pub tiptrn: Option<String>,
    pub tippre: Option<String>,
    pub frmpgo: Option<String>,
    pub tippta: Option<String>,
    pub tipcue: Option<String>,
    pub numcue: Option<String>,
    pub tasaint: Option<String>,
    pub cbrintatp: Option<String>,
    pub cuomen: Option<String>,
    pub mtoanu: Option<String>,
    pub cuoanu: Option<String>,
}

impl Default for CampoBanco {
    fn default() -> Self {
        CampoBanco {
            plan: "03487".to_string(),
            nac: "V".to_string(),
            cedula: "000000000".to_string(),
            nombre: None,
            edocivil: None,
            monto: 0.0,
            tiptrn: Some("1".to_string()),
            tippre: Some("00".to_string()),
            frmpgo: Some("0".to_string()),
            tippta: Some("N".to_string()),
            tipcue: Some("0".to_string()),
            numcue: Some("0000000000".to_string()),
            tasaint: Some("000000".to_string()),
            cbrintatp: Some(" ".to_string()),
            cuomen: Some("000".to_string()),
            mtoanu: Some("0000000000000".to_string()),
            cuoanu: Some("000".to_string()),
        }
    }
}
