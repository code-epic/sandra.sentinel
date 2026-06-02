pub mod cli;
pub mod core;
pub mod error;
pub mod io;
pub mod report;

pub use cli::{parse_args, CliArgs};
pub use core::{Comparator, ComparisonResult, KeyGenerator};
pub use error::{Result, ScfError};
pub use report::ReportGenerator;
