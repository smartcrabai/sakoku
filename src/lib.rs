pub mod category;
pub mod checker;
pub mod cli;
pub mod error;
pub mod report;
pub mod walker;

pub use category::Category;
pub use checker::{CheckOptions, Violation, check_bytes, is_default_allowed};
pub use error::SakokuError;
pub use report::{FileSummary, format_violation, render, summarize};
pub use walker::{FileResult, walk_and_check};
