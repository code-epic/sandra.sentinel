use std::io::{BufWriter, Write};
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use scf::{parse_args, Comparator, ReportGenerator};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> scf::Result<()> {
    let args = parse_args();
    let start_time = Instant::now();

    if !args.quiet {
        println!("SCF - Sandra Conciliation File v1.0.0");
        println!("Archivos: {} vs {}", args.comparison_file, args.origin_file);
        println!("Delimiter: {}", args.delimiter);
        println!("Output: {}/\n", args.output_dir);
    }

    scf::io::create_output_dir(&args.output_dir)?;

    let mut comparator = Comparator::new(
        args.comparison_columns.clone(),
        args.origin_columns.clone(),
        args.delimiter,
    );

    if args.verbose {
        println!("Cargando archivo de comparación: {}", args.comparison_file);
    }
    let comparison_count = comparator.load_comparison_file(&args.comparison_file)?;
    if !args.quiet {
        println!("Claves de comparación cargadas: {}", comparison_count);
    }

    let compare_path = format!("{}/compare.txt", args.output_dir);
    let hallazgos_path = format!("{}/hallazgos.txt", args.output_dir);
    let log_path = format!("{}/log.txt", args.output_dir);

    let compare_file = std::fs::File::create(&compare_path)?;
    let hallazgos_file = std::fs::File::create(&hallazgos_path)?;
    let log_file = std::fs::File::create(&log_path)?;

    let mut compare_writer = BufWriter::new(compare_file);
    let mut hallazgos_writer = BufWriter::new(hallazgos_file);
    let mut log_writer = BufWriter::new(log_file);

    let reader = scf::io::FileReader::new(args.delimiter, args.skip_header);
    let origin_lines = reader.read_all(&args.origin_file)?;
    let total_lines = origin_lines.len();

    let pb = if !args.quiet {
        let pb = ProgressBar::new(total_lines as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len}")
                .expect("Invalid progress bar template"),
        );
        pb
    } else {
        ProgressBar::hidden()
    };

    let lines_ref: Vec<&str> = origin_lines.iter().map(|s| s.as_str()).collect();

    let chunks: Vec<Vec<&str>> = lines_ref.chunks(1000).map(|c| c.to_vec()).collect();

    let results: Vec<(&str, scf::ComparisonResult)> = chunks
        .par_iter()
        .flat_map(|chunk| comparator.compare_lines(chunk))
        .collect();

    let mut match_count = 0;
    let mut no_match_count = 0;

    for (line, result) in results {
        match result {
            scf::ComparisonResult::Match => {
                writeln!(compare_writer, "{}", line)?;
                match_count += 1;
            }
            scf::ComparisonResult::NoMatch => {
                writeln!(hallazgos_writer, "{}", line)?;
                no_match_count += 1;
            }
        }
        pb.inc(1);
    }

    compare_writer.flush()?;
    hallazgos_writer.flush()?;
    log_writer.flush()?;
    pb.finish();

    let reporter = ReportGenerator::new(&args.output_dir);
    reporter.generate()?;

    let elapsed = start_time.elapsed();

    if !args.quiet {
        println!("\n=== RESULTADOS ===");
        println!("Coincidencias:  {}", match_count);
        println!("Hallazgos:      {}", no_match_count);
        println!("Tiempo:         {:.2?}", elapsed);
        println!("\nReporte generado en: {}/", args.output_dir);
    }

    Ok(())
}
