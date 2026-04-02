use std::io::{BufWriter, Write};
use std::time::Instant;
use scf::{Comparator, ComparisonResult, ReportGenerator};

pub async fn execute(
    comparison: String,
    comparison_columns: String,
    origin: String,
    origin_columns: String,
    delimiter: String,
    output: String,
    skip_header: bool,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    if !quiet {
        println!("SCF - Sandra Conciliation File v1.0.0");
        println!("========================================");
    }

    // Parsear columnas
    let comp_cols: Vec<usize> = comparison_columns
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let orig_cols: Vec<usize> = origin_columns
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if comp_cols.is_empty() || orig_cols.is_empty() {
        return Err("Las columnas no pueden estar vacías".into());
    }

    let delim = delimiter.chars().next().unwrap_or(';');

    // Crear comparador
    let mut comparator = Comparator::new(comp_cols, orig_cols, delim);

    // Cargar archivo de comparación
    if !quiet {
        println!("Cargando archivo de comparación: {}", comparison);
    }
    let count = comparator.load_comparison_file(&comparison)?;
    if !quiet {
        println!("{} claves cargadas\n", count);
    }

    // Crear directorio de salida
    scf::io::create_output_dir(&output)?;

    // Procesar archivo origen
    let reader = scf::io::FileReader::new(delim, skip_header);
    let lines = reader.read_all(&origin)?;
    let total = lines.len();

    if !quiet {
        println!("Procesando {} líneas...", total);
    }

    let compare_file = std::fs::File::create(format!("{}/coincidencias.txt", output))?;
    let hallazgos_file = std::fs::File::create(format!("{}/hallazgos.txt", output))?;
    let log_file = std::fs::File::create(format!("{}/log.txt", output))?;

    let mut c_writer = BufWriter::new(compare_file);
    let mut h_writer = BufWriter::new(hallazgos_file);
    let mut l_writer = BufWriter::new(log_file);

    let mut coincidencias = 0;
    let mut hallazgos = 0;

    for (idx, line) in lines.iter().enumerate() {
        match comparator.compare_line(line) {
            Ok(ComparisonResult::Match) => {
                writeln!(c_writer, "{}", line)?;
                coincidencias += 1;
            }
            Ok(ComparisonResult::NoMatch) => {
                writeln!(h_writer, "{}", line)?;
                hallazgos += 1;
            }
            Err(e) => {
                writeln!(l_writer, "Línea {}: {}", idx + 1, e)?;
            }
        }

        if !quiet && (idx + 1) % 10000 == 0 {
            println!("  Procesadas: {}/{}", idx + 1, total);
        }
    }

    c_writer.flush()?;
    h_writer.flush()?;
    l_writer.flush()?;

    // Generar reporte
    let reporter = ReportGenerator::new(&output);
    reporter.generate()?;

    let elapsed = start.elapsed();

    if !quiet {
        println!("\n=== RESULTADOS ===");
        println!("Coincidencias:  {}", coincidencias);
        println!("Hallazgos:      {}", hallazgos);
        println!("Tiempo:         {:.2?}", elapsed);
        println!("\nArchivos generados en: {}/", output);
    }

    Ok(())
}
