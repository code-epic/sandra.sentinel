use std::io::Write;

use crate::error::Result;
use crate::types::ReconcilerConfig;

pub fn write_staging_script(out_dir: &str, config: &ReconcilerConfig) -> Result<()> {
    let tabla = match config.grpc_parametros.trim_matches('"') {
        "1" => "tmp_ejercito",
        "2" => "tmp_armada",
        "3" => "tmp_aviacion",
        "4" => "tmp_guardia",
        _ => return Ok(()),
    };

    let path = format!("{}/cargar_staging.sh", out_dir);
    let mut file = std::fs::File::create(&path)?;

    let delim_display: String = if config.delimiter == '\t' {
        "\\t".to_string()
    } else {
        config.delimiter.to_string()
    };

    writeln!(file, r##"#!/usr/bin/env bash
set -euo pipefail

ARCHIVO="${{1:-nuevos.csv}}"
: "${{PGHOST:={pg_host}}}"
: "${{PGPORT:={pg_port}}}"
: "${{PGUSER:={pg_user}}}"
: "${{PGDATABASE:={pg_database}}}"
: "${{PGPASSWORD:={pg_password}}}"

export PGHOST PGPORT PGUSER PGDATABASE PGPASSWORD"##,
        pg_host = config.pg_host,
        pg_port = config.pg_port,
        pg_user = config.pg_user,
        pg_database = config.pg_database,
        pg_password = config.pg_password,
    )?;

    writeln!(file, r##"
TABLA="{tabla}"

if [[ ! -f "$ARCHIVO" ]]; then
    echo "[ERROR] No existe: $ARCHIVO" >&2
    exit 1
fi

psql -X --echo-errors <<SQL
BEGIN;

DROP TABLE IF EXISTS ${{TABLA}};

CREATE TABLE ${{TABLA}} (
    cedula          TEXT PRIMARY KEY,
    grado           TEXT,
    n_hijos         TEXT,
    f_ingreso       TEXT,
    f_ult_ascenso   TEXT,
    st_profesion    TEXT,
    anio_reconocido TEXT,
    mes_reconocido  TEXT,
    dia_reconocido  TEXT,
    estatus         INTEGER DEFAULT 0
);

\copy ${{TABLA}}(cedula, grado, n_hijos, f_ingreso, f_ult_ascenso, st_profesion, anio_reconocido, mes_reconocido, dia_reconocido) FROM '${{ARCHIVO}}' WITH (FORMAT csv, DELIMITER '{delim}', HEADER true);

COMMIT;
SQL

echo "[OK] Cargados $(wc -l < "$ARCHIVO") registros en ${{TABLA}}"
"##, tabla = tabla, delim = delim_display)?;

    file.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

pub fn execute_staging_script(out_dir: &str, config: &ReconcilerConfig, quiet: bool) -> Result<()> {
    let tabla = match config.grpc_parametros.trim_matches('"') {
        "1" => "tmp_ejercito",
        "2" => "tmp_armada",
        "3" => "tmp_aviacion",
        "4" => "tmp_guardia",
        _ => return Ok(()),
    };

    let script_path = format!("{}/cargar_staging.sh", out_dir);
    let csv_path = format!("{}/nuevos.csv", out_dir);

    if !std::path::Path::new(&script_path).exists() || !std::path::Path::new(&csv_path).exists() {
        return Ok(());
    }

    let output = std::process::Command::new("bash")
        .arg("./cargar_staging.sh")
        .arg("nuevos.csv")
        .current_dir(out_dir)
        .env("PGHOST", &config.pg_host)
        .env("PGPORT", &config.pg_port)
        .env("PGUSER", &config.pg_user)
        .env("PGDATABASE", &config.pg_database)
        .env("PGPASSWORD", &config.pg_password)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                if !quiet {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    for line in stdout.lines() {
                        if line.starts_with("[OK]") {
                            println!("  Staging PG:      {}", line);
                        }
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!(
                    "[WARN] Error cargando staging en PostgreSQL ({}): {}",
                    tabla,
                    stderr.trim()
                );
            }
        }
        Err(e) => {
            eprintln!("[WARN] No se pudo ejecutar cargar_staging.sh: {}", e);
        }
    }

    Ok(())
}
