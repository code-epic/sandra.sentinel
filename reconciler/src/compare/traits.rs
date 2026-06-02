use chrono::NaiveDate;

use crate::types::ComparisonStrategy;

#[derive(Debug, Clone)]
pub enum FieldComparison {
    Match,
    Mismatch { expected: String, actual: String },
    Ignored,
}

pub fn compare_values(csv_val: &str, grpc_val: &str, strategy: &ComparisonStrategy) -> FieldComparison {
    match strategy {
        ComparisonStrategy::Exact => {
            if csv_val.trim() == grpc_val.trim() {
                FieldComparison::Match
            } else {
                FieldComparison::Mismatch {
                    expected: csv_val.to_string(),
                    actual: grpc_val.to_string(),
                }
            }
        }
        ComparisonStrategy::Normalized => {
            let a = csv_val.trim().to_lowercase();
            let b = grpc_val.trim().to_lowercase();
            if a == b {
                FieldComparison::Match
            } else {
                FieldComparison::Mismatch {
                    expected: csv_val.to_string(),
                    actual: grpc_val.to_string(),
                }
            }
        }
        ComparisonStrategy::Cedula => {
            let a: String = csv_val.chars().filter(|c| c.is_ascii_digit()).collect();
            let b: String = grpc_val.chars().filter(|c| c.is_ascii_digit()).collect();
            if a == b {
                FieldComparison::Match
            } else {
                FieldComparison::Mismatch {
                    expected: a,
                    actual: b,
                }
            }
        }
        ComparisonStrategy::Numeric { epsilon } => {
            match (csv_val.parse::<f64>(), grpc_val.parse::<f64>()) {
                (Ok(a), Ok(b)) => {
                    if (a - b).abs() <= *epsilon {
                        FieldComparison::Match
                    } else {
                        FieldComparison::Mismatch {
                            expected: csv_val.to_string(),
                            actual: grpc_val.to_string(),
                        }
                    }
                }
                _ => FieldComparison::Mismatch {
                    expected: csv_val.to_string(),
                    actual: grpc_val.to_string(),
                },
            }
        }
        ComparisonStrategy::Date => {
            let formats = ["%Y-%m-%d", "%Y-%m-%dT%H:%M:%SZ", "%Y-%m-%dT%H:%M:%S%.fZ"];
            let mut a_date = None;
            let mut b_date = None;

            for fmt in &formats {
                if a_date.is_none() {
                    a_date = NaiveDate::parse_from_str(csv_val.trim(), fmt).ok();
                }
                if b_date.is_none() {
                    b_date = NaiveDate::parse_from_str(grpc_val.trim(), fmt).ok();
                }
                if a_date.is_some() && b_date.is_some() {
                    break;
                }
            }

            match (a_date, b_date) {
                (Some(a), Some(b)) => {
                    if a == b {
                        FieldComparison::Match
                    } else {
                        FieldComparison::Mismatch {
                            expected: csv_val.to_string(),
                            actual: grpc_val.to_string(),
                        }
                    }
                }
                _ => FieldComparison::Mismatch {
                    expected: csv_val.to_string(),
                    actual: grpc_val.to_string(),
                },
            }
        }
    }
}
