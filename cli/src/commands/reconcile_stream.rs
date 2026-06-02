use std::path::PathBuf;

use reconciler::cli::ReconcilerCli;
use reconciler::run;

pub async fn execute(
    csv_path: String,
    grpc_url: String,
    grpc_function: String,
    grpc_parametros: String,
    chunk_size: usize,
    output_dir: String,
    delimiter: String,
    skip_header: bool,
    field_mapping: Option<String>,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let field_mapping_path = field_mapping.map(PathBuf::from);

    let args = ReconcilerCli {
        csv_path: PathBuf::from(csv_path),
        grpc_url,
        grpc_function,
        grpc_parametros,
        chunk_size,
        output_dir: PathBuf::from(output_dir),
        delimiter,
        skip_header,
        field_mapping: field_mapping_path,
        quiet,
        debug: false,
    };

    let config = args.into_config()?;
    run(config, quiet).await?;
    Ok(())
}
