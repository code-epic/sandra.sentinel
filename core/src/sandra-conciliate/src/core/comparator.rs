use crate::core::keys::KeyGenerator;
use crate::error::{Result, ScfError};
use std::collections::HashSet;
use std::io::{BufRead, BufReader};

pub struct Comparator {
    comparison_key_gen: KeyGenerator,
    origin_key_gen: KeyGenerator,
    comparison_keys: HashSet<String>,
}

impl Comparator {
    pub fn new(
        comparison_columns: Vec<usize>,
        origin_columns: Vec<usize>,
        delimiter: char,
    ) -> Self {
        Self {
            comparison_key_gen: KeyGenerator::new(comparison_columns, delimiter),
            origin_key_gen: KeyGenerator::new(origin_columns, delimiter),
            comparison_keys: HashSet::new(),
        }
    }

    pub fn with_capacity(
        comparison_columns: Vec<usize>,
        origin_columns: Vec<usize>,
        delimiter: char,
        capacity: usize,
    ) -> Self {
        Self {
            comparison_key_gen: KeyGenerator::new(comparison_columns, delimiter),
            origin_key_gen: KeyGenerator::new(origin_columns, delimiter),
            comparison_keys: HashSet::with_capacity(capacity),
        }
    }

    pub fn load_comparison_file(&mut self, path: &str) -> Result<usize> {
        let file =
            std::fs::File::open(path).map_err(|_| ScfError::FileNotFound(path.to_string()))?;

        let reader = BufReader::with_capacity(1024 * 1024, file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line.map_err(ScfError::from)?;
            if line.is_empty() {
                continue;
            }
            let key = self.comparison_key_gen.generate(&line)?;
            self.comparison_keys.insert(key);
            count += 1;
        }

        Ok(count)
    }

    pub fn load_comparison_lines(&mut self, lines: &[String]) -> Result<usize> {
        let count = lines.len();
        if self.comparison_keys.capacity() < count {
            self.comparison_keys.reserve(count);
        }

        for line in lines {
            if line.is_empty() {
                continue;
            }
            let key = self.comparison_key_gen.generate(line)?;
            self.comparison_keys.insert(key);
        }

        Ok(count)
    }

    #[inline]
    pub fn compare_line(&self, line: &str) -> Result<ComparisonResult> {
        let key = self.origin_key_gen.generate(line)?;

        if self.comparison_keys.contains(&key) {
            Ok(ComparisonResult::Match)
        } else {
            Ok(ComparisonResult::NoMatch)
        }
    }

    pub fn compare_lines<'a>(&self, lines: &'a [&str]) -> Vec<(&'a str, ComparisonResult)> {
        let mut results = Vec::with_capacity(lines.len());

        for line in lines {
            let result = self.compare_line(line).unwrap_or(ComparisonResult::NoMatch);
            results.push((*line, result));
        }

        results
    }

    pub fn keys_count(&self) -> usize {
        self.comparison_keys.len()
    }

    pub fn keys_capacity(&self) -> usize {
        self.comparison_keys.capacity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonResult {
    Match,
    NoMatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison() {
        let mut cmp = Comparator::new(vec![0, 1], vec![0, 1], ';');

        cmp.comparison_keys.insert("a-1".to_string());

        let result = cmp.compare_line("a;1;extra").unwrap();
        assert_eq!(result, ComparisonResult::Match);

        let result = cmp.compare_line("b;2;extra").unwrap();
        assert_eq!(result, ComparisonResult::NoMatch);
    }

    #[test]
    fn test_with_capacity() {
        let cmp = Comparator::with_capacity(vec![0], vec![0], ';', 1000);
        assert!(cmp.keys_capacity() >= 1000);
    }
}
