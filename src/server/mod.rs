// 司衡 Web 服务器 — 工厂产线看板
//
// P0: 只读看板 (GET / + GET /api/dashboard)
// P1: 按钮可操作 (POST /api/actions/stage, POST /api/actions/validate)

use axum::{
    Json, Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::core::database::SihDatabase;
use crate::core::kanban;
use crate::core::models::Stage;
use crate::core::validator::{self, ValidationConfig};

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

async fn index() -> Html<String> {
    let html = std::fs::read_to_string("src/server/dashboard.html")
        .unwrap_or_else(|_| "<h1>dashboard.html not found</h1>".into());
    Html(html)
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
    State(_state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<serde_json::Value> {
    let result = match find_and_validate_on_disk(&req.doc_id).await {
        Ok(v) => serde_json::json!({"ok": true, "doc_id": req.doc_id, "violations": v}),
        Err(e) => serde_json::json!({"ok": false, "message": e}),
    };
    Json(result)
}

/// Run full philosophical review: iCL -> iWW -> iCT (five-law Dao check)
async fn api_assay(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<serde_json::Value> {
    let doc = match state.db.get_document(&req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => return Json(serde_json::json!({"ok": false, "message": "文档未找到"})),
        Err(e) => return Json(serde_json::json!({"ok": false, "message": e.to_string()})),
    };

    use crate::mind::icl::ICL;
    use crate::mind::ict::ICT;
    use crate::mind::iww::IWW;

    let icl = ICL::new(state.db.clone());
    let cognition = icl.analyze(&doc).await;
    let proposal = IWW::propose(&cognition);
    let verification = ICT::verify(&cognition, &proposal);

    let laws: Vec<serde_json::Value> = verification
        .five_law_check
        .iter()
        .map(|check| {
            serde_json::json!({
                "law": check.law,
                "result": format!("{:?}", check.result).to_lowercase(),
                "note": check.note,
            })
        })
        .collect();

    let dao_trace: Vec<String> = verification
        .dao_trace
        .iter()
        .map(|dt| format!("{}: {}", dt.dao, dt.trace))
        .collect();

    let mut gaps: Vec<String> = Vec::new();
    if cognition.governance_position.upstream_chain.is_empty() {
        gaps.push("无上游链：root 文档无法追溯授权来源".into());
    }
    for g in &cognition.relation_graph.gaps {
        gaps.push(format!("引用缺失: {}", g));
    }
    for d in &cognition.divergence_diagnosis {
        gaps.push(format!("[{:?}] {}", d.severity, d.description));
    }

    Json(serde_json::json!({
        "ok": true,
        "doc_id": req.doc_id,
        "overall": format!("{:?}", verification.overall).to_lowercase(),
        "laws": laws,
        "dao_trace": dao_trace,
        "gaps": gaps,
        "self_question": verification.overall != crate::mind::types::Verdict::Pass,
    }))
}

/// Advance document stage: 1/3 -> 2/3, or 2/3 -> 3/3
async fn api_stage(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<ActionResult> {
    let doc = match state.db.get_document(&req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Json(ActionResult {
                ok: false,
                message: "文档未找到".into(),
            });
        }
        Err(e) => {
            return Json(ActionResult {
                ok: false,
                message: e.to_string(),
            });
        }
    };

    let next_stage = match doc.stage.0.as_str() {
        "1/3" => "2/3",
        "2/3" => "3/3",
        _ => {
            return Json(ActionResult {
                ok: false,
                message: "无法推进：当前 stage 不支持".into(),
            });
        }
    };

    let mut updated = doc.clone();
    updated.stage = Stage(next_stage.into());

    match state.db.upsert_document(updated.clone()).await {
        Ok(_) => Json(ActionResult {
            ok: true,
            message: format!("已推进 {} -> {}", req.doc_id, next_stage),
        }),
        Err(e) => Json(ActionResult {
            ok: false,
            message: e.to_string(),
        }),
    }
}

/// Generate correction prompt when Dao check fails
async fn api_correct(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<serde_json::Value> {
    let doc = match state.db.get_document(&req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Json(serde_json::json!({"ok": false, "message": "文档未找到"}));
        }
        Err(e) => return Json(serde_json::json!({"ok": false, "message": e.to_string()})),
    };

    use crate::mind::icl::ICL;
    use crate::mind::ict::ICT;
    use crate::mind::iww::IWW;

    let icl = ICL::new(state.db.clone());
    let cognition = icl.analyze(&doc).await;
    let proposal = IWW::propose(&cognition);
    let verification = ICT::verify(&cognition, &proposal);

    let mut issues: Vec<String> = Vec::new();
    for check in &verification.five_law_check {
        if check.result != crate::mind::types::LawCheckResult::Pass {
            issues.push(format!(
                "[{}] {}: {}",
                check.law,
                check.note,
                if check.result == crate::mind::types::LawCheckResult::Fail {
                    "必须修正"
                } else {
                    "建议修正"
                }
            ));
        }
    }
    for div in &cognition.divergence_diagnosis {
        issues.push(format!("[发散] {}", div.description));
        if let Some(ref s) = div.suggestion {
            issues.push(format!("  建议: {}", s));
        }
    }

    let prompt = format!(
        "合道修正：{} ({})\n\n不通过项：\n{}\n\n修正要求：仅修正不合道部分，保持其他内容不变。",
        doc.id,
        doc.title,
        issues.join("\n"),
    );

    Json(serde_json::json!({
        "ok": true,
        "doc_id": req.doc_id,
        "correction_prompt": prompt,
    }))
}

/// Push correction to Agent — saves to .sih/corrections/ for suggest_next_action
async fn api_push_correction(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<serde_json::Value> {
    let doc = match state.db.get_document(&req.doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Json(serde_json::json!({"ok": false, "message": "文档未找到"}));
        }
        Err(e) => return Json(serde_json::json!({"ok": false, "message": e.to_string()})),
    };

    use crate::mind::grilling::GrillingEngine;
    use crate::mind::icl::ICL;
    use crate::mind::ict::ICT;
    use crate::mind::iww::IWW;

    let icl = ICL::new(state.db.clone());
    let cognition = icl.analyze(&doc).await;
    let proposal = IWW::propose(&cognition);
    let verification = ICT::verify(&cognition, &proposal);

    let mut issues: Vec<String> = Vec::new();
    for check in &verification.five_law_check {
        if check.result != crate::mind::types::LawCheckResult::Pass {
            issues.push(format!("[{}] {}", check.law, check.note));
        }
    }

    let engine = GrillingEngine::new(None);
    let answers = vec![
        crate::mind::grilling::Answer {
            question_id: "dao-er".into(),
            content: doc.nature.clone(),
        },
        crate::mind::grilling::Answer {
            question_id: "shun-yin".into(),
            content: doc.upstream.clone().unwrap_or_default(),
        },
        crate::mind::grilling::Answer {
            question_id: "you-du".into(),
            content: doc.stage.0.clone(),
        },
        crate::mind::grilling::Answer {
            question_id: "zhi-zhi".into(),
            content: "仅修正合道检查不通过的部分".into(),
        },
    ];
    let prompt = engine.build_prompt(&answers, &format!("修正 {}", doc.id));

    let corr_dir = std::path::Path::new(".sih").join("corrections");
    std::fs::create_dir_all(&corr_dir).ok();
    let task = serde_json::json!({
        "doc_id": req.doc_id,
        "doc_title": doc.title,
        "issues": issues,
        "correction_prompt": prompt,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    });
    let task_path = corr_dir.join(format!("{}.json", req.doc_id));
    std::fs::write(
        &task_path,
        serde_json::to_string_pretty(&task).unwrap_or_default(),
    )
    .ok();

    Json(serde_json::json!({
        "ok": true,
        "doc_id": req.doc_id,
        "message": format!("修正任务已推送: .sih/corrections/{}.json", req.doc_id),
        "correction_prompt": prompt,
    }))
}

/// Fix broken references — auto-remove dead DEPS entries
async fn api_fix_refs(
    State(state): State<AppState>,
    axum::extract::Json(req): axum::extract::Json<ActionRequest>,
) -> Json<serde_json::Value> {
    use crate::core::parser;

    let possible_paths = vec![
        format!("docs/specs/engineering/{}.sih.md", req.doc_id),
        format!("docs/specs/philosophy/{}.sih.md", req.doc_id),
        format!("docs/specs/techne/{}.sih.md", req.doc_id),
        format!("docs/proposals/{}.sih.md", req.doc_id),
        format!("docs/decisions/{}.sih.md", req.doc_id),
        format!("docs/reference/{}.sih.md", req.doc_id),
        format!("docs/knowledge/notes/{}.sih.md", req.doc_id),
    ];
    let path = match possible_paths
        .iter()
        .find(|p| std::path::Path::new(p).exists())
    {
        Some(p) => p.clone(),
        None => {
            // Fallback: search by ID in all .sih.md files
            let doc = state.db.get_document(&req.doc_id).await.ok().flatten();
            let found = doc.and_then(|d| {
                let dirs = [
                    "specs/engineering",
                    "specs/philosophy",
                    "specs/techne",
                    "proposals",
                    "decisions",
                    "reference",
                    "knowledge/notes",
                ];
                for dir in &dirs {
                    let full_dir = format!("docs/{}", dir);
                    if let Ok(entries) = std::fs::read_dir(&full_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.to_string_lossy().ends_with(".sih.md") {
                                match std::fs::read_to_string(&path) {
                                    Ok(content) => {
                                        let id_pattern = format!("id: {}\n", d.id);
                                        // Check with both exact and flexible matching
                                        if content.contains(&id_pattern) {
                                            return path.to_str().map(|s| s.to_string());
                                        }
                                        if content.contains(&d.id) {}
                                    }
                                    Err(e) => {}
                                }
                            }
                        }
                    } else {
                    }
                }
                None
            });
            match found {
                Some(p) => p,
                None => return Json(serde_json::json!({"ok": false, "message": "文件未找到"})),
            }
        }
    };

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let doc = match parser::parse_file(std::path::Path::new(&path)) {
        Ok(d) => d,
        Err(e) => {
            return Json(serde_json::json!({"ok": false, "message": format!("解析失败: {}", e)}));
        }
    };

    // Find DEPS section and check each reference
    let deps_marker = "\n## DEPS\n";
    let Some(deps_start) = doc.content.find(deps_marker) else {
        return Json(serde_json::json!({"ok": false, "message": "无 DEPS 章节"}));
    };
    let deps_content_start = deps_start + deps_marker.len();
    let deps_end = doc.content[deps_content_start..]
        .find("\n## ")
        .map(|i| deps_content_start + i)
        .unwrap_or(doc.content.len());
    let deps_block = &doc.content[deps_content_start..deps_end];

    let mut new_deps_lines: Vec<&str> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    for line in deps_block.lines() {
        let trimmed = line.trim();
        if let Some(ref_id) = trimmed
            .strip_prefix("- ")
            .and_then(|l| l.split(|c| c == '：' || c == ':').next())
        {
            let ref_id = ref_id.trim();
            if ref_id.len() >= 8 && ref_id.contains('-') {
                match state.db.get_document(ref_id).await {
                    Ok(Some(_)) => new_deps_lines.push(line),
                    _ => removed.push(ref_id.to_string()),
                }
                continue;
            }
        }
        new_deps_lines.push(line);
    }

    if removed.is_empty() {
        // Check if there are remaining issues after fix
        use crate::mind::icl::ICL;
        use crate::mind::ict::ICT;
        use crate::mind::iww::IWW;
        let icl = ICL::new(state.db.clone());
        let cognition = icl.analyze(&doc).await;
        let proposal = IWW::propose(&cognition);
        let verification = ICT::verify(&cognition, &proposal);
        let dao_passed = verification.overall == crate::mind::types::Verdict::Pass;
        return Json(serde_json::json!({
            "ok": true, "doc_id": req.doc_id,
            "dao_pass": dao_passed,
            "message": if dao_passed { "合道检查通过，无待修复问题。" } else { "未发现失效引用。运行合道检查查看其他问题。" },
        }));
    }

    // Rewrite file
    let before_deps = &content[..deps_content_start];
    let after_deps = &content[deps_end..];
    let new_deps_block = new_deps_lines.join("\n");
    let new_content = format!("{}{}{}", before_deps, new_deps_block, after_deps);

    // Write back and re-parse to verify
    std::fs::write(&path, &new_content).ok();
    if let Err(e) = parser::parse_file(std::path::Path::new(&path)) {
        // Rollback if parsing fails
        std::fs::write(&path, &content).ok();
        return Json(
            serde_json::json!({"ok": false, "message": format!("移除后解析失败，已回滚: {}", e)}),
        );
    }
    Json(serde_json::json!({
        "ok": true,
        "doc_id": req.doc_id,
        "message": format!("已移除 {} 个失效引用: {}", removed.len(), removed.join(", ")),
    }))
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
    let violations: Vec<serde_json::Value> = result
        .violations
        .iter()
        .map(|v| {
            serde_json::json!({
                "rule": v.rule_id,
                "severity": v.severity.as_str(),
                "message": v.message,
                "location": v.location,
                "fix": v.fix_suggestion,
            })
        })
        .collect();

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
        .route("/api/actions/assay", post(api_assay))
        .route("/api/actions/correct", post(api_correct))
        .route("/api/actions/push-correction", post(api_push_correction))
        .route("/api/actions/fix-refs", post(api_fix_refs))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("siheng-dashboard: http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
