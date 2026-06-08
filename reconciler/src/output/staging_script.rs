use std::io::Write;

use crate::error::Result;

pub fn write_staging_script(out_dir: &str, parametro: &str, delimiter: char) -> Result<()> {
    let tabla = match parametro {
        "1" => "tmp_ejercito",
        "2" => "tmp_armada",
        "3" => "tmp_aviacion",
        "4" => "tmp_guardia",
        _ => return Ok(()),
    };

    let path = format!("{}/cargar_staging.sh", out_dir);
    let mut file = std::fs::File::create(&path)?;

    let delim_display: String = if delimiter == '\t' {
        "\\t".to_string()
    } else {
        delimiter.to_string()
    };

    writeln!(file, r##"#!/usr/bin/env bash
set -euo pipefail

ARCHIVO="${{1:-nuevos.csv}}"
: "${{PGHOST:=localhost}}"
: "${{PGPORT:=5432}}"
: "${{PGUSER:=postgres}}"
: "${{PGDATABASE:=pace}}"
: "${{PGPASSWORD:=postgres}}"

export PGHOST PGPORT PGUSER PGDATABASE PGPASSWORD

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
