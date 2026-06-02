use std::collections::HashMap;

use crate::error::Result;
use crate::types::CsvRecord;

pub struct CsvIndex {
    pub inner: HashMap<String, CsvRecord>,
    pub headers: Vec<String>,
    pub total_lines: usize,
    pub warnings: Vec<String>,
}

pub fn build_index(
    data: &[u8],
    delimiter: char,
    _skip_header: bool,
    _cedula_column: usize,
) -> Result<CsvIndex> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(true)
        .from_reader(data);

    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();

    let mut inner = HashMap::new();
    let mut total_lines = 0;
    let mut warnings = vec![];

    let delim_str = delimiter.to_string();

    for (idx, result) in rdr.records().enumerate() {
        total_lines += 1;
        let record = result?;

        let mut fields = HashMap::new();
        for (i, header) in headers.iter().enumerate() {
            if let Some(val) = record.get(i) {
                fields.insert(header.clone(), val.to_string());
            }
        }

        let cedula_raw = record.get(0).unwrap_or("");
        let cedula_norm: String = cedula_raw.chars().filter(|c| c.is_ascii_digit()).collect();

        if cedula_norm.is_empty() {
            warnings.push(format!("Fila {}: cédula vacía o inválida '{}'", idx + 2, cedula_raw));
            continue;
        }

        let raw_line = record.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(&delim_str);
        let csv_record = CsvRecord { fields, raw_line };
        inner.insert(cedula_norm, csv_record);
    }

    Ok(CsvIndex { inner, headers, total_lines, warnings })
}
