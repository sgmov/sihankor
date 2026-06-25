// 司衡 Web 服务器 — 工厂产线看板
//
// 提供 Factorio 风格的治理装配线可视化界面。
// P0: 只读看板 (GET / + GET /api/dashboard)
// P1: 按钮可操作 + WebSocket

use axum::{Json, Router, extract::State, response::Html, routing::get};
use serde::Serialize;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::core::database::SihDatabase;
use crate::core::kanban;

// ---------------------------------------------------------------------------
// 应用状态
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn SihDatabase>,
}

// ---------------------------------------------------------------------------
// API 类型
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct DashboardResponse {
    columns: Vec<kanban::KanbanColumn>,
    summary: kanban::KanbanSummary,
    generated_at: String,
}

// ---------------------------------------------------------------------------
// 路由处理器
// ---------------------------------------------------------------------------

async fn index() -> Html<&'static str> {
    Html(include_str!("dashboard.html"))
}

async fn api_dashboard(State(state): State<AppState>) -> Json<DashboardResponse> {
    let board = kanban::generate_kanban(state.db.as_ref()).await;
    Json(DashboardResponse {
        columns: board.columns,
        summary: board.summary,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

// ---------------------------------------------------------------------------
// 服务器启动
// ---------------------------------------------------------------------------

pub async fn start(db: Arc<dyn SihDatabase>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { db };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/dashboard", get(api_dashboard))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("siheng-dashboard: http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
