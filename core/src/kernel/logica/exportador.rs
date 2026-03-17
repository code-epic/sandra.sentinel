use super::logger;
use crate::kernel::logica::memoria::Beneficiario;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoArchivo {
    pub nombre: String,
    #[serde(rename = "tipo")]
    pub tipo_archivo: String,
    #[serde(rename = "tamano_bytes")]
    pub tamano: u64,
    #[serde(rename = "sha256")]
    pub hash: String,
    #[serde(rename = "sha256csv")]
    pub hash_csv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestExport {
    #[serde(rename = "id_operacion")]
    pub id_operacion: String,
    #[serde(rename = "fecha_generacion")]
    pub fecha_generacion: String,
    #[serde(rename = "total_archivos")]
    pub total_archivos: usize,
    #[serde(rename = "compresion")]
    pub compresion: String,
    #[serde(rename = "archivos")]
    pub archivos: Vec<InfoArchivo>,
}

#[derive(Debug, Clone)]
pub struct ResultadoExport {
    pub ruta: String,
    pub tipo: String,
    pub tamano_original: u64,
    pub tamano_comprimido: Option<u64>,
    pub hash_sha256: Option<String>,
    pub hash_sha256_original: Option<String>,
    pub compresion_aplicada: bool,
}

pub fn comprimir_y_sellar(datos: &[u8], nivel: i32) -> (Vec<u8>, String) {
    let mut compresor =
        zstd::stream::write::Encoder::new(Vec::new(), nivel).expect("Error creando compresor zstd");
    compresor
        .write_all(datos)
        .expect("Error comprimiendo datos");
    let comprimido = compresor.finish().expect("Error finalizando compresión");

    let mut hasher = Sha256::new();
    hasher.update(&comprimido);
    let hash = format!("{:x}", hasher.finalize());

    (comprimido, hash)
}

pub fn generar_hash(datos: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(datos);
    format!("{:x}", hasher.finalize())
}

pub fn exportar_nomina_csv(
    beneficiarios: &Vec<Beneficiario>,
    ciclo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
    let nombre_archivo = format!("nomina_{}.csv", ciclo);
    let ruta_completa = if destino == "." || destino.is_empty() {
        PathBuf::from(&nombre_archivo)
    } else {
        PathBuf::from(destino).join(&nombre_archivo)
    };

    let nombre_mostrar = ruta_completa
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| nombre_archivo.clone());

    println!(
        "> Exportando nómina a CSV en '{}' ({} registros)...",
        nombre_mostrar,
        beneficiarios.len()
    );

    let file = File::create(&ruta_completa)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(&[
        "cedula",
        "nombres",
        "apellidos",
        "sexo",
        "edo_civil",
        "n_hijos (base)",
        "componente_id",
        "grado_id (base)",
        "categoria",
        "status_id",
        "status",
        "st_no_ascenso",
        "fecha_ingreso",
        "fecha_ultimo_ascenso",
        "fecha_retiro",
        "anio_reconocido",
        "mes_reconocido",
        "dia_reconocido",
        "antiguedad_total (decimal)",
        "antiguedad_grado (anos)",
        "sueldo_base",
        "prima_antiguedad",
        "prima_hijos",
        "prima_profesionalizacion",
        "total_asignaciones_base",
        "sueldo_mensual",
        "aguinaldos",
        "vacaciones",
        "dia_vacaciones",
        "sueldo_integral",
        "asignacion_antiguedad",
        "garantias",
        "dias_adicionales",
        "deposito_banco",
        "no_depositado_banco",
        "cap_banco",
        "anticipo",
        "f_cap_banco",
        "dif_asi_anti",
        "anticipo_retroactivo",
        "dep_adicional",
        "dep_garantia",
        "patterns",
    ])?;

    for b in beneficiarios {
        let get_calc = |key: &str| -> String {
            if let Some(map) = &b.base.calculos {
                if let Some(val) = map.get(key) {
                    return format!("{:.2}", val);
                }
            }
            "0.00".to_string()
        };

        wtr.write_record(&[
            &b.cedula,
            &b.nombres,
            &b.apellidos,
            b.sexo.as_deref().unwrap_or(""),
            b.edo_civil.as_deref().unwrap_or(""),
            &b.base.n_hijos.to_string(),
            &b.componente_id.to_string(),
            &b.base.grado_id.to_string(),
            b.categoria.as_deref().unwrap_or(""),
            &b.status_id.to_string(),
            &b.status.to_string(),
            &b.st_no_ascenso.to_string(),
            b.base
                .fecha_ingreso
                .as_deref()
                .unwrap_or(b.f_ingreso_sistema.as_deref().unwrap_or("")),
            b.base
                .f_ult_ascenso
                .as_deref()
                .unwrap_or(b.f_ult_ascenso.as_deref().unwrap_or("")),
            b.base
                .f_retiro
                .as_deref()
                .unwrap_or(b.f_retiro.as_deref().unwrap_or("")),
            &b.base.anio_reconocido.to_string(),
            &b.base.mes_reconocido.to_string(),
            &b.base.dia_reconocido.to_string(),
            &format!("{:.4}", b.base.antiguedad),
            &b.base.antiguedad_grado.to_string(),
            &format!("{:.2}", b.base.sueldo_base),
            &get_calc("prima_tiemposervicio"),
            &get_calc("prima_hijos"),
            &get_calc("prima_profesionalizacion"),
            &format!("{:.2}", b.base.total_asignaciones),
            &format!("{:.2}", b.base.sueldo_mensual),
            &format!("{:.2}", b.base.aguinaldos),
            &format!("{:.2}", b.base.vacaciones),
            &b.base.dia_vacaciones.to_string(),
            &format!("{:.2}", b.base.sueldo_integral),
            &format!("{:.2}", b.base.asignacion_antiguedad),
            &format!("{:.2}", b.base.garantias),
            &format!("{:.2}", b.base.dias_adicionales),
            &format!("{:.2}", b.base.deposito_banco),
            &format!("{:.2}", b.base.no_depositado_banco),
            &format!("{:.2}", b.movimientos.cap_banco),
            &format!("{:.2}", b.movimientos.anticipo),
            &format!("{:.2}", b.movimientos.fcap_banco),
            &format!("{:.2}", b.movimientos.dif_asi_anti),
            &format!("{:.2}", b.movimientos.anticipor),
            &format!("{:.2}", b.movimientos.dep_adicional),
            &format!("{:.2}", b.movimientos.dep_garantia),
            &b.patterns,
        ])?;
    }

    wtr.flush()?;

    let datos_csv = std::fs::read(&ruta_completa)?;
    let tamano_original = datos_csv.len() as u64;
    let hash_csv = generar_hash(&datos_csv);
    let resultado: ResultadoExport;

    if comprimir {
        println!(
            "    > Comprimiendo archivo con zstd (nivel {})...",
            nivel_compresion
        );
        let (comprimido, hash) = comprimir_y_sellar(&datos_csv, nivel_compresion);

        let ruta_zst = ruta_completa.with_extension("csv.zst");
        let mut archivo_zst = File::create(&ruta_zst)?;
        archivo_zst.write_all(&comprimido)?;

        std::fs::remove_file(&ruta_completa)?;

        let nombre_zst = ruta_zst
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| ruta_zst.display().to_string());

        println!(
            "    > Archivo comprimido: {} (Original: {} bytes, Comprimido: {} bytes)",
            nombre_zst,
            tamano_original,
            comprimido.len()
        );

        logger::log_info(
            "EXPORT",
            &format!(
                "Archivo CSV comprimido: {} - Hash: {}",
                nombre_zst,
                &hash[..16]
            ),
        );

        resultado = ResultadoExport {
            ruta: ruta_zst.display().to_string(),
            tipo: "nomina".to_string(),
            tamano_original,
            tamano_comprimido: Some(comprimido.len() as u64),
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_csv),
            compresion_aplicada: true,
        };
    } else {
        let hash = generar_hash(&datos_csv);

        logger::log_info(
            "EXPORT",
            &format!(
                "Archivo CSV generado: {} - Hash: {}",
                ruta_completa.display(),
                &hash[..16]
            ),
        );

        resultado = ResultadoExport {
            ruta: ruta_completa.display().to_string(),
            tipo: "nomina".to_string(),
            tamano_original,
            tamano_comprimido: None,
            hash_sha256: Some(hash.clone()),
            hash_sha256_original: Some(hash),
            compresion_aplicada: false,
        };
    }

    Ok(resultado)
}

pub fn exportar_aporte_csv(
    beneficiarios: &Vec<Beneficiario>,
    ciclo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
    let nombre_archivo = format!("aporte_{}.csv", ciclo);
    let ruta_completa = if destino == "." || destino.is_empty() {
        PathBuf::from(&nombre_archivo)
    } else {
        PathBuf::from(destino).join(&nombre_archivo)
    };

    let nombre_mostrar = ruta_completa
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| nombre_archivo.clone());

    println!(
        "> Exportando archivo de aporte a CSV en '{}' ({} registros)...",
        nombre_mostrar,
        beneficiarios.len()
    );

    let file = File::create(&ruta_completa)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(&[
        "cedula",
        "nombres",
        "apellidos",
        "numero_cuenta",
        "garantia_original",
        "factor_aplicado",
        "garantia_anticipo",
    ])?;

    for b in beneficiarios {
        wtr.write_record(&[
            &b.cedula,
            &b.nombres,
            &b.apellidos,
            &b.numero_cuenta,
            &format!("{:.2}", b.base.garantia_original),
            &format!("{:.6}", b.base.factor_aplicado),
            &format!("{:.2}", b.base.garantia_anticipo),
        ])?;
    }

    wtr.flush()?;

    let datos_csv = std::fs::read(&ruta_completa)?;
    let tamano_original = datos_csv.len() as u64;
    let hash_csv = generar_hash(&datos_csv);
    let resultado: ResultadoExport;

    if comprimir {
        println!(
            "    > Comprimiendo archivo con zstd (nivel {})...",
            nivel_compresion
        );
        let (comprimido, hash) = comprimir_y_sellar(&datos_csv, nivel_compresion);

        let ruta_zst = ruta_completa.with_extension("csv.zst");
        let mut archivo_zst = File::create(&ruta_zst)?;
        archivo_zst.write_all(&comprimido)?;

        // Eliminar el CSV original si se comprimió
        std::fs::remove_file(&ruta_completa)?;

        let nombre_zst = ruta_zst
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| ruta_zst.display().to_string());

        println!(
            "    > Archivo comprimido: {} (Original: {} bytes, Comprimido: {} bytes)",
            nombre_zst,
            tamano_original,
            comprimido.len()
        );

        logger::log_info(
            "EXPORT_APORTE",
            &format!("Archivo comprimido: {} - Hash: {}", nombre_zst, &hash[..16]),
        );

        resultado = ResultadoExport {
            ruta: ruta_zst.display().to_string(),
            tipo: "aporte".to_string(),
            tamano_original,
            tamano_comprimido: Some(comprimido.len() as u64),
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_csv),
            compresion_aplicada: true,
        };
    } else {
        let hash = generar_hash(&datos_csv);

        logger::log_info(
            "EXPORT_APORTE",
            &format!(
                "Archivo CSV generado: {} - Hash: {}",
                ruta_completa.display(),
                &hash[..16]
            ),
        );

        resultado = ResultadoExport {
            ruta: ruta_completa.display().to_string(),
            tipo: "aporte".to_string(),
            tamano_original,
            tamano_comprimido: None,
            hash_sha256: Some(hash.clone()),
            hash_sha256_original: Some(hash),
            compresion_aplicada: false,
        };
    }

    Ok(resultado)
}

pub fn generar_manifest(
    id_operacion: &str,
    destino: &str,
    resultados: &[ResultadoExport],
) -> Result<(), Box<dyn std::error::Error>> {
    let archivos: Vec<InfoArchivo> = resultados
        .iter()
        .map(|r| {
            let nombre = std::path::Path::new(&r.ruta)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("desconocido")
                .to_string();

            let hash_csv = r.hash_sha256_original.as_deref().unwrap_or("").to_string();

            let (tamano, hash) = if r.compresion_aplicada {
                (
                    r.tamano_comprimido.unwrap_or(0),
                    r.hash_sha256.as_deref().unwrap_or("").to_string(),
                )
            } else {
                // Sin compresión: no hay hash zst
                (r.tamano_original, String::new())
            };

            InfoArchivo {
                nombre,
                tipo_archivo: r.tipo.clone(),
                tamano: tamano,
                hash: hash,
                hash_csv: hash_csv,
            }
        })
        .collect();

    let manifest = ManifestExport {
        id_operacion: id_operacion.to_string(),
        fecha_generacion: chrono::Utc::now().to_rfc3339(),
        total_archivos: archivos.len(),
        compresion: if resultados
            .first()
            .map(|r| r.compresion_aplicada)
            .unwrap_or(false)
        {
            "zstd".to_string()
        } else {
            "ninguna".to_string()
        },
        archivos,
    };

    let nombre_manifest = "manifest.json".to_string();
    let ruta_manifest = if destino == "." || destino.is_empty() {
        std::path::PathBuf::from(&nombre_manifest)
    } else {
        std::path::PathBuf::from(destino).join(&nombre_manifest)
    };

    let json = serde_json::to_string_pretty(&manifest)?;
    let mut archivo = File::create(&ruta_manifest)?;
    archivo.write_all(json.as_bytes())?;

    let nombre_archivo = ruta_manifest
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| ruta_manifest.display().to_string());

    println!("    > Manifest generado: {}", nombre_archivo);
    logger::log_info("EXPORT", &format!("Manifest generado: {}", nombre_archivo));

    Ok(())
}
