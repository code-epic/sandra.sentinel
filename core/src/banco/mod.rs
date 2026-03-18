pub mod banfanb;
pub mod bicentenario;
pub mod tipos;
pub mod venezuela;

pub use tipos::{Banco, CampoBanco, TipoArchivo};

pub fn generar_txt_bancario(
    _codigo_banco: &str,
    _tipo: TipoArchivo,
    _ciclo: &str,
    _destino: &str,
    _porcentaje: f64,
) -> Result<crate::kernel::logica::exportador::ResultadoExport, Box<dyn std::error::Error>> {
    Ok(crate::kernel::logica::exportador::ResultadoExport {
        ruta: String::new(),
        tipo: String::new(),
        tamano_original: 0,
        tamano_comprimido: None,
        hash_sha256: None,
        hash_sha256_original: None,
        compresion_aplicada: false,
    })
}
