use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use sakoku::category::Category;
use sakoku::checker::{CheckOptions, check_bytes};
use sakoku::cli::Cli;
use sakoku::report;
use sakoku::walker::{FileResult, walk_and_check};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = CheckOptions { strict: cli.strict };
    let only: Vec<Category> = cli.only.iter().copied().map(Category::from).collect();

    let results: Vec<FileResult> = if cli.stdin {
        let mut input = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut input) {
            eprintln!("sakoku: error reading stdin: {e}");
            return ExitCode::from(2);
        }
        let violations = check_bytes(&input, options);
        vec![FileResult {
            path: PathBuf::from("<stdin>"),
            violations,
        }]
    } else {
        if cli.paths.is_empty() {
            eprintln!("sakoku: no paths specified. Use --help for usage.");
            return ExitCode::from(2);
        }
        match walk_and_check(&cli.paths, options) {
            Ok(mut results) => {
                results.sort_unstable_by(|a, b| a.path.cmp(&b.path));
                results
            }
            Err(e) => {
                eprintln!("sakoku: {e}");
                return ExitCode::from(2);
            }
        }
    };

    let stdout = io::stdout().lock();
    let mut out = BufWriter::new(stdout);

    let had_violations = match report::render(&mut out, &results, cli.format, &only, cli.max_files)
    {
        Ok(had) => had,
        // A closed stdout (e.g. `sakoku . | head`) is not an error worth reporting.
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => !results.is_empty(),
        Err(e) => {
            eprintln!("sakoku: error writing output: {e}");
            return ExitCode::from(2);
        }
    };

    if let Err(e) = out.flush()
        && e.kind() != io::ErrorKind::BrokenPipe
    {
        eprintln!("sakoku: error writing output: {e}");
        return ExitCode::from(2);
    }

    if had_violations {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
