use sandra_core::kernel::logica::{exportador, logger, telemetria};
use sandra_core::System;

use sandra_core::model::Manifiesto;

pub async fn execute(
    execute: bool,
    log: bool,
    sensors: bool,
    manifest_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    logger::init(log);
    telemetria::init(sensors);

    // --- BANNER DE INICIO ---
    println!("\n{:=<80}", "");
    println!("{:^80}", "SANDRA SENTINEL - EJECUCIÓN DE NÓMINA");
    println!("{:=<80}", "");

    let mut system = System::init(); // Restaurado

    // Cargar manifiesto si se especificó
    if let Some(path) = manifest_path {
        println!("> Cargando manifiesto desde '{}'...", path);
        match Manifiesto::cargar_desde_archivo(&path) {
            Ok(m) => {
                println!("{:<20} : {}", "[CONFIG] Manifiesto", m.nombre);
                println!("{:<20} : {}", "[CICLO ] Periodo", m.ciclo);
                println!("{:<20} : {}", "[INFO  ] Descripción", m.descripcion);
                system.kernel.config = m;
            }
            Err(e) => {
                eprintln!("[ERROR] Cargando manifiesto: {}", e);
                return Err(e);
            }
        }
    } else {
        println!("{:<20} : {}", "[CONFIG] Modo", "Estándar (Sin manifiesto)");
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
        match system.kernel.ejecutar_ciclo_carga().await {
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
                    // EXPORTACION
                    let export_path = std::path::Path::new("nomina_exportada.csv");
                    let t_export = std::time::Instant::now();

                    if let Err(e) =
                        exportador::exportar_nomina_csv(&system.kernel.beneficiarios, export_path)
                    {
                        let msg = format!("Error exportando CSV: {}", e);
                        eprintln!("  {:<25} : {:>10}", "Exportación CSV", "FALLO");
                        eprintln!("    └─ [ERROR] {}", msg);
                        logger::log_error("EXPORT", &msg);
                    } else {
                        // Registrar métrica
                        let size_mb = if let Ok(meta) = std::fs::metadata(export_path) {
                            meta.len() as f64 / 1_048_576.0
                        } else {
                            0.0
                        };

                        telemetria::record(
                            "EXPORT",
                            "CSV Nómina",
                            t_export.elapsed(),
                            system.kernel.beneficiarios.len(),
                            &format!("{:.2} MB", size_mb),
                        );

                        println!(
                            "  {:<25} : {:>10} ({})",
                            "Exportación CSV",
                            "OK",
                            export_path.display()
                        );
                    }
                }

                // Generar reporte final de telemetría
                telemetria::generate_report();
                println!(
                    "  {:<25} : {:>10} (sandra_metrics_report.txt)",
                    "Reporte Sensores", "GENERADO"
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
