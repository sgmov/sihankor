use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use sihankor::core::database::SqliteBackend;
use sihankor::mcp_server::governance::SihankorService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_path = find_db_path();
    let db = SqliteBackend::open(&db_path)?;

    let service = SihankorService::new(Arc::new(db));

    eprintln!("sihankor engine starting, db at {}", db_path.display());

    let io = (tokio::io::stdin(), tokio::io::stdout());
    let running = rmcp::serve_server(service, io).await?;
    eprintln!("sihankor engine ready");
    running.waiting().await?;
    Ok(())
}

fn find_db_path() -> PathBuf {
    // 从当前目录向上查找 .sih/ 目录
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        let candidate = dir.join(".sih/index.db");
        if candidate.parent().map(|p| p.exists()).unwrap_or(false) {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".sih/index.db")
}
