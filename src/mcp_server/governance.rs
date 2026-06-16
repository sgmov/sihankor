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
use crate::mind::icl::ICL;
use crate::mind::ict::ICT;
use crate::mind::iww::IWW;

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

    /// 文档认知分析：意图定位 + 关系照见 + 发散诊断
    #[tool(description = "Analyze a document through iCL cognition: governance position, relation graph, divergence diagnosis")]
    pub async fn analyze_document(
        &self,
        Parameters(AnalyzeDocumentRequest { target }): Parameters<AnalyzeDocumentRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());

        // 先按 id 查找，再按路径查找
        let doc = match self.db.get_document(&target).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                // 尝试按路径解析
                let file_path = PathBuf::from(&target);
                match parser::parse_file(&file_path) {
                    Ok(mut doc) => {
                        // 路径解析的文档需补 nature
                        doc.nature = validator::infer_nature(&file_path)
                            .unwrap_or("")
                            .to_string();
                        doc
                    }
                    Err(e) => return format!("Document not found: '{}'. Parse error: {}", target, e),
                }
            }
            Err(e) => return format!("Database error: {}", e),
        };

        let cognition = icl.analyze(&doc).await;

        match serde_json::to_string_pretty(&cognition) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 文档决策建议：iCL 认知 → iWW 生成决策建议
    #[tool(description = "Generate a decision proposal from document cognition: recommended action with alternatives, rationale, and affected documents")]
    pub async fn propose_decision(
        &self,
        Parameters(AnalyzeDocumentRequest { target }): Parameters<AnalyzeDocumentRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());

        // 先按 id 查找，再按路径查找
        let doc = match self.db.get_document(&target).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                let file_path = PathBuf::from(&target);
                match parser::parse_file(&file_path) {
                    Ok(mut doc) => {
                        doc.nature = validator::infer_nature(&file_path)
                            .unwrap_or("")
                            .to_string();
                        doc
                    }
                    Err(e) => return format!("Document not found: '{}'. Parse error: {}", target, e),
                }
            }
            Err(e) => return format!("Database error: {}", e),
        };

        let cognition = icl.analyze(&doc).await;
        let proposal = IWW::propose(&cognition);

        // 组装 iCL + iWW 结果
        let result = serde_json::json!({
            "cognition": cognition,
            "decision_proposal": proposal,
        });

        match serde_json::to_string_pretty(&result) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 决策验证：对已有的 decision_proposal 执行五法检验（iCT only）
    #[tool(description = "Verify a decision proposal through iCT five-law check: 顺因/有度/知止/损补/顺势 → pass/fail/conditional + dao trace")]
    pub async fn verify_decision(
        &self,
        Parameters(AnalyzeDocumentRequest { target }): Parameters<AnalyzeDocumentRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());

        let doc = match self.db.get_document(&target).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                let file_path = PathBuf::from(&target);
                match parser::parse_file(&file_path) {
                    Ok(mut doc) => {
                        doc.nature = validator::infer_nature(&file_path)
                            .unwrap_or("")
                            .to_string();
                        doc
                    }
                    Err(e) => return format!("Document not found: '{}'. Parse error: {}", target, e),
                }
            }
            Err(e) => return format!("Database error: {}", e),
        };

        let cognition = icl.analyze(&doc).await;
        let proposal = IWW::propose(&cognition);
        let verification = ICT::verify(&cognition, &proposal);

        let result = serde_json::json!({
            "cognition": cognition,
            "decision_proposal": proposal,
            "verification": verification,
        });

        match serde_json::to_string_pretty(&result) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 完整三机流转分析：iCL → iWW → iCT
    #[tool(description = "Full three-machine flow: iCL cognition → iWW decision proposal → iCT verification. Returns complete AnalysisResult.")]
    pub async fn full_analysis(
        &self,
        Parameters(AnalyzeDocumentRequest { target }): Parameters<AnalyzeDocumentRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());

        let doc = match self.db.get_document(&target).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                let file_path = PathBuf::from(&target);
                match parser::parse_file(&file_path) {
                    Ok(mut doc) => {
                        doc.nature = validator::infer_nature(&file_path)
                            .unwrap_or("")
                            .to_string();
                        doc
                    }
                    Err(e) => return format!("Document not found: '{}'. Parse error: {}", target, e),
                }
            }
            Err(e) => return format!("Database error: {}", e),
        };

        let cognition = icl.analyze(&doc).await;
        let proposal = IWW::propose(&cognition);
        let verification = ICT::verify(&cognition, &proposal);

        // 道四：从验证结果构建 limitations
        let mut limitations = Vec::new();
        let mut human_review_required = Vec::new();

        for check in &verification.five_law_check {
            if check.result == crate::mind::types::LawCheckResult::Fail {
                human_review_required.push(format!("[{}] {}", check.law, check.note));
                limitations.push(crate::mind::types::Limitation {
                    aspect: format!("{}-verification", check.law),
                    reason: format!("五法检验 Fail: {}", check.note),
                    confidence: 0.95,
                });
            } else if check.result == crate::mind::types::LawCheckResult::Conditional {
                human_review_required.push(format!("[{}] CONDITIONAL: {}", check.law, check.note));
                limitations.push(crate::mind::types::Limitation {
                    aspect: format!("{}-uncertainty", check.law),
                    reason: format!("五法检验 Conditional: {}", check.note),
                    confidence: 0.6,
                });
            }
        }

        // 补充 iCL 自身盲区
        if cognition.governance_position.upstream_chain.is_empty() {
            limitations.push(crate::mind::types::Limitation {
                aspect: "upstream-chain".into(),
                reason: "无上游链：root 文档无法追溯授权来源".into(),
                confidence: 0.9,
            });
        }
        if cognition.relation_graph.gaps.len() > 3 {
            limitations.push(crate::mind::types::Limitation {
                aspect: "reference-gaps".into(),
                reason: format!("{} 个引用目标缺失，关系图谱不完整", cognition.relation_graph.gaps.len()),
                confidence: 0.85,
            });
        }

        let self_question = if verification.overall == crate::mind::types::Verdict::Fail {
            format!(
                "五法检验整体 Fail：决策建议被拒绝。{} 项 Fail 是否因 iCL 误诊或 criteria 过于严格？",
                verification.five_law_check.iter().filter(|c| c.result == crate::mind::types::LawCheckResult::Fail).count()
            )
        } else if verification.overall == crate::mind::types::Verdict::Conditional {
            format!(
                "五法检验 Conditional：决策可执行但需确认。{} 项 Conditional 是否有误报？",
                verification.five_law_check.iter().filter(|c| c.result == crate::mind::types::LawCheckResult::Conditional).count()
            )
        } else {
            format!(
                "全 Pass：inter-document 关系可能未完全发现。当前仅检查了 {} 个引用和 {} 个重复，是否有遗漏？",
                cognition.relation_graph.references.len(),
                cognition.relation_graph.duplicates.len()
            )
        };

        let analysis_result = crate::mind::types::AnalysisResult {
            schema_version: "0.1.0".into(),
            analysis_id: format!("analysis-{}", doc.id),
            analysis_target: crate::mind::types::AnalysisTarget {
                id: doc.id.clone(),
                title: doc.title.clone(),
                nature: doc.nature.clone(),
                stage: doc.stage.0.clone(),
            },
            cognition,
            decision_proposal: Some(proposal),
            verification: Some(verification),
            limitations,
            self_question,
            human_review_required,
        };

        match serde_json::to_string_pretty(&analysis_result) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
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
