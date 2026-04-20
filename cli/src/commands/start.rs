use sandra_core::banco::{self, TipoArchivo};
use sandra_core::kernel::logica::{exportador, logger, telemetria};
use sandra_core::tipos::TipoNomina;
use sandra_core::System;

use sandra_core::model::Manifiesto;

use chrono;

fn path_relative(full_path: &str, destino: &str) -> String {
    if destino == "." || destino.is_empty() {
        std::path::Path::new(full_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| full_path.to_string())
    } else {
        std::path::Path::new(full_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| full_path.to_string())
    }
}

pub async fn execute(
    execute: bool,
    log: bool,
    sensors: bool,
    manifest_path: Option<String>,
    tipo: TipoNomina,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Guardar flag de debug globalmente para usar en otros modulos
    if debug {
        std::env::set_var("SANDRA_DEBUG", "1");
        println!("[DEBUG] Modo debug habilitado");
    }
    telemetria::init(sensors);

    // --- BANNER DE INICIO ---
    println!("\n{:=<80}", "");
    println!("{:^80}", "SANDRA SENTINEL - EJECUCIÓN DE NÓMINA");
    println!("{:^80}", "");
    println!("{:^80}", format!("TIPO: {} - {}", tipo, tipo.descripcion()));
    println!("{:=<80}", "");

    let mut system = System::init(); // Restaurado

    // Cargar manifiesto si se especificó
    let mut destino = String::from("."); // Default destination
    if let Some(path) = manifest_path {
        let nombre_manifest = std::path::Path::new(&path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        println!("> Cargando manifiesto desde '{}'...", nombre_manifest);
        match Manifiesto::cargar_desde_archivo(&path) {
            Ok(m) => {
                println!("{:<20} : {}", "[CONFIG] Manifiesto", m.nombre);
                println!("{:<20} : {}", "[CICLO ] Periodo", m.ciclo);
                println!("{:<20} : {}", "[INFO  ] Descripción", m.descripcion);
                
                destino = m.salida.destino.clone();
                logger::init(log, &destino);
                
                system.kernel.config = m;
            }
            Err(e) => {
                eprintln!("[ERROR] Cargando manifiesto: {}", e);
                return Err(e);
            }
        }
    } else {
        println!("{:<20} : {}", "[CONFIG] Modo", "Estándar (Sin manifiesto)");
        logger::init(log, &destino);
    }
    println!("{:-<80}", "");

    // Conectar a Sandra (Golang)
    let url = system.config.get_url();
    // println!("Conectando a Sandra Server en {}...", url); // Ruido
    if let Err(e) = system.connect_sandra(url.clone()).await {
        let msg = format!("Error conectando a Sandra Server: {}", e);
        eprintln!("[ERROR] {}", msg);
        logger::log_error("CONEXION", &msg);
        return Ok(());
    }
    println!("{:<20} : {} ({})", "[STATUS] Conexión", "ESTABLE", url);
    println!("{:=<80}\n", "");

    if execute {
        let start = std::time::Instant::now();
        match system.kernel.ejecutar_ciclo_carga(tipo).await {
            Ok(_) => {
                let duration = start.elapsed();

                // --- RESUMEN FINAL ---
                println!("\n{:=<80}", "");
                println!("{:^80}", "RESUMEN FINAL DE EJECUCIÓN");
                println!("{:=<80}", "");

                let len = system.kernel.beneficiarios.len();
                println!("  {:<25} : {:>10} Beneficiarios", "Total Procesado", len);
                println!("  {:<25} : {:>10.2?}", "Tiempo Total", duration);

                if len > 0 {
                    // Obtener configuración de salida
                    let ciclo = &system.kernel.config.ciclo;
                    let destino = &system.kernel.config.salida.destino;
                    let comprimir = system.kernel.config.salida.compresion;
                    let nivel = system.kernel.config.salida.nivel_compresion;

                    // Vector para almacenar resultados y generar manifest
                    let mut resultados_export: Vec<exportador::ResultadoExport> = Vec::new();

                    // Determinar tipo de nómina como string
                    let tipo_str = match tipo {
                        sandra_core::tipos::TipoNomina::Npr => "npr",
                        sandra_core::tipos::TipoNomina::Nact => "nact",
                        sandra_core::tipos::TipoNomina::Nrcp => "nrcp",
                        sandra_core::tipos::TipoNomina::Nfcp => "nfcp",
                        sandra_core::tipos::TipoNomina::Npat => "npat",
                    };
                    let es_nfcp = matches!(tipo, sandra_core::tipos::TipoNomina::Nfcp);

                    // EXPORTACION NÓMINA POR TIPO
                    let t_export = std::time::Instant::now();

                    match exportador::exportar_nomina_por_tipo(
                        &system.kernel.beneficiarios,
                        ciclo,
                        tipo_str,
                        destino,
                        comprimir,
                        nivel,
                        es_nfcp,
                    ) {
                        Ok(resultados_nomina) => {
                            telemetria::record(
                                "EXPORT",
                                "CSV Nómina",
                                t_export.elapsed(),
                                system.kernel.beneficiarios.len(),
                                &format!("{} archivos", resultados_nomina.len()),
                            );

                            for resultado in resultados_nomina {
                                println!(
                                    "  {:<25} : {:>10} ({})",
                                    "Exportación Nómina",
                                    "OK",
                                    path_relative(&resultado.ruta, &destino)
                                );

                                if let Some(hash) = &resultado.hash_sha256 {
                                    println!(
                                        "    {:<23} : SHA256: {}",
                                        "Firma Digital",
                                        hash
                                    );
                                }

                                if resultado.compresion_aplicada {
                                    println!(
                                        "    {:<23} : Original: {} bytes, Comprimido: {} bytes",
                                        "Compresión",
                                        resultado.tamano_original,
                                        resultado.tamano_comprimido.unwrap_or(0)
                                    );
                                }

                                resultados_export.push(resultado);
                            }
                        }
                        Err(e) => {
                            let msg = format!("Error exportando CSV: {}", e);
                            eprintln!("  {:<25} : {:>10}", "Exportación Nómina", "FALLO");
                            eprintln!("    └─ [ERROR] {}", msg);
                            logger::log_error("EXPORT", &msg);
                        }
                    }

                    // EXPORTACIÓN APORTE (SOLO para NPR y si está habilitado)
                    if matches!(tipo, sandra_core::tipos::TipoNomina::Npr) 
                       && system.kernel.config.aportes.habilitar {
                        let t_export_aporte = std::time::Instant::now();

                        if system.kernel.config.aportes.generar_apertura_con_aporte {
                            println!("\n{:-<80}", "");
                            println!("{:^80}", "EXPORTANDO APORTE Y APERTURA CONJUNTOS");
                            println!("{:-<80}\n", "");

                            match exportador::exportar_aporte_y_apertura_txt(
                                &system.kernel.beneficiarios,
                                ciclo,
                                destino,
                                comprimir,
                                nivel,
                            ) {
                                Ok((res_aporte, res_apertura)) => {
                                    telemetria::record(
                                        "EXPORT",
                                        "CSV Aporte+Apertura",
                                        t_export_aporte.elapsed(),
                                        system.kernel.beneficiarios.len(),
                                        &format!("aporte:{} bytes", res_aporte.tamano_original),
                                    );

                                    println!(
                                        "  {:<25} : {:>10} ({})",
                                        "Exportación Aporte",
                                        "OK",
                                        path_relative(&res_aporte.ruta, &destino)
                                    );

                                    if let Some(hash) = &res_aporte.hash_sha256 {
                                        println!("    {:<23} : SHA256: {}", "Firma Digital", hash);
                                    }

                                    if res_aporte.compresion_aplicada {
                                        println!(
                                            "    {:<23} : Original: {} bytes, Comprimido: {} bytes",
                                            "Compresión",
                                            res_aporte.tamano_original,
                                            res_aporte.tamano_comprimido.unwrap_or(0)
                                        );
                                    }

                                    resultados_export.push(res_aporte);

                                    if res_apertura.tamano_original > 0 {
                                        println!(
                                            "  {:<25} : {:>10} ({})",
                                            "Exportación Apertura",
                                            "OK",
                                            path_relative(&res_apertura.ruta, &destino)
                                        );

                                        if let Some(hash) = &res_apertura.hash_sha256 {
                                            println!("    {:<23} : SHA256: {}", "Firma Digital", hash);
                                        }

                                        if res_apertura.compresion_aplicada {
                                            println!(
                                                "    {:<23} : Original: {} bytes, Comprimido: {} bytes",
                                                "Compresión",
                                                res_apertura.tamano_original,
                                                res_apertura.tamano_comprimido.unwrap_or(0)
                                            );
                                        }

                                        resultados_export.push(res_apertura);
                                    } else {
                                        println!("  {:<25} : {:>10}", "Exportación Apertura", "SKIP (vacío)");
                                    }
                                }
                                Err(e) => {
                                    let msg = format!("Error exportando CSV de aporte y apertura: {}", e);
                                    eprintln!("  {:<25} : {:>10}", "Export Aporte+Apertura", "FALLO");
                                    eprintln!("    └─ [ERROR] {}", msg);
                                    logger::log_error("EXPORT", &msg);
                                }
                            }
                        } else {
                            match exportador::exportar_aporte_csv(
                                &system.kernel.beneficiarios,
                                ciclo,
                                destino,
                                comprimir,
                                nivel,
                            ) {
                                Ok(resultado) => {
                                    telemetria::record(
                                        "EXPORT",
                                        "CSV Aporte",
                                        t_export_aporte.elapsed(),
                                        system.kernel.beneficiarios.len(),
                                        &format!("{} bytes", resultado.tamano_original),
                                    );

                                    println!(
                                        "  {:<25} : {:>10} ({})",
                                        "Exportación Aporte",
                                        "OK",
                                        path_relative(&resultado.ruta, &destino)
                                    );

                                    if let Some(hash) = &resultado.hash_sha256 {
                                        println!(
                                            "    {:<23} : SHA256: {}",
                                            "Firma Digital",
                                            hash
                                        );
                                    }

                                    if resultado.compresion_aplicada {
                                        println!(
                                            "    {:<23} : Original: {} bytes, Comprimido: {} bytes",
                                            "Compresión",
                                            resultado.tamano_original,
                                            resultado.tamano_comprimido.unwrap_or(0)
                                        );
                                    }

                                    resultados_export.push(resultado);
                                }
                                Err(e) => {
                                    let msg = format!("Error exportando CSV de aporte: {}", e);
                                    eprintln!("  {:<25} : {:>10}", "Exportación Aporte", "FALLO");
                                    eprintln!("    └─ [ERROR] {}", msg);
                                    logger::log_error("EXPORT", &msg);
                                }
                            }
                        }
                    }

                    // GENERAR ARCHIVOS TXT BANCARIOS (SOLO para NPR)
                    if matches!(tipo, sandra_core::tipos::TipoNomina::Npr) {
                        if let Some(format_txt) = &system.kernel.config.salida.format_txt {
                            let bancos = &system.kernel.config.salida.bancos;
                            if !bancos.is_empty() {
                                println!("\n{:-<80}", "");
                                println!("{:^80}", "GENERANDO ARCHIVOS TXT BANCARIOS");
                                println!("{:-<80}\n", "");

                                let tipo = TipoArchivo::from_str(format_txt).unwrap_or(TipoArchivo::Aporte);
                                let comprimir = system.kernel.config.salida.compresion;
                                let nivel = system.kernel.config.salida.nivel_compresion;
                                
                                for codigo_banco in bancos {
                                println!("> Procesando banco: {}...", codigo_banco);
                                
                                match codigo_banco.as_str() {
                                    "0102" => {
                                        match banco::venezuela::generar_txt_venezuela(
                                            &system.kernel.beneficiarios,
                                            tipo,
                                            ciclo,
                                            destino,
                                            100.0,
                                            comprimir,
                                            nivel,
                                        ) {
                                            Ok(resultado) => {
                                                println!(
                                                    "  {:<25} : {:>10} ({})",
                                                    "TXT Venezuela",
                                                    "OK",
                                                    path_relative(&resultado.ruta, destino)
                                                );
                                                resultados_export.push(resultado);
                                            }
                                            Err(e) => {
                                                eprintln!("  {:<25} : {:>10}", "TXT Venezuela", "FALLO");
                                                eprintln!("    └─ [ERROR] {}", e);
                                            }
                                        }
                                    }
                                    "0177" => {
                                        match banco::banfanb::generar_txt_banfanb(
                                            &system.kernel.beneficiarios,
                                            ciclo,
                                            destino,
                                            "0131",
                                            comprimir,
                                            nivel,
                                        ) {
                                            Ok(resultado) => {
                                                println!(
                                                    "  {:<25} : {:>10} ({})",
                                                    "TXT Banfanb",
                                                    "OK",
                                                    path_relative(&resultado.ruta, destino)
                                                );
                                                resultados_export.push(resultado);
                                            }
                                            Err(e) => {
                                                eprintln!("  {:<25} : {:>10}", "TXT Banfanb", "FALLO");
                                                eprintln!("    └─ [ERROR] {}", e);
                                            }
                                        }
                                    }
                                    "0175" => {
                                        match banco::bicentenario::generar_txt_bicentenario(
                                            &system.kernel.beneficiarios,
                                            ciclo,
                                            destino,
                                            "0175",
                                            comprimir,
                                            nivel,
                                        ) {
                                            Ok(resultado) => {
                                                println!(
                                                    "  {:<25} : {:>10} ({})",
                                                    "TXT Bicentenario",
                                                    "OK",
                                                    path_relative(&resultado.ruta, destino)
                                                );
                                                resultados_export.push(resultado);
                                            }
                                            Err(e) => {
                                                eprintln!("  {:<25} : {:>10}", "TXT Bicentenario", "FALLO");
                                                eprintln!("    └─ [ERROR] {}", e);
                                            }
                                        }
                                    }
                                    _ => {
                                        println!("  {:<25} : {:>10} (banco no soportado)", "TXT", "SKIP");
                                    }
                                }
                            }
                        }
                    }
                    }  // Cierre if NPR para TXT bancarios

                    // GENERAR MANIFEST
                    if !resultados_export.is_empty() {
                        let id_operacion = format!("NOM-{}-{}", ciclo, chrono::Local::now().format("%Y%m%d-%H%M"));
                        
                        if let Err(e) = exportador::generar_manifest(
                            &id_operacion,
                            destino,
                            &resultados_export,
                        ) {
                            eprintln!("  {:<25} : {:>10}", "Manifest", "FALLO");
                            eprintln!("    └─ [ERROR] {}", e);
                        }
                    }
                }

                // Generar reporte final de telemetría
                telemetria::generate_report(&destino);
                println!(
                    "  {:<25} : {:>10} ({})",
                    "Reporte Sensores", "GENERADO",
                    path_relative(&format!("{}/sandra_metrics_report.txt", &destino), &destino)
                );
                println!("{:=<80}\n", "");
            }
            Err(e) => {
                let msg = format!("Error crítico en el ciclo de carga: {}", e);
                eprintln!("\n{:=<80}", "");
                eprintln!(">>> ABORTO CRÍTICO DEL SISTEMA <<<");
                eprintln!("{}", msg);
                println!("{:=<80}\n", "");
                logger::log_error("KERNEL", &msg);
            }
        }
    } else {
        println!("Sistema en espera (use -x para ejecutar prueba inmediata).");
    }

    Ok(())
}
