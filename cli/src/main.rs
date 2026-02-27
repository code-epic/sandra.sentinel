use clap::{Parser, Subcommand};

mod commands;

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

        /// Ruta a un archivo de manifiesto (.json) para configurar la ejecución.
        #[arg(short = 'm', long = "manifest")]
        manifest: Option<String>,
    },

    /// Procesa conciliación de nómina desde un archivo local.
    #[command(
        long_about = "Permite procesar archivos de nómina para validación y conciliación manual.\nAnteriormente conocido como modo Lote."
    )]
    Conciliacion {
        /// Ruta al archivo JSON de entrada.
        #[arg(short, long)]
        archivo: Option<String>,
    },

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
            manifest,
        }) => {
            commands::start::execute(*execute, *log, *sensors, manifest.clone()).await?;
        }
        Some(Commands::Conciliacion { archivo }) => {
            commands::conciliacion::execute(archivo.clone()).await;
        }
        Some(Commands::Validar { clave }) => {
            commands::validar::execute(clave.clone());
        }
        Some(Commands::Version) => {
            commands::version::execute();
        }
        None => {
            println!("Por favor usa --help para ver los comandos disponibles.");
        }
    }

    Ok(())
}
