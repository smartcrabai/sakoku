use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc;

use ignore::{WalkBuilder, WalkState};

use crate::checker::{Violation, check_bytes};
use crate::error::SakokuError;

/// The result of checking a single file - always contains at least one violation.
pub struct FileResult {
    /// Path of the checked file.
    pub path: PathBuf,
    /// Non-empty list of violations found in the file.
    pub violations: Vec<Violation>,
}

/// Walks `paths` in parallel (honouring `.gitignore`) and returns files that contain violations.
///
/// File-level I/O errors are printed to stderr and do not halt the walk.
///
/// # Errors
///
/// Returns `Err` if an error occurs before the walk begins (currently unused; reserved for
/// future path-validation logic).
pub fn walk_and_check(paths: &[PathBuf]) -> Result<Vec<FileResult>, SakokuError> {
    let (tx, rx) = mpsc::channel::<FileResult>();

    let Some((first, rest)) = paths.split_first() else {
        return Ok(Vec::new());
    };
    let mut builder = WalkBuilder::new(first);
    for path in rest {
        builder.add(path);
    }
    let walker = builder
        .hidden(true)
        .git_ignore(true)
        .add_custom_ignore_filename(".sakokuignore")
        .build_parallel();

    walker.run(|| {
        let tx = tx.clone();
        let mut buf = Vec::new();
        Box::new(move |entry_result| {
            let Ok(entry) = entry_result else {
                return WalkState::Continue;
            };
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                buf.clear();
                match File::open(entry.path()).and_then(|mut f| f.read_to_end(&mut buf)) {
                    Ok(_) => {
                        let violations = check_bytes(&buf);
                        if !violations.is_empty() {
                            let _ = tx.send(FileResult {
                                path: entry.path().to_path_buf(),
                                violations,
                            });
                        }
                    }
                    Err(e) => {
                        eprintln!("sakoku: {}: {e}", entry.path().display());
                    }
                }
            }
            WalkState::Continue
        })
    });
    drop(tx);

    Ok(rx.into_iter().collect())
}
