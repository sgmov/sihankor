#![allow(clippy::print_stdout)]
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use sihankor::core::database::SqliteBackend;
use sihankor::core::orchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_path = PathBuf::from(".sih/index.db");
    let db: Arc<dyn sihankor::core::database::SihDatabase> = Arc::new(SqliteBackend::open(&db_path)?);
    let docs_dir = PathBuf::from("docs/");

    println!("Starting index rebuild...");
    let report = orchestrator::run_pipeline(&*db, &docs_dir, &Default::default()).await;

    println!("\nIndex Rebuild Report");
    println!("===================");
    println!("Discovered: {}", report.index.discovered);
    println!("Parsed: {}", report.index.parsed);
    println!("Indexed: {}", report.index.indexed);
    println!("Warnings: {}", report.index.warnings.len());
    println!("Errors: {}", report.index.errors.len());

    if !report.index.errors.is_empty() {
        println!("\nErrors:");
        for (path, error) in &report.index.errors {
            println!("  {}: {}", path, error);
        }
    }

    if !report.index.warnings.is_empty() {
        println!("\nWarnings:");
        for (id, msg) in &report.index.warnings {
            println!("  {}: {}", id, msg);
        }
    }

    Ok(())
}