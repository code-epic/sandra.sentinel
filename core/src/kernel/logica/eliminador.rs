// =============================================================================
// MÓDULO: ELIMINADOR DE ARCHIVOS CSV DE MOVIMIENTOS
// =============================================================================
// Centraliza la lógica de visibilidad de archivos CSV de movimientos dentro de
// la funcionalidad de eliminar.
//
// Cada archivo puede estar en distintos estados. Por ahora, los archivos con
// estado o nombre `NO_DEPOSITADO_BANCO` se ocultan de la funcionalidad de
// eliminar (no se muestran como candidatos a eliminar).
// =============================================================================

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Nombre canónico del estado oculto (convención del dominio).
pub const ESTADO_NO_DEPOSITADO_BANCO: &str = "NO_DEPOSITADO_BANCO";

/// Estados posibles de un archivo CSV de movimientos en el flujo de eliminación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EstadoArchivo {
    /// Archivo depositado en banco: visible para eliminar.
    DepositadoBanco,
    /// Archivo NO depositado en banco: oculto de la funcionalidad de eliminar.
    NoDepositadoBanco,
    /// Archivo pendiente de procesamiento: visible para eliminar.
    Pendiente,
    /// Archivo ya procesado: visible para eliminar.
    Procesado,
    /// Estado no reconocido: visible para eliminar (comportamiento conservador).
    #[default]
    Desconocido,
}

impl EstadoArchivo {
    /// Indica si el archivo debe ocultarse de la funcionalidad de eliminar.
    pub fn esta_oculto(&self) -> bool {
        matches!(self, EstadoArchivo::NoDepositadoBanco)
    }
}

impl FromStr for EstadoArchivo {
    type Err = ();

    /// Convierte una cadena al estado correspondiente.
    ///
    /// Reconoce la forma canónica `NO_DEPOSITADO_BANCO` además de variantes en
    /// minúsculas o con separadores comunes (guion bajo, guion medio).
    /// Si el estado no es reconocido, intenta derivarlo del nombre del archivo.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalizado = s.trim().to_uppercase().replace(['-', ' '], "_");

        match normalizado.as_str() {
            "DEPOSITADO_BANCO" | "DEPOSITADO" => Ok(EstadoArchivo::DepositadoBanco),
            "NO_DEPOSITADO_BANCO" | "NO_DEPOSITADO" => Ok(EstadoArchivo::NoDepositadoBanco),
            "PENDIENTE" | "PENDIENTE_REVISION" => Ok(EstadoArchivo::Pendiente),
            "PROCESADO" | "PROCESADO_OK" => Ok(EstadoArchivo::Procesado),
            _ => {
                if es_nombre_no_depositado(s) {
                    // El nombre del archivo también puede derivar el estado.
                    Ok(EstadoArchivo::NoDepositadoBanco)
                } else {
                    Err(())
                }
            }
        }
    }
}

/// Descriptor de un archivo CSV de movimientos candidato a eliminación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivoMovimiento {
    /// Nombre del archivo (p. ej. `movimientos_2026-01.csv`).
    pub nombre: String,
    /// Estado actual del archivo dentro del flujo.
    pub estado: EstadoArchivo,
}

impl ArchivoMovimiento {
    /// Crea un descriptor a partir del nombre y de un estado en formato texto.
    /// Si el estado no se reconoce, se infiere a partir del nombre.
    pub fn nuevo(nombre: impl Into<String>, estado: &str) -> Self {
        let nombre = nombre.into();
        let estado = estado.parse().unwrap_or_default();
        ArchivoMovimiento { nombre, estado }
    }

    /// Indica si este archivo debe ocultarse de la funcionalidad de eliminar
    /// (por estado o por nombre `NO_DEPOSITADO_BANCO`).
    pub fn esta_oculto(&self) -> bool {
        self.estado.esta_oculto() || es_nombre_no_depositado(&self.nombre)
    }
}

/// Verifica si el nombre de un archivo corresponde a `NO_DEPOSITADO_BANCO`
/// (insensible a mayúsculas, guiones, espacios y prefijos/sufijos del nombre).
pub fn es_nombre_no_depositado(nombre: &str) -> bool {
    let nombre_normalizado = nombre.to_uppercase().replace(['-', ' '], "_");
    nombre_normalizado.contains(ESTADO_NO_DEPOSITADO_BANCO)
}

/// Filtra la lista de archivos dejando únicamente los visibles para eliminar.
/// Los archivos `NO_DEPOSITADO_BANCO` (por estado o nombre) se ocultan.
pub fn filtrar_visibles(archivos: &[ArchivoMovimiento]) -> Vec<ArchivoMovimiento> {
    archivos
        .iter()
        .filter(|archivo| !archivo.esta_oculto())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Crea un archivo visible con el estado indicado.
    fn archivo(nombre: &str, estado: EstadoArchivo) -> ArchivoMovimiento {
        ArchivoMovimiento {
            nombre: nombre.to_string(),
            estado,
        }
    }

    #[test]
    fn test_estado_no_depositado_esta_oculto() {
        assert!(EstadoArchivo::NoDepositadoBanco.esta_oculto());
    }

    #[test]
    fn test_estados_visibles_no_estan_ocultos() {
        assert!(!EstadoArchivo::DepositadoBanco.esta_oculto());
        assert!(!EstadoArchivo::Pendiente.esta_oculto());
        assert!(!EstadoArchivo::Procesado.esta_oculto());
        assert!(!EstadoArchivo::Desconocido.esta_oculto());
    }

    #[test]
    fn test_default_es_desconocido_y_visible() {
        let estado = EstadoArchivo::default();
        assert_eq!(estado, EstadoArchivo::Desconocido);
        assert!(!estado.esta_oculto());
    }

    #[test]
    fn test_from_str_reconoce_forma_canonica() {
        assert_eq!(
            "NO_DEPOSITADO_BANCO".parse::<EstadoArchivo>(),
            Ok(EstadoArchivo::NoDepositadoBanco)
        );
    }

    #[test]
    fn test_from_str_es_insensible_a_mayusculas_y_separadores() {
        assert_eq!(
            "no-depositado-banco".parse::<EstadoArchivo>(),
            Ok(EstadoArchivo::NoDepositadoBanco)
        );
        assert_eq!(
            "NO DEPOSITADO BANCO".parse::<EstadoArchivo>(),
            Ok(EstadoArchivo::NoDepositadoBanco)
        );
    }

    #[test]
    fn test_from_str_reconoce_depositado() {
        assert_eq!(
            "DEPOSITADO_BANCO".parse::<EstadoArchivo>(),
            Ok(EstadoArchivo::DepositadoBanco)
        );
    }

    #[test]
    fn test_from_str_infiere_estado_por_nombre() {
        // El estado no es reconocido, pero el nombre deriva el estado.
        assert_eq!(
            "MOV_2026-01_NO_DEPOSITADO_BANCO.csv".parse::<EstadoArchivo>(),
            Ok(EstadoArchivo::NoDepositadoBanco)
        );
    }

    #[test]
    fn test_es_nombre_no_depositado_variantes() {
        assert!(es_nombre_no_depositado("movimientos_NO_DEPOSITADO_BANCO.csv"));
        assert!(es_nombre_no_depositado("no-depositado-banco_2026-01.csv"));
        assert!(es_nombre_no_depositado("NO DEPOSITADO BANCO"));
        assert!(!es_nombre_no_depositado("movimientos_depositado_banco.csv"));
        assert!(!es_nombre_no_depositado("aporte_2026-01.csv"));
    }

    #[test]
    fn test_filtrar_visibles_oculta_por_estado() {
        let archivos = vec![
            archivo("aporte_2026-01.csv", EstadoArchivo::Procesado),
            archivo("movimientos_2026-01.csv", EstadoArchivo::NoDepositadoBanco),
            archivo("aporte_2026-02.csv", EstadoArchivo::DepositadoBanco),
        ];

        let visibles = filtrar_visibles(&archivos);

        assert_eq!(visibles.len(), 2);
        assert_eq!(visibles[0].nombre, "aporte_2026-01.csv");
        assert_eq!(visibles[1].nombre, "aporte_2026-02.csv");
    }

    #[test]
    fn test_filtrar_visibles_oculta_por_nombre() {
        let archivos = vec![
            archivo("aporte_2026-01.csv", EstadoArchivo::Pendiente),
            archivo("mov_NO_DEPOSITADO_BANCO_2026-01.csv", EstadoArchivo::Pendiente),
            archivo("aporte_2026-03.csv", EstadoArchivo::Pendiente),
        ];

        let visibles = filtrar_visibles(&archivos);

        assert_eq!(visibles.len(), 2);
        assert!(!visibles.iter().any(|a| a.nombre.contains("NO_DEPOSITADO")));
    }

    #[test]
    fn test_filtrar_visibles_con_lista_vacia() {
        let visibles = filtrar_visibles(&[]);
        assert!(visibles.is_empty());
    }

    #[test]
    fn test_archivo_nuevo_deriva_estado_desde_cadena() {
        let archivo = ArchivoMovimiento::nuevo("mov.csv", "NO_DEPOSITADO_BANCO");
        assert_eq!(archivo.estado, EstadoArchivo::NoDepositadoBanco);
        assert!(archivo.esta_oculto());
    }

    #[test]
    fn test_archivo_nuevo_con_estado_desconocido_queda_visible() {
        let archivo = ArchivoMovimiento::nuevo("aporte.csv", "ESTADO_RARO");
        assert_eq!(archivo.estado, EstadoArchivo::Desconocido);
        assert!(!archivo.esta_oculto());
    }
}