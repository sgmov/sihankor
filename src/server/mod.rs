// 司衡 Web 服务器 — 工厂产线看板
//
// P0: 只读看板 (GET / + GET /api/dashboard)
// P1: 按钮可操作 (POST /api/actions/stage, POST /api/actions/validate)

use axum::{Json, Router, extract::State, response::Html, routing::{get, post}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::core::database::SihDatabase;
use crate::core::kanban;
use crate::core::validator::{self, ValidationConfig};
use crate::core::models::Stage;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn SihDatabase>,
}

// ---- API types ----

#[derive(Serialize)]
struct DashboardResponse {
    columns: Vec<kanban::KanbanColumn>,
    summary: kanban::KanbanSummary,
    generated_at: String,
}

#[derive(Deserialize)]
struct ActionRequest {
    doc_id: String,
}

#[derive(Serialize)]
struct ActionResult {
    ok: bool,
    message: String,
}

// ---- Handlers ----

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

/// Re-validate a document: parse from disk, run fresh validation
async fn api_validate(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<serde_json::Value> {
    let result = match find_and_validate_on_disk(&req.doc_id).await {
        Ok(v) => serde_json::json!({"ok": true, "doc_id": req.doc_id, "violations": v}),
        Err(e) => serde_json::json!({"ok": false, "message": e}),
    };
    Json(result)
}

/// Advance document stage: 1/3 -> 2/3, or 2/3 -> 3/3
async fn api_stage(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<ActionResult> {
    let doc = match state.db.get_document(&req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => return Json(ActionResult { ok: false, message: "文档未找到".into() }),
        Err(e) => return Json(ActionResult { ok: false, message: e.to_string() }),
    };

    let next_stage = match doc.stage.0.as_str() {
        "1/3" => "2/3",
        "2/3" => "3/3",
        _ => return Json(ActionResult { ok: false, message: "无法推进：当前 stage 不支持".into() }),
    };

    let mut updated = doc.clone();
    updated.stage = Stage(next_stage.into());

    match state.db.upsert_document(updated.clone()).await {
        Ok(_) => Json(ActionResult {
            ok: true,
            message: format!("已推进 {} -> {}", req.doc_id, next_stage),
        }),
        Err(e) => Json(ActionResult { ok: false, message: e.to_string() }),
    }
}

// ---- Helpers ----

/// Parse document from actual disk file and run fresh validation
async fn find_and_validate_on_disk(doc_id: &str) -> Result<Vec<serde_json::Value>, String> {
    use crate::core::parser;

    let possible_paths = vec![
        format!("docs/specs/engineering/{}.sih.md", doc_id),
        format!("docs/specs/philosophy/{}.sih.md", doc_id),
        format!("docs/specs/techne/{}.sih.md", doc_id),
        format!("docs/proposals/{}.sih.md", doc_id),
        format!("docs/decisions/{}.sih.md", doc_id),
        format!("docs/reference/{}.sih.md", doc_id),
        format!("docs/knowledge/notes/{}.sih.md", doc_id),
    ];

    let mut file_path = None;
    for p in &possible_paths {
        if std::path::Path::new(p).exists() {
            file_path = Some(PathBuf::from(p));
            break;
        }
    }

    let path = file_path.ok_or_else(|| format!("文件未找到: {}", doc_id))?;
    let doc = parser::parse_file(&path).map_err(|e| format!("解析失败: {}", e))?;

    let result = validator::validate_document(&doc, Some(&path), &ValidationConfig::default());
    let violations: Vec<serde_json::Value> = result.violations.iter().map(|v| {
        serde_json::json!({
            "rule": v.rule_id,
            "severity": v.severity.as_str(),
            "message": v.message,
            "location": v.location,
            "fix": v.fix_suggestion,
            "dao": v.dao_trace,
        })
    }).collect();

    // Re-index after validation
    // state.db.upsert_document(doc).await ... (needs async context, skip for now)

    Ok(violations)
}

// ---- Server start ----

pub async fn start(db: Arc<dyn SihDatabase>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { db };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/dashboard", get(api_dashboard))
        .route("/api/actions/validate", post(api_validate))
        .route("/api/actions/stage", post(api_stage))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("siheng-dashboard: http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
