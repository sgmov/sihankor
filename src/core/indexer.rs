use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::database::SihDatabase;
use super::metrics::MetricEvent;
use super::models::{DocStatus, Document, Violation, ViolationSeverity};
use super::parser;
use super::validator::{ValidationConfig, validate_document};

/// 索引结果报告
#[derive(Debug, Clone, Default)]
pub struct IndexReport {
    pub discovered: usize,
    pub parsed: usize,
    pub indexed: usize,
    pub errors: Vec<(String, String)>,
    pub warnings: Vec<(String, String)>,
    /// F 级违规明细（per-document，已在 validate_document 中识别）
    /// 用于 CI 自治理的 strict 模式阻断
    pub fatal_violations: Vec<ViolationDetail>,
    /// G 级违规明细（per-document）
    /// 用于报告但默认不阻断
    pub guideline_violations: Vec<ViolationDetail>,
    /// J 级违规计数（per-document）
    /// 仅计数不阻断，不保留明细（J 语义：静默记录）
    pub judgment_count: usize,
}

/// 单条违规的精简表示（CI 报告用，不携带 fix_suggestion 以减体积）
#[derive(Debug, Clone, serde::Serialize)]
pub struct ViolationDetail {
    pub doc_id: String,
    pub rule_id: String,
    pub severity: String,
    pub location: String,
    pub message: String,
}

impl ViolationDetail {
    fn from_violation(doc_id: &str, v: &Violation) -> Self {
        Self {
            doc_id: doc_id.to_string(),
            rule_id: v.rule_id.clone(),
            severity: v.severity.as_str().to_string(),
            location: v.location.clone(),
            message: v.message.clone(),
        }
    }
}

/// 发现文档文件
pub fn discover_documents(docs_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(docs_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "target"
                && name != "archived"
                && name != "node_modules"
        })
        .flatten()
    {
        let path = entry.path();
        if path.is_file() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name.ends_with(".sih.md") {
                files.push(path.to_path_buf());
            }
        }
    }

    files
}

/// 索引结果：单篇文档 + 验证违规明细
///
/// 返回 `IndexedDoc` 而非直接 `Document`，让调用方（如 `rebuild_index`）能拿到
/// F/G/J 违规明细用于 CI 自治理阻断与报告。
#[derive(Debug, Clone)]
pub struct IndexedDoc {
    pub doc: Document,
    pub violations: Vec<Violation>,
}

/// 索引单篇文档
pub async fn index_document(
    db: &dyn SihDatabase,
    file_path: &Path,
    validation_config: &ValidationConfig,
) -> Result<IndexedDoc, IndexError> {
    // 解析
    let mut doc = parser::parse_file(file_path).map_err(|e| IndexError::ParseError {
        path: file_path.to_string_lossy().to_string(),
        error: e.to_string(),
    })?;

    // 验证
    let result = validate_document(&doc, Some(file_path), validation_config);

    // 根据验证结果设置状态
    if result.has_errors() {
        doc.status = DocStatus::Error;
    } else if result.has_warnings() {
        doc.status = DocStatus::Warning;
    }

    // 推断 nature
    let nature = crate::core::validator::infer_nature(file_path)
        .unwrap_or("unknown")
        .to_string();
    doc.nature = nature.clone();

    // 度量采集: ValidationCompleted（失败不影响主流程）
    {
        let fatal_count = result
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Fatal)
            .count();
        let guideline_count = result
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Guideline)
            .count();
        let judgment_count = result
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Judgment)
            .count();
        let event = MetricEvent::ValidationCompleted {
            doc_id: doc.id.clone(),
            nature: nature.clone(),
            stage: doc.stage.to_display(),
            fatal_count,
            guideline_count,
            judgment_count,
            passed: result.is_ok(),
        };
        if let Ok(payload) = serde_json::to_string(&event) {
            let _ = db.record_metric("ValidationCompleted", &payload).await;
        }
    }

    // 写入数据库
    db.upsert_document(doc.clone())
        .await
        .map_err(|e| IndexError::DatabaseError {
            path: file_path.to_string_lossy().to_string(),
            error: e.to_string(),
        })?;

    // 度量采集: IndexCompleted（失败不影响主流程）
    {
        let event = MetricEvent::IndexCompleted {
            doc_id: doc.id.clone(),
            nature: nature.clone(),
        };
        if let Ok(payload) = serde_json::to_string(&event) {
            let _ = db.record_metric("IndexCompleted", &payload).await;
        }
    }

    Ok(IndexedDoc {
        doc,
        violations: result.violations,
    })
}

/// 全量索引重建
///
/// 返回 `IndexReport`，包含 F/G/J 违规明细（per-document），
/// 调用方可用其构建 CI 报告并按严重度决定退出码。
pub async fn rebuild_index(
    db: &dyn SihDatabase,
    docs_dir: &Path,
    validation_config: &ValidationConfig,
) -> IndexReport {
    let mut report = IndexReport {
        discovered: 0,
        parsed: 0,
        indexed: 0,
        errors: Vec::new(),
        warnings: Vec::new(),
        fatal_violations: Vec::new(),
        guideline_violations: Vec::new(),
        judgment_count: 0,
    };

    let files = discover_documents(docs_dir);
    report.discovered = files.len();

    for file_path in &files {
        match index_document(db, file_path, validation_config).await {
            Ok(indexed) => {
                report.parsed += 1;
                report.indexed += 1;
                if indexed.doc.status == DocStatus::Warning {
                    report.warnings.push((
                        indexed.doc.id.clone(),
                        "Document has validation warnings".to_string(),
                    ));
                }

                // 收集 F/G/J 明细到 report
                for v in &indexed.violations {
                    match v.severity {
                        ViolationSeverity::Fatal => {
                            report
                                .fatal_violations
                                .push(ViolationDetail::from_violation(&indexed.doc.id, v));
                        }
                        ViolationSeverity::Guideline => {
                            report
                                .guideline_violations
                                .push(ViolationDetail::from_violation(&indexed.doc.id, v));
                        }
                        ViolationSeverity::Judgment => {
                            report.judgment_count += 1;
                        }
                    }
                }
            }
            Err(IndexError::ParseError { path, error }) => {
                report.errors.push((path, error));
            }
            Err(IndexError::DatabaseError { path, error }) => {
                report.errors.push((path, error));
            }
        }
    }

    report
}

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("Parse error for {path}: {error}")]
    ParseError { path: String, error: String },
    #[error("Database error for {path}: {error}")]
    DatabaseError { path: String, error: String },
}
