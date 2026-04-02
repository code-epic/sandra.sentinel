use clap::{Arg, Command};

pub struct CliArgs {
    pub comparison_file: String,
    pub comparison_columns: Vec<usize>,
    pub origin_file: String,
    pub origin_columns: Vec<usize>,
    pub delimiter: char,
    pub output_dir: String,
    pub skip_header: bool,
    pub quiet: bool,
    pub verbose: bool,
}

pub fn parse_args() -> CliArgs {
    let matches = Command::new("SCF - Sandra Conciliation File")
        .version("1.0.0")
        .author("Carlos Peña")
        .about("Compara dos archivos basado en columnas específicas")
        .arg(
            Arg::new("comparacion")
                .help("Archivo de comparación")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("columnas_comparacion")
                .help("Columnas del archivo de comparación (ej. 0,1,2,3,4)")
                .short('c')
                .long("columnas-comparacion")
                .required(true),
        )
        .arg(
            Arg::new("origen")
                .help("Archivo de origen")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("columnas_origen")
                .help("Columnas del archivo de origen (ej. 0,1,2,3,4)")
                .short('o')
                .long("columnas-origen")
                .required(true),
        )
        .arg(
            Arg::new("delimiter")
                .help("Delimitador de columnas")
                .short('d')
                .long("delimiter")
                .default_value(";"),
        )
        .arg(
            Arg::new("output")
                .help("Directorio de salida")
                .short('O')
                .long("output")
                .default_value("out"),
        )
        .arg(
            Arg::new("skip-header")
                .help("Omitir primera línea (header)")
                .long("skip-header")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .help("Modo silencioso")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .help("Salida detallada")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let parse_columns =
        |s: &str| -> Vec<usize> { s.split(',').filter_map(|x| x.trim().parse().ok()).collect() };

    CliArgs {
        comparison_file: matches.get_one::<String>("comparacion").unwrap().clone(),
        comparison_columns: parse_columns(
            matches.get_one::<String>("columnas_comparacion").unwrap(),
        ),
        origin_file: matches.get_one::<String>("origen").unwrap().clone(),
        origin_columns: parse_columns(matches.get_one::<String>("columnas_origen").unwrap()),
        delimiter: matches
            .get_one::<String>("delimiter")
            .unwrap()
            .chars()
            .next()
            .unwrap_or(';'),
        output_dir: matches.get_one::<String>("output").unwrap().clone(),
        skip_header: matches.get_flag("skip-header"),
        quiet: matches.get_flag("quiet"),
        verbose: matches.get_flag("verbose"),
    }
}
