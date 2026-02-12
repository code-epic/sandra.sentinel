use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Componente {
    Ejercito = 1,
    Armada = 2,
    Aviacion = 3,
    GuardiaNacional = 4,
    Milicia = 5,
}

impl Componente {
    pub fn description(&self) -> &str {
        match self {
            Componente::Ejercito => "Ejército Bolivariano",
            Componente::Armada => "Armada Bolivariana",
            Componente::Aviacion => "Aviación Militar Bolivariana",
            Componente::GuardiaNacional => "Guardia Nacional Bolivariana",
            Componente::Milicia => "Milicia Bolivariana",
        }
    }
}
