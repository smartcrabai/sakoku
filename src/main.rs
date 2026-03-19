use std::io::{self, Read};
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use sakoku::checker::check_bytes;
use sakoku::cli::Cli;
use sakoku::report::format_violation;
use sakoku::walker::walk_and_check;

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.stdin {
        let mut input = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("sakoku: error reading stdin: {e}");
            return ExitCode::from(2);
        }
        let violations = check_bytes(&input);
        for v in &violations {
            println!("{}", format_violation(Path::new("<stdin>"), v));
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

    match walk_and_check(&cli.paths) {
        Err(e) => {
            eprintln!("sakoku: {e}");
            ExitCode::from(2)
        }
        Ok(results) => {
            let mut sorted = results;
            sorted.sort_by(|a, b| a.path.cmp(&b.path));
            for result in &sorted {
                for v in &result.violations {
                    println!("{}", format_violation(&result.path, v));
                }
            }
            if sorted.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
    }
}
