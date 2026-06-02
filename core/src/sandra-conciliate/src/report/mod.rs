use crate::error::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct ReportGenerator {
    output_dir: String,
}

impl ReportGenerator {
    pub fn new(output_dir: &str) -> Self {
        Self {
            output_dir: output_dir.to_string(),
        }
    }

    pub fn generate(&self) -> Result<()> {
        let files = vec!["compare.txt", "log.txt", "hallazgos.txt"];
        let report_path = Path::new(&self.output_dir).join("reporte.txt");
        let mut report = File::create(&report_path)?;

        writeln!(report, "----------------------------------------")?;
        writeln!(report, "Archivo\t\tLíneas\tPeso (bytes)")?;
        writeln!(report, "----------------------------------------")?;

        for file_name in files {
            let file_path = Path::new(&self.output_dir).join(file_name);
            if file_path.exists() {
                let lines = crate::io::count_lines(&file_path)?;
                let size = crate::io::file_size(&file_path)?;
                writeln!(report, "{}\t\t{}\t{}", file_name, lines, size)?;
            }
        }

        Ok(())
    }

    pub fn generate_json(&self) -> Result<String> {
        let mut output = String::from("{\n");
        let files = vec!["compare.txt", "log.txt", "hallazgos.txt"];

        for (i, file_name) in files.iter().enumerate() {
            let file_path = Path::new(&self.output_dir).join(file_name);
            let lines = if file_path.exists() {
                crate::io::count_lines(&file_path)?
            } else {
                0
            };
            let size = if file_path.exists() {
                crate::io::file_size(&file_path)?
            } else {
                0
            };

            output.push_str(&format!(
                "  \"{}\": {{\"lines\": {}, \"size\": {}}}",
                file_name.replace(".txt", ""),
                lines,
                size
            ));

            if i < files.len() - 1 {
                output.push(',');
            }
            output.push('\n');
        }

        output.push_str("}\n");
        Ok(output)
    }
}
