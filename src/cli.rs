use std::path::PathBuf;

use clap::Parser;

/// Detect non-ASCII bytes in source files.
#[derive(Debug, Parser)]
#[command(name = "sakoku", version, about)]
pub struct Cli {
    /// Files or directories to check (required unless --stdin is used).
    pub paths: Vec<PathBuf>,

    /// Read from standard input instead of files.
    #[arg(long, conflicts_with = "paths")]
    pub stdin: bool,
}
