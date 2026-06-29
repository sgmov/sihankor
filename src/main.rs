use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use sihankor::core::database::SqliteBackend;
use sihankor::mcp_server::governance::SihankorService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let project_root = parse_project_root_arg()
        .unwrap_or_else(find_project_root);

    let db_path = project_root.join(".sih/index.db");
    let db = SqliteBackend::open(&db_path)?;
    let db = Arc::new(db);

    // Start dashboard server in background (syncs with MCP lifecycle)
    let db_dash = db.clone();
    let dash_handle = tokio::spawn(async move {
        if let Err(e) = sihankor::server::start(db_dash, 9741).await {
            eprintln!("siheng-dashboard: {}", e);
        }
    });
    tokio::task::yield_now().await;

    let service = SihankorService::new(db, project_root.clone());
    eprintln!("sihankor engine starting, db at {}", db_path.display());

    let io = (tokio::io::stdin(), tokio::io::stdout());
    let running = rmcp::serve_server(service, io).await?;
    eprintln!("sihankor engine ready");
    running.waiting().await?;
    // On MCP disconnect, process exits -> dashboard task is cancelled
    dash_handle.abort();
    Ok(())
}

/// 解析 `--project-root <path>` 命令行参数。
fn parse_project_root_arg() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--project-root" && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        i += 1;
    }
    None
}

/// 从 cwd 向上查找 `.sih/` 目录，返回包含它的目录作为项目根。
/// 未找到时回退到 cwd 自身。
fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join(".sih").exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    PathBuf::from(".")
}
