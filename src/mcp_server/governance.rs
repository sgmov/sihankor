use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{ServerHandler, handler::server::wrapper::Parameters, schemars, tool, tool_router};

use crate::core::database::SihDatabase;
use crate::core::glossary::Glossary;
use crate::core::indexer;
use crate::core::models::DocStatus;
use crate::core::orchestrator::PipelineConfig;
use crate::core::parser;
use crate::core::project_state::{self, ProjectState};
use crate::core::validator::{self, ValidationConfig, ValidationResult};
use crate::mind::grilling::GrillingEngine;
use crate::mind::icl::ICL;
use crate::mind::ict::ICT;
use crate::mind::iww::IWW;

/// 司衡引擎治理 MCP 服务
#[derive(Clone)]
pub struct SihankorService {
    db: Arc<dyn SihDatabase>,
    config: PipelineConfig,
    glossary: Option<Glossary>,
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

const fn default_depth() -> u32 {
    10
}

/// Empty parameters for parameterless tools (schema: type=object, no required properties)
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
struct EmptyParams {}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct AnalyzeDocumentRequest {
    /// 文档路径或 ID
    pub target: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct GenerateDocumentPlanRequest {
    /// 目标文档类型：spec / proposal / decision / note / reference
    pub target_nature: String,
    /// 上游文档 ID（root 文档不填）
    #[serde(default)]
    pub upstream_id: Option<String>,
    /// 主题提示，用于生成文档 ID 语义短名和内容大纲
    pub topic_hint: String,
}

/// 追问引擎请求：用户提供意图，引擎生成四个元规则追问
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ProposeRequest {
    /// 用户意图的自然语言描述
    pub intent: String,
}

/// 追问回答请求：用户回答追问后，引擎生成结构化提示词
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ProposeAnswersRequest {
    /// 用户意图的自然语言描述
    pub intent: String,
    /// 用户对四个追问的回答
    pub answers: Vec<ProposeAnswer>,
}

/// 追问回答（用于 MCP 参数序列化）
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ProposeAnswer {
    pub question_id: String,
    pub content: String,
}

#[tool_router]
impl SihankorService {
    pub fn new(db: Arc<dyn SihDatabase>) -> Self {
        let glossary = Glossary::load(std::path::Path::new("glossary/zh.yml")).ok();
        Self {
            db,
            config: PipelineConfig::default(),
            glossary,
        }
    }

    /// 验证 .sih.md 文档合规性
    #[tool(
        description = "[SiHankor] Validate a .sih.md document for compliance with SiHankor governance rules"
    )]
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
    #[tool(description = "[SiHankor] Search indexed documents by content query")]
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
                        .map(|r| format!("[{}] {} ({}) - {}", r.id, r.title, r.stage.0, r.snippet))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("Search error: {}", e),
        }
    }

    /// 获取文档元数据和结构
    #[tool(description = "[SiHankor] Get document metadata and structure by ID")]
    pub async fn get_document(
        &self,
        Parameters(GetDocumentRequest { id }): Parameters<GetDocumentRequest>,
    ) -> String {
        match self.db.get_document(&id).await {
            Ok(Some(doc)) => {
                let violations =
                    validator::validate_document(&doc, None, &ValidationConfig::default());
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
    #[tool(
        description = "[SiHankor] Trace the governance authorization chain (upstream) for a document"
    )]
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
    #[tool(
        description = "[SiHankor] Get project governance overview: document counts, stage distribution, alerts"
    )]
    pub async fn project_status(&self, Parameters(_): Parameters<EmptyParams>) -> String {
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

        let error_docs = self
            .db
            .get_documents_by_status(&DocStatus::Error)
            .await
            .unwrap_or_default();
        let warning_docs = self
            .db
            .get_documents_by_status(&DocStatus::Warning)
            .await
            .unwrap_or_default();

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
            total,
            stage_summary,
            nature_summary,
            error_docs.len(),
            error_list,
            warning_docs.len(),
            warning_list
        )
    }

    /// 触发全量索引重建
    #[tool(
        description = "[SiHankor] Trigger a full index rebuild: discover, parse, validate, and index all .sih.md documents"
    )]
    pub async fn index_rebuild(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let docs_dir = PathBuf::from(&self.config.docs_dir);
        let report =
            indexer::rebuild_index(self.db.as_ref(), &docs_dir, &self.config.validation).await;

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
    #[tool(
        description = "[SiHankor] Analyze a document through iCL cognition: governance position, relation graph, divergence diagnosis"
    )]
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
                    Err(e) => {
                        return format!("Document not found: '{}'. Parse error: {}", target, e);
                    }
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
    #[tool(
        description = "[SiHankor] Generate a decision proposal from document cognition: recommended action with alternatives, rationale, and affected documents"
    )]
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
                    Err(e) => {
                        return format!("Document not found: '{}'. Parse error: {}", target, e);
                    }
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
    #[tool(
        description = "[SiHankor] Verify a decision proposal through iCT five-law check: 顺因/有度/知止/损补/顺势 → pass/fail/conditional + dao trace"
    )]
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
                    Err(e) => {
                        return format!("Document not found: '{}'. Parse error: {}", target, e);
                    }
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
    #[tool(
        description = "[SiHankor] Full three-machine flow: iCL cognition → iWW decision proposal → iCT verification. Returns complete AnalysisResult."
    )]
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
                    Err(e) => {
                        return format!("Document not found: '{}'. Parse error: {}", target, e);
                    }
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
                reason: format!(
                    "{} 个引用目标缺失，关系图谱不完整",
                    cognition.relation_graph.gaps.len()
                ),
                confidence: 0.85,
            });
        }

        let self_question = if verification.overall == crate::mind::types::Verdict::Fail {
            format!(
                "五法检验整体 Fail：决策建议被拒绝。{} 项 Fail 是否因 iCL 误诊或 criteria 过于严格？",
                verification
                    .five_law_check
                    .iter()
                    .filter(|c| c.result == crate::mind::types::LawCheckResult::Fail)
                    .count()
            )
        } else if verification.overall == crate::mind::types::Verdict::Conditional {
            format!(
                "五法检验 Conditional：决策可执行但需确认。{} 项 Conditional 是否有误报？",
                verification
                    .five_law_check
                    .iter()
                    .filter(|c| c.result == crate::mind::types::LawCheckResult::Conditional)
                    .count()
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

    /// 生成项目看板：治理链阶段列、文档和代码卡片、阻塞检测
    #[tool(
        description = "[SiHankor] Generate a kanban board: governance chain columns with document and code task cards, blocker detection"
    )]
    pub async fn generate_kanban(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let kanban = crate::core::kanban::generate_kanban(self.db.as_ref()).await;
        match serde_json::to_string_pretty(&kanban) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 生成自包含 HTML 可视化看板（浏览器直接打开）
    #[tool(
        description = "[SiHankor] Generate a self-contained HTML kanban board viewable in any browser"
    )]
    pub async fn kanban_html(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let kanban = crate::core::kanban::generate_kanban(self.db.as_ref()).await;
        crate::core::kanban::render_html(&kanban)
    }

    /// 流程推进建议：基于项目状态和文档 stage，推导下一步应推进什么
    #[tool(
        description = "[SiHankor] Suggest next action: based on project state and document stages, recommends what to advance next"
    )]
    pub async fn suggest_next_action(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let project_root = std::path::PathBuf::from(&self.config.docs_dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let state = ProjectState::load(&project_root).unwrap_or_default();
        let suggestions = project_state::suggest_next_action(&state, self.db.as_ref());
        let result = serde_json::json!({
            "current_stage": state.current_stage,
            "active_proposal": state.active_proposal,
            "pending_checkpoints": state.pending_checkpoints.iter().filter(|cp| cp.decision.is_none()).count(),
            "suggestions": suggestions,
        });
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("Serialization error: {}", e))
    }

    /// 追问引擎第一步：输入意图，返回四个元规则追问
    ///
    /// 调用此 tool 后，用户看到四个问题（道二/顺因/有度/知止）。
    /// 用户回答后再调用 sihankor_propose_answers 获取结构化提示词。
    #[tool(
        description = "[SiHankor] Grilling engine step 1: input your intent, get four Dao-driven questions to clarify the document's governance identity"
    )]
    pub async fn sihankor_propose(
        &self,
        Parameters(ProposeRequest { intent }): Parameters<ProposeRequest>,
    ) -> String {
        let engine = GrillingEngine::new(self.glossary.clone());
        let questions = engine.questions(&intent);
        serde_json::to_string_pretty(&serde_json::json!({
            "step": "questions",
            "intent": intent,
            "questions": questions,
            "instruction": "Answer these 4 questions, then call sihankor_propose_answers with your answers to get a structured prompt template."
        }))
        .unwrap_or_else(|e| format!("Serialization error: {}", e))
    }

    /// 追问引擎第二步：提交追问回答，返回结构化提示词
    ///
    /// 提示词包含 frontmatter 模板、章节结构、validator 约束注入、可证伪条件。
    /// 将该提示词发送给外部 Agent 即可生成符合治理约束的文档。
    #[tool(
        description = "[SiHankor] Grilling engine step 2: submit your answers to the four questions, get a structured prompt with frontmatter template, section outline, and constraint injection"
    )]
    pub async fn sihankor_propose_answers(
        &self,
        Parameters(ProposeAnswersRequest { intent, answers }): Parameters<ProposeAnswersRequest>,
    ) -> String {
        let engine = GrillingEngine::new(self.glossary.clone());
        let answers: Vec<crate::mind::grilling::Answer> = answers
            .into_iter()
            .map(|a| crate::mind::grilling::Answer {
                question_id: a.question_id,
                content: a.content,
            })
            .collect();
        let prompt = engine.build_prompt(&answers, &intent);
        serde_json::to_string_pretty(&prompt)
            .unwrap_or_else(|e| format!("Serialization error: {}", e))
    }

    /// 术层编排：生成文档蓝图
    ///
    /// 编排 iCL 认知 + iWW 决策，产出一份结构化的文档生成蓝图(GenerationPlan)。
    /// Agent 拿到蓝图后用 LLM 写出文档内容，再通过 validate_sihmd 和 full_analysis 校验。
    #[tool(
        description = "[SiHankor] Techne orchestration: generate a document GenerationPlan by coordinating iCL cognition and iWW decision"
    )]
    pub async fn generate_document_plan(
        &self,
        Parameters(GenerateDocumentPlanRequest {
            target_nature,
            upstream_id,
            topic_hint,
        }): Parameters<GenerateDocumentPlanRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());
        let plan =
            build_generation_plan(&icl, &target_nature, upstream_id.as_deref(), &topic_hint).await;
        match serde_json::to_string_pretty(&plan) {
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
            .map(|v| {
                format!(
                    "[{}] {} ({}): {}",
                    v.severity.as_str(),
                    v.rule_id,
                    v.location,
                    v.message
                )
            })
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

// ---------------------------------------------------------------------------
// Techne 术层：编排工具
// ---------------------------------------------------------------------------

/// 术层产出：文档生成蓝图
#[derive(Debug, Clone, serde::Serialize)]
struct GenerationPlan {
    plan_id: String,
    plan_type: String,
    context: GenerationContext,
    frontmatter_template: FrontmatterTemplate,
    sections: Vec<SectionOutline>,
    required_references: Vec<String>,
    tone_constraints: Vec<String>,
    issues_to_address: Vec<String>,
    success_criteria: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct GenerationContext {
    target_nature: String,
    target_stage: String,
    upstream: Option<UpstreamContext>,
    governance_chain: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UpstreamContext {
    id: String,
    title: String,
    nature: String,
    stage: String,
    role_in_chain: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct FrontmatterTemplate {
    id_format: String,
    stage: String,
    nature: String,
    title_hint: String,
    upstream: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SectionOutline {
    heading: String,
    points: Vec<String>,
}

/// 编排 iCL 认知 + 上下文，生成文档蓝图
async fn build_generation_plan(
    icl: &ICL,
    target_nature: &str,
    upstream_id: Option<&str>,
    topic_hint: &str,
) -> GenerationPlan {
    let now = chrono::Utc::now();
    let date_part = now.format("%y%m%d-%H%M").to_string();
    let slug = topic_hint
        .to_lowercase()
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join("-");
    let plan_id = format!("plan-{}-{}-{}", date_part, target_nature, slug);

    let mut governance_chain = Vec::new();
    let mut upstream_ctx: Option<UpstreamContext> = None;
    let mut references = Vec::new();
    let mut issues = Vec::new();

    // 分析上游文档
    if let Some(up_id) = upstream_id {
        governance_chain.push(up_id.to_string());
        if let Ok(Some(up_doc)) = icl.db().get_document(up_id).await {
            let cognition = icl.analyze(&up_doc).await;
            upstream_ctx = Some(UpstreamContext {
                id: up_doc.id.clone(),
                title: up_doc.title.clone(),
                nature: up_doc.nature.clone(),
                stage: up_doc.stage.0.clone(),
                role_in_chain: format!("{:?}", cognition.governance_position.role_in_chain)
                    .to_lowercase(),
            });
            governance_chain.extend(cognition.governance_position.upstream_chain.clone());
            references = cognition.relation_graph.references.clone();
            references.push(up_id.to_string());

            for div in &cognition.divergence_diagnosis {
                if let Some(ref suggestion) = div.suggestion {
                    issues.push(format!(
                        "[{:?}] {} → {}",
                        div.severity, div.description, suggestion
                    ));
                } else {
                    issues.push(format!("[{:?}] {}", div.severity, div.description));
                }
            }
        } else {
            issues.push(format!(
                "上游文档 '{}' 在索引中不存在，请确认 id 是否正确",
                up_id
            ));
        }
    }

    // 确定 stage
    let stage = match target_nature {
        "note" => "1/3",
        _ => "1/3",
    };

    // 构建大纲
    let sections = build_section_outline(target_nature, upstream_ctx.as_ref(), topic_hint);

    // 措辞约束
    let tone_constraints = build_tone_constraints(target_nature, stage);

    // 成功标准
    let success_criteria = build_success_criteria(target_nature);

    GenerationPlan {
        plan_id,
        plan_type: "generate".to_string(),
        context: GenerationContext {
            target_nature: target_nature.to_string(),
            target_stage: stage.to_string(),
            upstream: upstream_ctx,
            governance_chain,
        },
        frontmatter_template: FrontmatterTemplate {
            id_format: format!("{}-{}", date_part, slug),
            stage: stage.to_string(),
            nature: target_nature.to_string(),
            title_hint: topic_hint.to_string(),
            upstream: upstream_id.map(String::from),
        },
        sections,
        required_references: references,
        tone_constraints,
        issues_to_address: issues,
        success_criteria,
    }
}

fn build_section_outline(
    nature: &str,
    upstream: Option<&UpstreamContext>,
    topic_hint: &str,
) -> Vec<SectionOutline> {
    let mut sections = Vec::new();

    match nature {
        "spec" => {
            sections.push(SectionOutline {
                heading: format!("一、正名：{}是什么", topic_hint),
                points: vec![
                    "定义核心概念及其在司衡体系中的位置".into(),
                    "与相邻概念的边界".into(),
                ],
            });
            sections.push(SectionOutline {
                heading: format!("二、顺因：{}在治理链中的定位", topic_hint),
                points: vec![
                    "法源追溯：本规范的授权来源".into(),
                    "下游影响：本规范的约束范围".into(),
                ],
            });
            sections.push(SectionOutline {
                heading: format!("三、有度：{}的边界", topic_hint),
                points: vec!["纳入范围".into(), "不纳入范围".into()],
            });
        }
        "proposal" => {
            sections.push(SectionOutline {
                heading: "一、正名：提议对象".into(),
                points: vec![
                    format!("明确要变更的内容：{}", topic_hint),
                    "变更的动机和背景".into(),
                ],
            });
            sections.push(SectionOutline {
                heading: "二、顺因：治理依据".into(),
                points: vec!["上游文档的授权".into(), "变更的因果必要性".into()],
            });
            sections.push(SectionOutline {
                heading: "三、方案".into(),
                points: vec![
                    "推荐方案".into(),
                    "替代方案及取舍".into(),
                    "实施步骤".into(),
                ],
            });
        }
        "decision" => {
            sections.push(SectionOutline {
                heading: "一、背景".into(),
                points: vec!["提议摘要".into(), "审阅过程".into()],
            });
            sections.push(SectionOutline {
                heading: "二、方案选择".into(),
                points: vec!["| 维度 | 决策 | 法依据 |".into(), "每条决策一行".into()],
            });
            sections.push(SectionOutline {
                heading: "三、ADR".into(),
                points: vec!["decided-by".into(), "DEPS".into()],
            });
        }
        "reference" => {
            sections.push(SectionOutline {
                heading: format!("一、{}的定义", topic_hint),
                points: vec![
                    "中文对、英文对、词源".into(),
                    "命名理据".into(),
                    "在体系中的定位".into(),
                ],
            });
            sections.push(SectionOutline {
                heading: "二、规则".into(),
                points: vec!["本条目的规则或约定".into()],
            });
        }
        "note" => {
            sections.push(SectionOutline {
                heading: format!("一、关于{}", topic_hint),
                points: vec!["实践背景".into(), "核心发现".into()],
            });
            sections.push(SectionOutline {
                heading: "二、启示".into(),
                points: vec!["可迁移的经验".into(), "注意事项".into()],
            });
        }
        _ => {
            sections.push(SectionOutline {
                heading: format!("一、{}", topic_hint),
                points: vec!["内容待 Agent 根据上下文填充".into()],
            });
        }
    }

    // 如果有上游，添加上游相关提示
    if let Some(up) = upstream {
        sections.push(SectionOutline {
            heading: "附录：上游文档上下文".into(),
            points: vec![
                format!("上游文档：{} ({} {})", up.id, up.nature, up.stage),
                "请确保本文档的声明与上游一致".into(),
            ],
        });
    }

    sections
}

fn build_tone_constraints(nature: &str, stage: &str) -> Vec<String> {
    let mut constraints = Vec::new();

    match nature {
        "spec" => {
            if stage == "3/3" {
                constraints
                    .push("使用确定性措辞：应使用'是'、'必须'、'不可'，避免'可能'、'或许'".into());
            } else {
                constraints.push("使用规范性措辞：使用'应'、'建议'，避免'必须'".into());
            }
        }
        "proposal" => {
            constraints.push("使用开放性措辞：使用'建议'、'考虑'，避免'必须'、'不可不'".into());
            constraints.push("明确标注方案的取舍理由".into());
        }
        "decision" => {
            constraints.push("使用确定性措辞：使用'决定'、'确认'，每条决策有法依据".into());
        }
        "reference" => {
            constraints.push("使用定义性措辞：使用'是'、'指'，力求精确".into());
        }
        "note" => {
            constraints.push("使用描述性措辞：记录实践，不做规范性断言".into());
        }
        _ => {}
    }

    constraints
}

fn build_success_criteria(nature: &str) -> Vec<String> {
    let mut criteria = vec![
        "validate_sihmd 通过（无 Error 级违规）".into(),
        "full_analysis 五法检验无 Fail".into(),
    ];

    match nature {
        "proposal" => {
            criteria.push("上游 stage 在可引用范围内（2/3 或 3/3）".into());
            criteria.push("方案包含至少一个替代方案".into());
        }
        "decision" => {
            criteria.push("包含 decided-by 字段".into());
            criteria.push("每条决策有对应的法依据".into());
        }
        "spec" => {
            criteria.push("正名/顺因/有度三节完整".into());
        }
        _ => {}
    }

    criteria
}

impl ServerHandler for SihankorService {
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
        let tcc = rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        let mut result = SihankorService::tool_router().call(tcc).await?;
        result
            .content
            .insert(0, rmcp::model::Content::text("[SiHankor]"));
        Ok(result)
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        Ok(rmcp::model::ListToolsResult {
            tools: SihankorService::tool_router().list_all(),
            meta: None,
            next_cursor: None,
        })
    }

    fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        SihankorService::tool_router().get(name).cloned()
    }

    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_instructions(
            "[SiHankor] SiHankor governance engine: document validation, search, indexing, and chain resolution",
        )
    }
}
