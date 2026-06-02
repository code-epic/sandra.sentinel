#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    use reconciler::cli::ReconcilerCli;
    use reconciler::run;

    let cli = ReconcilerCli::parse();
    let quiet = cli.quiet;
    let config = cli.into_config()?;
    run(config, quiet).await?;
    Ok(())
}
