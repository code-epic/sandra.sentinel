// =============================================================================
// COMANDO: NOMINA PATRIA
// =============================================================================
// Genera archivos de nómina para el Sistema Patria
// 
// Uso:
//   sandra patria --manifest nomina_patria.json --conciliacion
//   sandra patria -m nomina_patria.json
// =============================================================================

use sandra_core::kernel::logica::cargador::Cargador;
use sandra_core::kernel::logica::memoria::FiniquitoPatria;
use sandra_core::kernel::logica::{logger, telemetria};
use sandra_core::model::Manifiesto;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use sha2::{Sha256, Digest};

/// Ejecuta el proceso de Nómina Patria
pub async fn execute(
    manifest_path: Option<String>,
    conciliacion: bool,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar sensor de tiempo
    let start_time = std::time::Instant::now();
    
    // Debug flag
    if debug {
        std::env::set_var("SANDRA_DEBUG", "1");
        println!("[DEBUG] Modo debug habilitado");
    }
    
    // Inicializar logging
    logger::init(true, "out");
    telemetria::init(true);
    
    // Banner de inicio
    println!("\n{:=<80}", "");
    println!("{:^80}", "SANDRA SENTINEL - NOMINA PATRIA");
    println!("{:^80}", "");
    println!("{:=<80}", "");

    logger::log_info("PATRIA", "Iniciando proceso de Nómina Patria");

    // Cargar manifisto
    let config = if let Some(path) = manifest_path {
        println!("> Cargando manifsto desde '{}'...", path);
        logger::log_info("MANIFEST", &format!("Cargando desde: {}", path));
        Manifiesto::cargar_desde_archivo(&path)?
    } else {
        let err = "Manifsto requerido";
        logger::log_error("PATRIA", err);
        return Err(err.into());
    };

    println!("{:<20} : {}", "[CONFIG] Manifsto", config.nombre);
    println!("{:<20} : {}", "[CICLO]", config.ciclo);
    println!("{:<20} : {}", "[DESCRIPCION]", config.descripcion);
    logger::log_info("CONFIG", &format!("Ciclo: {}", config.ciclo));
    println!("{:-<80}", "");

    // Obtener parámetros del manifisto desde parametros_globales
    let fecha_desde = config.parametros_globales
        .get("fecha_desde")
        .cloned()
        .unwrap_or_else(|| "2025-01-01".to_string());

    let fecha_hasta = config.parametros_globales
        .get("fecha_hasta")
        .cloned()
        .unwrap_or_else(|| "2025-12-31".to_string());

    println!("> Rango de fechas: {} a {}", fecha_desde, fecha_hasta);
    logger::log_info("FECHAS", &format!("{} -> {}", fecha_desde, fecha_hasta));
    println!("{:-<80}", "");

    // Inicializar Cargador (usando config del manifisto)
    let mut cargardor = Cargador::new(config.clone());

    // URL de Sandra Server (desde variable de entorno o默认值)
    let url = std::env::var("SANDRA_URL")
        .unwrap_or_else(|_| "http://localhost:50051".to_string());
    println!("> Conectando a Sandra Server en {}...", url);
    logger::log_info("CONEXION", &format!("Sandra Server: {}", url));
    
    // Intentar conexion (si falla, continuamos en modo demo)
    let cargadora_exito = match cargardor.connect(url.clone()).await {
        Ok(_) => true,
        Err(e) => {
            logger::log_error("CONEXION", &format!("Error: {}", e));
            println!("[WARN] No se pudo conectar a Sandra Server: {}", e);
            println!("[INFO] Ejecutando en modo DEMO (sin datos)");
            false
        }
    };

    // Cargar datos o modo demo
    let finiquitos: Vec<FiniquitoPatria> = if cargadora_exito {
        println!("> Cargando finiquitos (IPSFA_CFiniquitosNomina)...");
        logger::log_info("CARGA", "IPSFA_CFiniquitosNomina");
        match cargardor.cargar_finiquitos_patria().await {
            Ok(datos) => datos,
            Err(e) => {
                logger::log_error("CARGA", &format!("Error: {}", e));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    println!("  Total registros cargados: {}", finiquitos.len());
    println!("{:-<80}", "");

    // Filtrar y transformar
    let mut lineas_txt = Vec::new();
    let mut monto_total = 0.0;
    let mut registros_validos = 0;

    // Indicadores de conciliación
    let mut negativos = Vec::new();
    let mut sin_cuenta = Vec::new();
    let mut invalidds = Vec::new();

    for fq in &finiquitos {
        // Validar reglas
        let es_valido = fq.es_valido();

        if es_valido {
            // Agregar al TXT
            lineas_txt.push(fq.to_line_patria());
            monto_total += fq.monto;
            registros_validos += 1;
        } else {
            // Clasificar para indicadores
            if fq.monto <= 0.0 {
                negativos.push(fq.clone());
            } else if !fq.numero_cuenta.starts_with("0102") {
                sin_cuenta.push(fq.clone());
            } else {
                invalidds.push(fq.clone());
            }
        }
    }

    println!("> Registros válidos: {}", registros_validos);
    println!("> Monto total: {:.2} Bs", monto_total);
    println!("{:-<80}", "");

    // Generar indicadores si se solicita
    if conciliacion {
        println!("[CONCILIACION] Generando reportes...");

        // Reporte: Montos negativos
        println!("  - Negativos: {}", negativos.len());
        if !negativos.is_empty() {
            let mut file = File::create("out/patria_negativos.csv")?;
            writeln!(file, "cedula,apellidos,numero_cuenta,monto")?;
            for fq in &negativos {
                writeln!(file, "{},{},{},{}", fq.cedula, fq.apellidos, fq.numero_cuenta, fq.monto)?;
            }
        }

        // Reporte: Sin cuenta
        println!("  - Sin cuenta 0102: {}", sin_cuenta.len());
        if !sin_cuenta.is_empty() {
            let mut file = File::create("out/patria_sin_cuenta.csv")?;
            writeln!(file, "cedula,apellidos,numero_cuenta,monto")?;
            for fq in &sin_cuenta {
                writeln!(file, "{},{},{},{}", fq.cedula, fq.apellidos, fq.numero_cuenta, fq.monto)?;
            }
        }

        // Reporte: Inválidos
        println!("  - Inválidos: {}", invalidds.len());
        if !invalidds.is_empty() {
            let mut file = File::create("out/patria_invalidos.csv")?;
            writeln!(file, "cedula,apellidos,numero_cuenta,monto")?;
            for fq in &invalidds {
                writeln!(file, "{},{},{},{}", fq.cedula, fq.apellidos, fq.numero_cuenta, fq.monto)?;
            }
        }
    }

    // Generar archivo TXT para Patria
    let nombre_txt = format!("patria_{}.txt", cargardor.config.ciclo);
    
    // Usar destino del manifisto
    let destino = &cargardor.config.salida.destino;
    let usar_compresion = config.salida.compresion;
    
    // Crear directorio destino
    let salida_dir = Path::new(destino);
    if !salida_dir.exists() {
        std::fs::create_dir_all(salida_dir)?;
    }
    
    let ruta_txt = salida_dir.join(&nombre_txt);
    
    // Escribir archivo TXT
    let mut file = File::create(&ruta_txt)?;
    
    // Encabezado (primera línea)
    let rif = "J0000000001"; // RIF IPSFA
    let cantidad = format!("{:0>7}", lineas_txt.len());
    let monto_str = format!("{:015.0}", monto_total * 100.0);
    let fecha_pago = fecha_hasta.replace("-", ""); // YYYYMMDD
    
    let encabezado = format!(
        "ONTNOM{}{}{}VES{}",
        rif,
        cantidad,
        monto_str,
        fecha_pago
    );
    writeln!(file, "{}", encabezado)?;

    // Detalle (registros)
    for linea in &lineas_txt {
        writeln!(file, "{}", linea)?;
    }
    file.flush()?;
    drop(file);
    
    // Calcular SHA256 del archivo TXT
    let mut file = File::open(&ruta_txt)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    drop(file);
    
    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let sha256 = format!("{:x}", hasher.finalize());
    
    // Generar manifest de guia (sin raw string para evitar errores)
    let fecha_gen = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    let manifest_guia = serde_json::json!({
        "nombre": nombre_txt,
        "ciclo": cargardor.config.ciclo,
        "sha256": sha256,
        "registros": lineas_txt.len(),
        "monto": monto_total,
        "fecha_generacion": fecha_gen,
        "compresion": usar_compresion
    }).to_string();
    
    // Escribir manifest de guia
    let guia_path = salida_dir.join(format!("manifesto_{}.json", cargardor.config.ciclo));
    let mut file_guia = File::create(&guia_path)?;
    file_guia.write_all(manifest_guia.as_bytes())?;
    file_guia.flush()?;
    drop(file_guia);
    
    // Si compression habilitada, generar ZSTD
    let mut nombre_archivo = nombre_txt.clone();
    if usar_compresion {
        let nombre_zst = format!("patria_{}.zst", cargardor.config.ciclo);
        let ruta_zst = salida_dir.join(&nombre_zst);
        
        // Comprimir con zstd (nivel 3)
        let encoded = zstd::encode_all(buffer.as_slice(), 3).map_err(|e| format!("ZSTD error: {}", e))?;
        
        let mut file_zst = File::create(&ruta_zst)?;
        file_zst.write_all(&encoded)?;
        file_zst.flush()?;
        drop(file_zst);
        
        // Calcular SHA256 del archivo ZSTD
        let mut file_zst = File::open(&ruta_zst)?;
        let mut buffer_zst = Vec::new();
        file_zst.read_to_end(&mut buffer_zst)?;
        drop(file_zst);
        
        let mut hasher_zst = Sha256::new();
        hasher_zst.update(&buffer_zst);
        let sha256_zst = format!("{:x}", hasher_zst.finalize());
        
        // Actualizar manifest con SHA256 del ZSTD
        let manifest_compressed = serde_json::json!({
            "nombre": nombre_zst,
            "ciclo": cargardor.config.ciclo,
            "sha256_txt": sha256,
            "sha256_zst": sha256_zst,
            "registros": lineas_txt.len(),
            "monto": monto_total,
            "fecha_generacion": fecha_gen,
            "compresion": true,
            "formato": "zstd"
        }).to_string();
        
        let mut file_guia = File::create(&guia_path)?;
        file_guia.write_all(manifest_compressed.as_bytes())?;
        
        nombre_archivo = nombre_zst;
        
        // Eliminar archivo TXT original si hay compresión
        if usar_compresion {
            if let Err(e) = std::fs::remove_file(&ruta_txt) {
                println!("[WARN] No se pudo eliminar TXT: {}", e);
            } else {
                println!("[INFO] Archivo TXT eliminado (solo ZST)");
            }
        }
    }
    
    // Logging final
    logger::log_info("SALIDA", &format!("Archivo: {}", nombre_archivo));
    logger::log_info("REGISTROS", &format!("Total: {}", lineas_txt.len()));
    logger::log_info("MONTO", &format!("Total: {:.2} Bs", monto_total));
    
    // Telemetry: registrar metrics finales
    let elapsed = start_time.elapsed();
    telemetria::record(
        "PATRIA", 
        "GENERACION", 
        elapsed, 
        lineas_txt.len(), 
        &format!("monto:{},negativos:{},sin_cuenta:{}", monto_total, negativos.len(), sin_cuenta.len())
    );

    // Generar reporte de telemetria
    if telemetria::is_enabled() {
        telemetria::generate_report("out");
    }

    println!("{:=<80}", "");
    println!("[OK] Archivo generado: {}", nombre_archivo);
    println!("  - Registros: {}", lineas_txt.len());
    println!("  - Monto: {:.2} Bs", monto_total);
    println!("  - Tiempo: {:?}", elapsed);
    if conciliacion {
        println!("  - Indicadores:");
        println!("    * Negativos: {}", negativos.len());
        println!("    * Sin cuenta 0102: {}", sin_cuenta.len());
        println!("    * Inválidos: {}", invalidds.len());
    }
    println!("{:=<80}", "");

    Ok(())
}