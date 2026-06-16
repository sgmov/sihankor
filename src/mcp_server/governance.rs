use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    ServerHandler, handler::server::wrapper::Parameters, schemars, tool, tool_handler, tool_router,
};

use crate::core::database::SihDatabase;
use crate::core::indexer;
use crate::core::models::DocStatus;
use crate::core::orchestrator::PipelineConfig;
use crate::core::parser;
use crate::core::validator::{self, ValidationConfig, ValidationResult};

/// 司衡引擎治理 MCP 服务
#[derive(Clone)]
pub struct SihankorService {
    db: Arc<dyn SihDatabase>,
    config: PipelineConfig,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ValidateRequest {
    /// 文件路径（相对于项目根目录）
    pub path: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct SearchRequest {
    /// 搜索查询字符串
    pub query: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct GetDocumentRequest {
    /// 文档 ID
    pub id: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ResolveChainRequest {
    /// 文档 ID
    pub id: String,
    /// 追溯深度（默认 10）
    #[serde(default = "default_depth")]
    pub depth: u32,
}

fn default_depth() -> u32 {
    10
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct AnalyzeDocumentRequest {
    /// 文档路径或 ID
    pub target: String,
}

#[tool_router]
impl SihankorService {
    pub fn new(db: Arc<dyn SihDatabase>) -> Self {
        Self {
            db,
            config: PipelineConfig::default(),
        }
    }

    /// 验证 .sih.md 文档合规性
    #[tool(description = "Validate a .sih.md document for compliance with SiHankor governance rules")]
    pub async fn validate_sihmd(
        &self,
        Parameters(ValidateRequest { path }): Parameters<ValidateRequest>,
    ) -> String {
        let file_path = PathBuf::from(&path);
        match parser::parse_file(&file_path) {
            Ok(doc) => {
                let result = validator::validate_document(
                    &doc,
                    Some(&file_path),
                    &ValidationConfig::default(),
                );
                format_validation_result(&doc.id, &result)
            }
            Err(e) => format!("Parse error: {}", e),
        }
    }

    /// 搜索已索引文档
    #[tool(description = "Search indexed documents by content query")]
    pub async fn search_docs(
        &self,
        Parameters(SearchRequest { query }): Parameters<SearchRequest>,
    ) -> String {
        match self.db.search_content(&query).await {
            Ok(results) => {
                if results.is_empty() {
                    "No documents found.".to_string()
                } else {
                    results
                        .iter()
                        .map(|r| {
                            format!(
                                "[{}] {} ({}) - {}",
                                r.id, r.title, r.stage.0, r.snippet
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("Search error: {}", e),
        }
    }

    /// 获取文档元数据和结构
    #[tool(description = "Get document metadata and structure by ID")]
    pub async fn get_document(
        &self,
        Parameters(GetDocumentRequest { id }): Parameters<GetDocumentRequest>,
    ) -> String {
        match self.db.get_document(&id).await {
            Ok(Some(doc)) => {
                let violations = validator::validate_document(
                    &doc,
                    None,
                    &ValidationConfig::default(),
                );
                format!(
                    "ID: {}\nStage: {}\nTitle: {}\nUpstream: {}\nStatus: {:?}\nIndexed: {}\nContent length: {} chars\nValidation: {} violations",
                    doc.id,
                    doc.stage.0,
                    doc.title,
                    doc.upstream.as_deref().unwrap_or("none"),
                    doc.status,
                    doc.indexed_at.to_rfc3339(),
                    doc.content.len(),
                    violations.violations.len(),
                )
            }
            Ok(None) => format!("Document '{}' not found.", id),
            Err(e) => format!("Database error: {}", e),
        }
    }

    /// 追溯授权链
    #[tool(description = "Trace the governance authorization chain (upstream) for a document")]
    pub async fn resolve_chain(
        &self,
        Parameters(ResolveChainRequest { id, depth }): Parameters<ResolveChainRequest>,
    ) -> String {
        match self.db.resolve_chain(&id, depth).await {
            Ok(nodes) => {
                if nodes.is_empty() {
                    format!("No chain found for document '{}'.", id)
                } else {
                    nodes
                        .iter()
                        .map(|n| {
                            format!(
                                "{}[{}] {} ({}) <- {}",
                                "  ".repeat(n.depth as usize),
                                n.depth,
                                n.title,
                                n.stage.0,
                                n.upstream.as_deref().unwrap_or("ROOT"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("Chain resolution error: {}", e),
        }
    }

    /// 项目治理概览
    #[tool(description = "Get project governance overview: document counts, stage distribution, alerts")]
    pub async fn project_status(
        &self,
        Parameters(_): Parameters<()>,
    ) -> String {
        let total = self.db.count_documents().await.unwrap_or(0);
        let by_stage = self.db.count_by_stage().await.unwrap_or_default();
        let by_nature = self.db.count_by_nature().await.unwrap_or_default();

        let stage_summary = by_stage
            .iter()
            .map(|(s, c)| format!("  {}: {}", s, c))
            .collect::<Vec<_>>()
            .join("\n");

        let nature_summary = by_nature
            .iter()
            .map(|(t, c)| format!("  {}: {}", t, c))
            .collect::<Vec<_>>()
            .join("\n");

        let error_docs = self.db.get_documents_by_status(&DocStatus::Error).await.unwrap_or_default();
        let warning_docs = self.db.get_documents_by_status(&DocStatus::Warning).await.unwrap_or_default();

        let error_list = if error_docs.is_empty() {
            "  (none)".to_string()
        } else {
            error_docs
                .iter()
                .map(|d| format!("  [{}] {} ({})", d.id, d.title, d.stage.0))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let warning_list = if warning_docs.is_empty() {
            "  (none)".to_string()
        } else {
            warning_docs
                .iter()
                .map(|d| format!("  [{}] {} ({})", d.id, d.title, d.stage.0))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            "SiHankor Project Status\n\
             ========================\n\
             Total documents: {}\n\n\
             By stage:\n{}\n\n\
             By nature:\n{}\n\n\
             Errors ({}):\n{}\n\n\
             Warnings ({}):\n{}",
            total, stage_summary, nature_summary,
            error_docs.len(), error_list,
            warning_docs.len(), warning_list
        )
    }

    /// 触发全量索引重建
    #[tool(description = "Trigger a full index rebuild: discover, parse, validate, and index all .sih.md documents")]
    pub async fn index_rebuild(
        &self,
        Parameters(_): Parameters<()>,
    ) -> String {
        let docs_dir = PathBuf::from(&self.config.docs_dir);
        let report = indexer::rebuild_index(self.db.as_ref(), &docs_dir, &self.config.validation).await;

        let errors = if report.errors.is_empty() {
            "None".to_string()
        } else {
            report
                .errors
                .iter()
                .map(|(p, e)| format!("  {}: {}", p, e))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            "Index Rebuild Report\n\
             ====================\n\
             Discovered: {}\n\
             Parsed: {}\n\
             Indexed: {}\n\
             Warnings: {}\n\
             Errors:\n{}",
            report.discovered,
            report.parsed,
            report.indexed,
            report.warnings.len(),
            errors,
        )
    }
}

fn format_validation_result(doc_id: &str, result: &ValidationResult) -> String {
    if result.violations.is_empty() {
        format!("Document '{}' passed all validation checks.", doc_id)
    } else {
        let violations = result
            .violations
            .iter()
            .map(|v| format!("[{}] {} ({}): {}", v.severity.as_str(), v.rule_id, v.location, v.message))
            .collect::<Vec<_>>()
            .join("\n");

        let status = if result.has_errors() {
            "FAIL"
        } else if result.has_warnings() {
            "PASS WITH WARNINGS"
        } else {
            "PASS"
        };

        format!(
            "Validation Result: {}\n\
             Document: {}\n\
             Violations:\n{}",
            status, doc_id, violations
        )
    }
}

#[tool_handler(instructions = "SiHankor governance engine: document validation, search, indexing, and chain resolution")]
impl ServerHandler for SihankorService {}
