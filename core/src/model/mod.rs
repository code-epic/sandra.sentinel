pub mod beneficiario;
pub mod componente;
pub mod grado;

pub mod manifiesto;

// Re-export common types for easier access
pub use beneficiario::{Beneficiario, EstadoCivil, Estatus, Sexo};
pub use componente::Componente;
pub use grado::Grado;
pub use manifiesto::Manifiesto;
