use clap::{Parser, Subcommand, ValueEnum};
use sandra_core::tipos::TipoNomina;

mod commands;

#[derive(Clone, Copy, Debug)]
pub enum TipoNominaCli {
    Npr,
    Nact,
    Nrcp,
    Nfcp,
    // Nómina Patria
    Npat,
}

impl From<TipoNominaCli> for TipoNomina {
    fn from(t: TipoNominaCli) -> Self {
        match t {
            TipoNominaCli::Npr => TipoNomina::Npr,
            TipoNominaCli::Nact => TipoNomina::Nact,
            TipoNominaCli::Nrcp => TipoNomina::Nrcp,
            TipoNominaCli::Nfcp => TipoNomina::Nfcp,
            TipoNominaCli::Npat => TipoNomina::Npat,
        }
    }
}

impl ValueEnum for TipoNominaCli {
    fn value_variants<'a>() -> &'a [Self] {
        &[TipoNominaCli::Npr, TipoNominaCli::Nact, TipoNominaCli::Nrcp, TipoNominaCli::Nfcp, TipoNominaCli::Npat]
    }

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        match input.to_lowercase().as_str() {
            "npr" => Ok(TipoNominaCli::Npr),
            "nact" => Ok(TipoNominaCli::Nact),
            "nrcp" => Ok(TipoNominaCli::Nrcp),
            "nfcp" => Ok(TipoNominaCli::Nfcp),
            "npat" => Ok(TipoNominaCli::Npat),
            _ => Err(format!("Unknown tipo: {}", input)),
        }
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(match self {
            TipoNominaCli::Npr => "npr",
            TipoNominaCli::Nact => "nact",
            TipoNominaCli::Nrcp => "nrcp",
            TipoNominaCli::Nfcp => "nfcp",
            TipoNominaCli::Npat => "npat",
        }))
    }
}

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

        /// Tipo de nómina a generar.
        #[arg(short = 't', long, value_enum, default_value = "npr")]
        tipo: TipoNominaCli,

        /// Activa mensajes de debug para depuración.
        #[arg(short = 'd', long = "debug")]
        debug: bool,
    },

    /// Compara archivos CSV para conciliación bancaria.
    #[command(
        long_about = "Compara dos archivos CSV para identificar coincidencias y hallazgos.\n\nEjemplo:\n  sandra conciliate --comparison banco.csv --comparison-columns 0,1,2 --origin sistema.csv --origin-columns 0,1,3"
    )]
    Conciliate {
        /// Archivo de comparación (ej. movimientos del banco)
        #[arg(short, long)]
        comparison: String,

        /// Columnas a usar como clave en el archivo de comparación (ej. "0,1,2")
        #[arg(long)]
        comparison_columns: String,

        /// Archivo de origen (ej. registros del sistema)
        #[arg(short, long)]
        origin: String,

        /// Columnas del archivo origen
        #[arg(long)]
        origin_columns: String,

        /// Delimitador (default: ";")
        #[arg(long, default_value = ";")]
        delimiter: String,

        /// Directorio de salida (default: "out")
        #[arg(long, default_value = "out")]
        output: String,

        /// Omitir primera línea (header)
        #[arg(long)]
        skip_header: bool,

        /// Modo silencioso
        #[arg(short, long)]
        quiet: bool,
    },

    /// Genera Nómina Patria para el Sistema de Patrimonio.
    #[command(
        long_about = "Genera archivo de nómina para cargar en el Sistema Patria.\n\n\
        Este proceso genera finiquitos de asignación de antigüedad en formato TXT para carga en el Sistema Patria.\n\n\
        Parámetros (via manifisto):\n\
        - fecha_desde: Fecha inicial del rango (YYYY-MM-DD)\n\
        - fecha_hasta: Fecha final del rango (YYYY-MM-DD)\n\
        - conciliacion: Generar reportes de control (default: false)\n\n\
        Ejemplo:\n  sandra patria --manifest nomina_patria.json --tipo npat"
    )]
    Patria {
        /// Ruta al archivo de manifiesto (.json) con configuración.
        #[arg(short = 'm', long = "manifest")]
        manifest: Option<String>,

        /// Generar reportes de conciliación (montos negativos, sin cuenta, duplicados).
        #[arg(short = 'c', long = "conciliacion")]
        conciliacion: bool,

        /// Activa mensajes de debug para depuración.
        #[arg(short = 'd', long = "debug")]
        debug: bool,
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
            tipo,
            debug,
        }) => {
            commands::start::execute(*execute, *log, *sensors, manifest.clone(), (*tipo).into(), *debug).await?;
        }
        Some(Commands::Conciliate {
            comparison,
            comparison_columns,
            origin,
            origin_columns,
            delimiter,
            output,
            skip_header,
            quiet,
        }) => {
            commands::conciliate::execute(
                comparison.clone(),
                comparison_columns.clone(),
                origin.clone(),
                origin_columns.clone(),
                delimiter.clone(),
                output.clone(),
                *skip_header,
                *quiet,
            ).await?;
        }
        Some(Commands::Conciliacion { archivo }) => {
            commands::conciliacion::execute(archivo.clone()).await;
        }
        Some(Commands::Validar { clave }) => {
            commands::validar::execute(clave.clone());
        }
Some(Commands::Patria { manifest, conciliacion, debug }) => {
            // Ejecutar comando Patria
            commands::patria::execute(manifest.clone(), *conciliacion, *debug).await?;
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
