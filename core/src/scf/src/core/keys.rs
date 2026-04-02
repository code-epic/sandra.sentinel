use crate::error::{Result, ScfError};

pub struct KeyGenerator {
    columns: Vec<usize>,
    delimiter: char,
}

impl KeyGenerator {
    pub fn new(columns: Vec<usize>, delimiter: char) -> Self {
        Self { columns, delimiter }
    }

    pub fn generate(&self, line: &str) -> Result<String> {
        let columns: Vec<&str> = line.split(self.delimiter).collect();

        if columns.len() < self.columns.len() {
            return Err(ScfError::Parse(format!(
                "Línea con {} columnas, esperado al menos {}",
                columns.len(),
                self.columns.len()
            )));
        }

        let key = self
            .columns
            .iter()
            .map(|&col| columns.get(col).copied().unwrap_or(""))
            .collect::<Vec<&str>>()
            .join("-");

        Ok(key)
    }

    pub fn generate_from_split(&self, columns: &[&str]) -> Result<String> {
        if columns.len() < self.columns.len() {
            return Err(ScfError::Parse(format!(
                "Columnas insuficientes: {} < {}",
                columns.len(),
                self.columns.len()
            )));
        }

        let key = self
            .columns
            .iter()
            .map(|&col| columns[col])
            .collect::<Vec<&str>>()
            .join("-");

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let gen = KeyGenerator::new(vec![0, 1, 2], ';');
        let key = gen.generate("123;456;789;abc").unwrap();
        assert_eq!(key, "123-456-789");
    }

    #[test]
    fn test_generate_key_less_columns() {
        let gen = KeyGenerator::new(vec![0, 1, 2], ';');
        let result = gen.generate("123;456");
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_delimiter() {
        let gen = KeyGenerator::new(vec![0, 1], ',');
        let key = gen.generate("a,b,c").unwrap();
        assert_eq!(key, "a-b");
    }
}
