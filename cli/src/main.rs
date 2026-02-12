use clap::{Parser, Subcommand};
use sandra_core::System;

#[derive(Parser)]
#[command(name = "sandra")]
#[command(about = "CLI para Sandra Sentinel", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicia el Core y sus módulos
    Start {
        #[arg(short = 'x', long)]
        execute: bool,
    },
    /// Procesa cálculos en lote
    Lote {
        #[arg(short, long)]
        archivo: Option<String>,
    },
    /// Monitor de salud del sistema
    Monitor,
    /// Valida claves y accesos
    Validar {
        #[arg(short, long)]
        clave: String,
    },
    /// Muestra la versión del sistema
    Version,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start { execute }) => {
            println!("Inicializando Sentinel...");
            let mut system = System::init();

            // Conectar a Sandra (Golang)
            let url = system.config.get_url();
            if let Err(e) = system.connect_sandra(url).await {
                eprintln!("Error conectando a Sandra Server: {}", e);
                return Ok(());
            }

            println!("Sentinel conectado y listo.");

            if *execute {
                let start = std::time::Instant::now();
                match system.kernel.ejecutar_ciclo_carga().await {
                    Ok(_) => {
                        let duration = start.elapsed();
                        println!("🚀 Ciclo de carga finalizado en {:.2?}.", duration);
                        // Aquí podrías mostrar estadísticas o detalles desde system.kernel.beneficiarios
                        println!("--- Muestra de 5 Beneficiarios (Distribuidos) ---");
                        let len = system.kernel.beneficiarios.len();
                        if len > 0 {
                            // let step = if len > 2 { len / 2 } else { 1 };
                            // for i in (0..len).step_by(step).take(2) {
                            //     if let Ok(json_str) =
                            //         serde_json::to_string_pretty(&system.kernel.beneficiarios[i])
                            //     {
                            //         println!("Beneficiario [{}]:\n{}", i, json_str);
                            //     }
                            // }

                            // EXPORTACION
                            let export_path = std::path::Path::new("nomina_exportada.csv");
                            if let Err(e) =
                                sandra_core::kernel::logica::exportador::exportar_nomina_csv(
                                    &system.kernel.beneficiarios,
                                    export_path,
                                )
                            {
                                eprintln!("❌ Error exportando CSV: {}", e);
                            } else {
                                println!("✅ Nómina exportada a: {}", export_path.display());
                            }
                        }
                    }
                    Err(e) => eprintln!("Error en el ciclo de carga: {}", e),
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
