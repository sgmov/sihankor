//! `sihankor-fmt` — format lint for .sih.md documents.
//!
//! Usage: `sihankor-fmt [path]`
#![allow(clippy::print_stdout)]
//!
//! Scans .sih.md files for format violations defined in Document-Conventions §八.
//! Exits with code 1 if any Error-level violation found, 0 otherwise.

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = if args.len() > 1 {
        Path::new(&args[1]).to_path_buf()
    } else {
        Path::new("docs").to_path_buf()
    };

    if !root.exists() {
        eprintln!("sihankor-fmt: path not found: {}", root.display());
        std::process::exit(2);
    }

    let mut all_violations: Vec<sihankor::fmt::Violation> = Vec::new();
    let mut files_scanned = 0u64;

    let config = sihankor::fmt::FormatConfig::load();

    // Walk the directory tree
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !filename.ends_with(".sih.md") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("sihankor-fmt: cannot read {}: {}", path.display(), e);
                continue;
            }
        };

        let rel_path = path
            .strip_prefix(std::env::current_dir().unwrap_or_default())
            .unwrap_or(path)
            .display()
            .to_string();

        let violations = sihankor::fmt::lint_document(&rel_path, &content, &config);
        files_scanned += 1;
        all_violations.extend(violations);
    }

    if files_scanned == 0 {
        println!(
            "sihankor-fmt: no .sih.md files found under {}",
            root.display()
        );
        std::process::exit(0);
    }

    // Output violations
    let errors: Vec<_> = all_violations
        .iter()
        .filter(|v| v.level == sihankor::fmt::Level::Error)
        .collect();
    let warnings: Vec<_> = all_violations
        .iter()
        .filter(|v| v.level == sihankor::fmt::Level::Warning)
        .collect();

    for v in &all_violations {
        println!("{}", v.format());
    }

    println!(
        "sihankor-fmt: {} files scanned, {} errors, {} warnings",
        files_scanned,
        errors.len(),
        warnings.len()
    );

    // 治理追溯标记
    println!("{}", sihankor::fmt::governance_trailer(&all_violations));

    if !errors.is_empty() {
        std::process::exit(1);
    }
}
