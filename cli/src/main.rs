use clap::{Parser, Subcommand};
use sandra_core::System;

#[derive(Parser)]
#[command(name = "sandra")]
#[command(author = "Equipo de Desarrollo Sandra")]
#[command(version = "1.0.0")]
#[command(
    about = "Sandra Sentinel - Motor de Cálculo de Nómina Militar",
    long_about = "Sandra Sentinel es el núcleo de procesamiento de nómina desarrollado en Rust.\n\nPermite la carga masiva de datos, ejecución de fórmulas dinámicas (Rhai) y exportación de resultados.\nDiseñado para alta concurrencia y tolerancia a fallos."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicia el ciclo de carga y cálculo de nómina.
    #[command(
        long_about = "Inicia el Core del sistema, conecta con Sandra Server y ejecuta el ciclo de nómina.\n\nEjemplos:\n  sandra start -x --log --sensors\n  sandra start --execute"
    )]
    Start {
        /// Ejecuta el ciclo de carga inmediatamente al iniciar.
        #[arg(short = 'x', long = "execute")]
        execute: bool,

        /// Habilita el registro de eventos en archivo ('sandra_sentinel.log').
        #[arg(long)]
        log: bool,

        /// Activa la recolección de métricas de rendimiento y genera reporte final (-s).
        #[arg(short = 's', long = "sensors")]
        sensors: bool,
    },

    /// Procesa cálculos de nómina en lote desde un archivo local (Offline).
    #[command(
        long_about = "Permite procesar un archivo JSON local con beneficiarios sin conectar al servidor.\nÚtil para pruebas de fórmulas o reprocesos manuales."
    )]
    Lote {
        /// Ruta al archivo JSON de entrada.
        #[arg(short, long)]
        archivo: Option<String>,
    },

    /// Muestra el estado de salud del sistema y recursos.
    Monitor,

    /// Valida claves de acceso y permisos de seguridad (Herramienta admin).
    Validar {
        /// Clave o Token a validar.
        #[arg(short, long)]
        clave: String,
    },

    /// Muestra la versión detallada del compilado.
    Version,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start {
            execute,
            log,
            sensors,
        }) => {
            // Inicializar logger y sensores
            sandra_core::kernel::logica::logger::init(*log);
            sandra_core::kernel::logica::telemetria::init(*sensors);

            println!("Inicializando Sentinel...");
            let mut system = System::init();

            // Conectar a Sandra (Golang)
            let url = system.config.get_url();
            if let Err(e) = system.connect_sandra(url).await {
                let msg = format!("Error conectando a Sandra Server: {}", e);
                eprintln!("{}", msg);
                sandra_core::kernel::logica::logger::log_error("CONEXION", &msg);
                return Ok(());
            }

            println!("Sentinel conectado y listo.");

            if *execute {
                let start = std::time::Instant::now();
                match system.kernel.ejecutar_ciclo_carga().await {
                    Ok(_) => {
                        let duration = start.elapsed();
                        println!("🚀 Ciclo de carga finalizado en {:.2?}.", duration);

                        sandra_core::kernel::logica::telemetria::record(
                            "SISTEMA",
                            "Ciclo Total",
                            duration,
                            system.kernel.beneficiarios.len(),
                            "Ciclo completo",
                        );

                        // Aquí podrías mostrar estadísticas
                        let len = system.kernel.beneficiarios.len();
                        if len > 0 {
                            // EXPORTACION
                            let export_path = std::path::Path::new("nomina_exportada.csv");
                            let t_export = std::time::Instant::now(); // Medir exportación

                            if let Err(e) =
                                sandra_core::kernel::logica::exportador::exportar_nomina_csv(
                                    &system.kernel.beneficiarios,
                                    export_path,
                                )
                            {
                                let msg = format!("Error exportando CSV: {}", e);
                                eprintln!("❌ {}", msg);
                                sandra_core::kernel::logica::logger::log_error("EXPORT", &msg);
                            } else {
                                // Registrar métrica de exportación
                                let size_mb = if let Ok(meta) = std::fs::metadata(export_path) {
                                    meta.len() as f64 / 1_048_576.0
                                } else {
                                    0.0
                                };

                                sandra_core::kernel::logica::telemetria::record(
                                    "EXPORT",
                                    "CSV Nómina",
                                    t_export.elapsed(),
                                    system.kernel.beneficiarios.len(),
                                    &format!("{:.2} MB", size_mb),
                                );

                                println!("✅ Nómina exportada a: {}", export_path.display());
                            }
                        }

                        // Generar reporte final de telemetría
                        sandra_core::kernel::logica::telemetria::generate_report();
                    }
                    Err(e) => {
                        let msg = format!("Error crítico en el ciclo de carga: {}", e);
                        eprintln!("{}", msg);
                        sandra_core::kernel::logica::logger::log_error("KERNEL", &msg);
                    }
                }
            } else {
                println!("Sistema en espera (use -x para ejecutar prueba inmediata).");
            }
        }
        Some(Commands::Lote { archivo }) => {
            let _system = System::init(); // Core necesario para cálculos
            match archivo {
                Some(path) => println!("Procesando lote desde: {}", path),
                None => println!("Procesando lote estándar..."),
            }
        }
        Some(Commands::Monitor) => {
            println!("Estado del Sistema: OK");
            println!("Memoria: 24MB"); // Ejemplo
            println!("Conexiones: 0");
        }
        Some(Commands::Validar { clave }) => {
            println!("Validando clave: {}", clave);
            // Lógica de validación dummy
            if clave == "1234" {
                println!("Clave VALIDA");
            } else {
                println!("Clave INVALIDA");
            }
        }
        Some(Commands::Version) => {
            println!("Sandra Sentinel v0.1.0");
        }
        None => {
            println!("Por favor usa --help para ver los comandos disponibles.");
        }
    }

    Ok(())
}
