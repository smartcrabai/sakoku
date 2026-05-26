use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use sakoku::checker::{CheckOptions, check_bytes};
use sakoku::cli::Cli;
use sakoku::report::format_violation;
use sakoku::walker::walk_and_check;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = CheckOptions { strict: cli.strict };

    let stdout = io::stdout().lock();
    let mut out = BufWriter::new(stdout);

    if cli.stdin {
        let mut input = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("sakoku: error reading stdin: {e}");
            return ExitCode::from(2);
        }
        let violations = check_bytes(&input, options);
        for v in &violations {
            let _ = writeln!(out, "{}", format_violation(Path::new("<stdin>"), v));
        }
        return if violations.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }

    if cli.paths.is_empty() {
        eprintln!("sakoku: no paths specified. Use --help for usage.");
        return ExitCode::from(2);
    }

    match walk_and_check(&cli.paths, options) {
        Err(e) => {
            eprintln!("sakoku: {e}");
            ExitCode::from(2)
        }
        Ok(mut results) => {
            results.sort_unstable_by(|a, b| a.path.cmp(&b.path));
            for result in &results {
                for v in &result.violations {
                    let _ = writeln!(out, "{}", format_violation(&result.path, v));
                }
            }
            if results.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
    }
}
