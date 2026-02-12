use crate::model::componente::Componente;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grado {
    pub id: u32,
    pub codigo: String,
    pub nombre: String,
    pub componente: Componente,
    // Add other fields from legacy MGrado if needed, like 'clasificacion'
}
