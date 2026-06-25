//! `sihankor-dashboard` — 司衡工厂产线看板 Web 服务器
//!
//! Usage: `sihankor-dashboard [port]` (default: 9741)
#![allow(clippy::print_stdout)]

use std::path::PathBuf;
use std::sync::Arc;

use sihankor::core::database::{SihDatabase, SqliteBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9741);

    let db_path = PathBuf::from(".sih/index.db");
    let db: Arc<dyn SihDatabase> = Arc::new(SqliteBackend::open(&db_path)?);

    println!("SiHankor Dashboard starting on http://localhost:{}", port);
    sihankor::server::start(db, port).await
}
