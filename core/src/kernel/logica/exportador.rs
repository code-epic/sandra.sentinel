use super::logger;
use crate::banco::tipos::TipoArchivo;
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

pub fn exportar_aporte_y_apertura_txt(
    beneficiaries: &Vec<Beneficiario>,
    ciclo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
) -> Result<(ResultadoExport, ResultadoExport), Box<dyn std::error::Error>> {
    let mut aporte = Vec::new();
    let mut apertura = Vec::new();

    for b in beneficiaries {
        let m = &b.movimientos;
        let total_mov = m.cap_banco + m.anticipo + m.dep_adicional + m.dep_garantia + m.anticipor;
        if total_mov > 0.0 {
            aporte.push(b.clone());
        } else {
            apertura.push(b.clone());
        }
    }

    println!(
        "    > Dividiendo beneficiarios: {} aporte, {} apertura",
        aporte.len(),
        apertura.len()
    );

    let res_aporte = exportar_aporte_csv(&mut aporte, ciclo, destino, comprimir, nivel_compresion)?;
    let res_apertura = crate::banco::venezuela::generar_txt_venezuela(
        &apertura,
        TipoArchivo::Apertura,
        ciclo,
        destino,
        100.0,
        comprimir,
        nivel_compresion,
    )?;

    Ok((res_aporte, res_apertura))
}

pub fn exportar_aporte_y_apertura_csv(
    beneficiarios: &Vec<Beneficiario>,
    ciclo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
    let nombre_archivo = format!("apertura_{}.csv", ciclo);
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
        "> Exportando archivo de apertura a CSV en '{}' ({} registros)...",
        nombre_mostrar,
        beneficiarios.len()
    );

    if beneficiarios.is_empty() {
        println!("    > No hay beneficiarios para apertura (todos tienen movimientos)");
        return Ok(ResultadoExport {
            ruta: String::new(),
            tipo: "apertura".to_string(),
            tamano_original: 0,
            tamano_comprimido: None,
            hash_sha256: None,
            hash_sha256_original: None,
            compresion_aplicada: false,
        });
    }

    let file = File::create(&ruta_completa)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(&[
        "cedula",
        "nombres",
        "apellidos",
        "numero_cuenta",
        "garantia_original",
    ])?;

    for b in beneficiarios {
        wtr.write_record(&[
            &b.cedula,
            &b.nombres,
            &b.apellidos,
            &b.numero_cuenta,
            &format!("{:.2}", b.base.garantia_original),
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
            "EXPORT_APERTURA",
            &format!("Archivo comprimido: {} - Hash: {}", nombre_zst, &hash[..16]),
        );

        resultado = ResultadoExport {
            ruta: ruta_zst.display().to_string(),
            tipo: "apertura".to_string(),
            tamano_original,
            tamano_comprimido: Some(comprimido.len() as u64),
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_csv),
            compresion_aplicada: true,
        };
    } else {
        let hash = generar_hash(&datos_csv);

        logger::log_info(
            "EXPORT_APERTURA",
            &format!(
                "Archivo CSV generado: {} - Hash: {}",
                ruta_completa.display(),
                &hash[..16]
            ),
        );

        resultado = ResultadoExport {
            ruta: ruta_completa.display().to_string(),
            tipo: "apertura".to_string(),
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

pub fn exportar_nomina_dinamica(
    beneficiarios: &Vec<Beneficiario>,
    ciclo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
    es_nfcp: bool,
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
        "> Exportando nómina dinámica a CSV '{}' ({} registros)...",
        nombre_mostrar,
        beneficiarios.len()
    );

    let file = File::create(&ruta_completa)?;
    let mut wtr = csv::Writer::from_writer(file);

    let mut headers = vec!["cedula", "nombres", "apellidos"];

    headers.push("porcentaje");
    headers.push("sueldo_integral");
    headers.push("sueldo_neto_porcentaje");
    headers.push("total_asignaciones");
    headers.push("total_deducciones");
    headers.push("neto");

    if es_nfcp {
        headers.push("cedula_titular");
        headers.push("parentesco");
        headers.push("nombre_autorizado");
    }

    wtr.write_record(&headers)?;

    for b in beneficiarios {
        let neto_porcentaje = b.base.sueldo_integral * (b.porcentaje / 100.0);

        let mut record = vec![
            b.cedula.clone(),
            b.nombres.clone(),
            b.apellidos.clone(),
            format!("{:.2}", b.porcentaje),
            format!("{:.2}", b.base.sueldo_integral),
            format!("{:.2}", neto_porcentaje),
            format!("{:.2}", b.total_asignaciones),
            format!("{:.2}", b.total_deducciones),
            format!("{:.2}", b.neto),
        ];

        if es_nfcp {
            record.push(b.cedula_titular.clone().unwrap_or_default());
            record.push(b.parentesco.clone().unwrap_or_default());
            record.push(b.nombre_autorizado.clone().unwrap_or_default());
        }

        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    drop(wtr);

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
                "Archivo CSV dinámico comprimido: {} - Hash: {}",
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
                "Archivo CSV dinámico generado: {} - Hash: {}",
                ruta_completa.display(),
                &hash[..16]
            ),
        );

        resultado = ResultadoExport {
            ruta: ruta_completa.display().to_string(),
            tipo: "nomina".to_string(),
            tamano_original,
            tamano_comprimido: None,
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_csv),
            compresion_aplicada: false,
        };
    }

    Ok(resultado)
}

pub fn exportar_nomina_por_tipo(
    beneficiarios: &Vec<Beneficiario>,
    ciclo: &str,
    tipo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
    es_nfcp: bool,
) -> Result<Vec<ResultadoExport>, Box<dyn std::error::Error>> {
    let es_npr = tipo == "npr";
    let mut resultados = Vec::new();

    let (principales, paralizados): (Vec<&Beneficiario>, Vec<&Beneficiario>) =
        beneficiarios.iter().partition(|b| b.porcentaje > 0.0);

    if !principales.is_empty() {
        let resultado = generar_csv_nomina(
            principales,
            ciclo,
            tipo,
            destino,
            comprimir,
            nivel_compresion,
            es_nfcp,
            es_npr,
            false,
        )?;
        resultados.push(resultado);
    }

    if !paralizados.is_empty() {
        let resultado = generar_csv_nomina(
            paralizados,
            ciclo,
            tipo,
            destino,
            comprimir,
            nivel_compresion,
            es_nfcp,
            es_npr,
            true,
        )?;
        resultados.push(resultado);
    }

    Ok(resultados)
}

fn generar_csv_nomina(
    beneficiarios: Vec<&Beneficiario>,
    ciclo: &str,
    tipo: &str,
    destino: &str,
    comprimir: bool,
    nivel_compresion: i32,
    es_nfcp: bool,
    es_npr: bool,
    es_paralizado: bool,
) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
    let sufijo = if es_paralizado { "_paralizado" } else { "" };
    let nombre_archivo = format!("nomina_{}{}_{}.csv", tipo, sufijo, ciclo);

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
        "> Exportando {} a CSV '{}' ({} registros)...",
        if es_paralizado {
            "paralizados"
        } else {
            "nomina"
        },
        nombre_mostrar,
        beneficiarios.len()
    );

    let file = File::create(&ruta_completa)?;
    let mut wtr = csv::Writer::from_writer(file);

    // Para archivos de paralizados, no se incluyen conceptos en headers ni registros
    let incluir_conceptos = !es_npr && !es_paralizado;
    let headers = generar_headers_nomina(es_npr, es_nfcp, incluir_conceptos, &beneficiarios);
    wtr.write_record(&headers)?;

    for b in &beneficiarios {
        let record = generar_registro_nomina(b, &headers, es_npr, es_nfcp, incluir_conceptos);
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    drop(wtr);

    comprimir_y_guardar(&ruta_completa, comprimir, nivel_compresion)
}

fn generar_headers_nomina(
    es_npr: bool,
    es_nfcp: bool,
    incluir_conceptos: bool,
    beneficiarios: &[&Beneficiario],
) -> Vec<String> {
    let mut headers = Vec::new();

    headers.push("cedula".to_string());
    headers.push("nombres".to_string());
    headers.push("apellidos".to_string());
    headers.push("sexo".to_string());
    headers.push("edo_civil".to_string());
    headers.push("n_hijos".to_string());
    headers.push("componente_id".to_string());
    headers.push("grado_id".to_string());
    headers.push("categoria".to_string());
    headers.push("status_id".to_string());
    headers.push("status".to_string());
    headers.push("st_no_ascenso".to_string());
    headers.push("fecha_ingreso".to_string());
    headers.push("fecha_ultimo_ascenso".to_string());
    headers.push("fecha_retiro".to_string());
    headers.push("anio_reconocido".to_string());
    headers.push("mes_reconocido".to_string());
    headers.push("dia_reconocido".to_string());
    headers.push("antiguedad_total".to_string());
    headers.push("antiguedad_grado".to_string());
    headers.push("sueldo_base".to_string());
    headers.push("prima_antiguedad".to_string());
    headers.push("prima_hijos".to_string());
    headers.push("prima_profesionalizacion".to_string());
    headers.push("total_asignaciones_base".to_string());
    headers.push("sueldo_mensual".to_string());

    if es_npr {
        headers.push("aguinaldos".to_string());
        headers.push("vacaciones".to_string());
        headers.push("dia_vacaciones".to_string());
        headers.push("sueldo_integral".to_string());
        headers.push("asignacion_antiguedad".to_string());
        headers.push("garantias".to_string());
        headers.push("dias_adicionales".to_string());
        headers.push("deposito_banco".to_string());
        headers.push("no_depositado_banco".to_string());
        headers.push("cap_banco".to_string());
        headers.push("anticipo".to_string());
        headers.push("f_cap_banco".to_string());
        headers.push("dif_asi_anti".to_string());
        headers.push("anticipo_retroactivo".to_string());
        headers.push("dep_adicional".to_string());
        headers.push("dep_garantia".to_string());
    }

    headers.push("patterns".to_string());
    headers.push("porcentaje".to_string());
    headers.push("sueldo_neto_porcentaje".to_string());

    if incluir_conceptos {
        let mut descripciones_conceptos: Vec<String> = Vec::new();
        for b in beneficiarios {
            if let Some(conceptos) = &b.conceptos_calculados {
                for (_, concepto) in conceptos.iter() {
                    if !descripciones_conceptos.contains(&concepto.descripcion) {
                        descripciones_conceptos.push(concepto.descripcion.clone());
                    }
                }
            }
        }
        descripciones_conceptos.sort();
        headers.extend(descripciones_conceptos);

        headers.push("total_asignaciones".to_string());
        headers.push("total_deducciones".to_string());
    }

    headers.push("sueldo_total".to_string());

    if es_nfcp {
        headers.push("cedula_titular".to_string());
        headers.push("parentesco".to_string());
        headers.push("nombre_autorizado".to_string());
    }

    headers
}

fn generar_registro_nomina(
    b: &Beneficiario,
    headers: &[String],
    es_npr: bool,
    es_nfcp: bool,
    incluir_conceptos: bool,
) -> Vec<String> {
    let mut record = Vec::new();

    let get_calc = |key: &str| -> String {
        if let Some(map) = &b.base.calculos {
            if let Some(val) = map.get(key) {
                return format!("{:.2}", val);
            }
        }
        "0.00".to_string()
    };

    record.push(b.cedula.clone());
    record.push(b.nombres.clone());
    record.push(b.apellidos.clone());
    record.push(b.sexo.as_deref().unwrap_or("").to_string());
    record.push(b.edo_civil.as_deref().unwrap_or("").to_string());
    record.push(b.base.n_hijos.to_string());
    record.push(b.componente_id.to_string());
    record.push(b.base.grado_id.to_string());
    record.push(b.categoria.as_deref().unwrap_or("").to_string());
    record.push(b.status_id.to_string());
    record.push(b.status.to_string());
    record.push(b.st_no_ascenso.to_string());
    record.push(
        b.base
            .fecha_ingreso
            .as_deref()
            .unwrap_or(b.f_ingreso_sistema.as_deref().unwrap_or(""))
            .to_string(),
    );
    record.push(
        b.base
            .f_ult_ascenso
            .as_deref()
            .unwrap_or(b.f_ult_ascenso.as_deref().unwrap_or(""))
            .to_string(),
    );
    record.push(
        b.base
            .f_retiro
            .as_deref()
            .unwrap_or(b.f_retiro.as_deref().unwrap_or(""))
            .to_string(),
    );
    record.push(b.base.anio_reconocido.to_string());
    record.push(b.base.mes_reconocido.to_string());
    record.push(b.base.dia_reconocido.to_string());
    record.push(format!("{:.4}", b.base.antiguedad));
    record.push(b.base.antiguedad_grado.to_string());
    record.push(format!("{:.2}", b.base.sueldo_base));
    record.push(get_calc("prima_tiemposervicio"));
    record.push(get_calc("prima_hijos"));
    record.push(get_calc("prima_profesionalizacion"));
    record.push(format!("{:.2}", b.base.total_asignaciones));
    record.push(format!("{:.2}", b.base.sueldo_mensual));

    if es_npr {
        record.push(format!("{:.2}", b.base.aguinaldos));
        record.push(format!("{:.2}", b.base.vacaciones));
        record.push(b.base.dia_vacaciones.to_string());
        record.push(format!("{:.2}", b.base.sueldo_integral));
        record.push(format!("{:.2}", b.base.asignacion_antiguedad));
        record.push(format!("{:.2}", b.base.garantias));
        record.push(format!("{:.2}", b.base.dias_adicionales));
        record.push(format!("{:.2}", b.base.deposito_banco));
        record.push(format!("{:.2}", b.base.no_depositado_banco));
        record.push(format!("{:.2}", b.movimientos.cap_banco));
        record.push(format!("{:.2}", b.movimientos.anticipo));
        record.push(format!("{:.2}", b.movimientos.fcap_banco));
        record.push(format!("{:.2}", b.movimientos.dif_asi_anti));
        record.push(format!("{:.2}", b.movimientos.anticipor));
        record.push(format!("{:.2}", b.movimientos.dep_adicional));
        record.push(format!("{:.2}", b.movimientos.dep_garantia));
    }

    record.push(b.patterns.clone());
    record.push(format!("{:.2}", b.porcentaje));

    let sueldo_neto_pct = b.base.sueldo_mensual * (b.porcentaje / 100.0);
    record.push(format!("{:.2}", sueldo_neto_pct));

    // Para NACT/NRCP/NFCP (no NPR, no paralizados): agregar columnas de conceptos
    if incluir_conceptos {
        // Los headers de conceptos estan despues de los campos fijos (26 campos base)
        let num_campos_fijos = 26;

        if headers.len() > num_campos_fijos {
            let conceptos_headers = &headers[num_campos_fijos..];

            for header in conceptos_headers {
                if es_campo_fijo(header) {
                    continue;
                }

                // Buscar este concepto en los calculos del beneficiario
                let valor = if let Some(conceptos) = &b.conceptos_calculados {
                    conceptos
                        .values()
                        .find(|c| &c.descripcion == header)
                        .map(|c| format!("{:.2}", c.valor))
                        .unwrap_or_else(|| "0.00".to_string())
                } else {
                    "0.00".to_string()
                };
                record.push(valor);
            }
        }

        record.push(format!("{:.2}", b.total_asignaciones.abs()));
        record.push(format!("{:.2}", b.total_deducciones.abs()));
    }

    let sueldo_total = if es_npr {
        b.base.sueldo_integral
    } else if !incluir_conceptos {
        0.0
    } else {
        sueldo_neto_pct + b.total_asignaciones - b.total_deducciones
    };
    record.push(format!("{:.2}", sueldo_total));

    if es_nfcp {
        record.push(b.cedula_titular.clone().unwrap_or_default());
        record.push(b.parentesco.clone().unwrap_or_default());
        record.push(b.nombre_autorizado.clone().unwrap_or_default());
    }

    record
}

fn es_campo_fijo(campo: &str) -> bool {
    matches!(
        campo,
        "cedula"
            | "nombres"
            | "apellidos"
            | "sexo"
            | "edo_civil"
            | "n_hijos"
            | "componente_id"
            | "grado_id"
            | "categoria"
            | "status_id"
            | "status"
            | "st_no_ascenso"
            | "fecha_ingreso"
            | "fecha_ultimo_ascenso"
            | "fecha_retiro"
            | "anio_reconocido"
            | "mes_reconocido"
            | "dia_reconocido"
            | "antiguedad_total"
            | "antiguedad_grado"
            | "sueldo_base"
            | "prima_antiguedad"
            | "prima_hijos"
            | "prima_profesionalizacion"
            | "total_asignaciones_base"
            | "sueldo_mensual"
            | "aguinaldos"
            | "vacaciones"
            | "dia_vacaciones"
            | "sueldo_integral"
            | "asignacion_antiguedad"
            | "garantias"
            | "dias_adicionales"
            | "deposito_banco"
            | "no_depositado_banco"
            | "cap_banco"
            | "anticipo"
            | "f_cap_banco"
            | "dif_asi_anti"
            | "anticipo_retroactivo"
            | "dep_adicional"
            | "dep_garantia"
            | "patterns"
            | "porcentaje"
            | "sueldo_neto_porcentaje"
            | "total_asignaciones"
            | "total_deducciones"
            | "sueldo_total"
            | "cedula_titular"
            | "parentesco"
            | "nombre_autorizado"
    )
}

fn comprimir_y_guardar(
    ruta_completa: &PathBuf,
    comprimir: bool,
    nivel_compresion: i32,
) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
    let datos_csv = std::fs::read(ruta_completa)?;
    let tamano_original = datos_csv.len() as u64;
    let hash_csv = generar_hash(&datos_csv);

    if comprimir {
        println!(
            "    > Comprimiendo archivo con zstd (nivel {})...",
            nivel_compresion
        );
        let (comprimido, hash) = comprimir_y_sellar(&datos_csv, nivel_compresion);

        let ruta_zst = ruta_completa.with_extension("csv.zst");
        let mut archivo_zst = File::create(&ruta_zst)?;
        archivo_zst.write_all(&comprimido)?;

        std::fs::remove_file(ruta_completa)?;

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

        Ok(ResultadoExport {
            ruta: ruta_zst.display().to_string(),
            tipo: "nomina".to_string(),
            tamano_original,
            tamano_comprimido: Some(comprimido.len() as u64),
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_csv),
            compresion_aplicada: true,
        })
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

        Ok(ResultadoExport {
            ruta: ruta_completa.display().to_string(),
            tipo: "nomina".to_string(),
            tamano_original,
            tamano_comprimido: None,
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_csv),
            compresion_aplicada: false,
        })
    }
}
