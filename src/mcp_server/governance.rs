use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{ServerHandler, handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;

use crate::core::database::SihDatabase;
use crate::core::glossary::Glossary;
use crate::core::indexer;
use crate::core::metrics::{
    RuleAuditMetric, RuleDensityMetric, SnapshotDiff, TradeoffCoverageMetric, TrendAlignmentMetric,
    VarianceMetric, compute_latest_snapshot_diff, compute_rule_audit, compute_rule_density,
    compute_tradeoff_coverage, compute_trend_alignment, compute_variance_metric,
};
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
    project_root: PathBuf,
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
pub struct EmptyParams {}

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

/// 术层编排请求：审阅文档并规划 stage 推进
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ReviewProgressionPlanRequest {
    /// 待审文档 ID
    pub doc_id: String,
}

/// 术层编排请求：规划治理链依赖操作顺序
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct ChainPlanRequest {
    /// 目标文档 ID
    pub doc_id: String,
}

/// 术层编排请求：按概念检索并审计代码与治理文档一致性
#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct CodeAuditPlanRequest {
    /// 概念或关键词，用于在文档库中检索涉及的概念
    pub concept_hint: String,
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

/// ValidationCompleted 事件的最小反序列化结构，用于从 metrics 表中聚合 fatal_count
/// 不复用 metrics.rs 内部的同名私有类型，避免破坏模块边界
#[derive(Debug, Deserialize)]
struct ValidationCompletedFatalCount {
    fatal_count: usize,
}

/// record_trail 工具参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TrailRecord {
    pub anchor_doc: String,
    pub r#type: String,
    pub turning_point: String,
    pub rationale: String,
    pub consequences: String,
    pub agents_involved: Option<String>,
}

#[tool_router]
impl SihankorService {
    pub fn new(db: Arc<dyn SihDatabase>, project_root: PathBuf) -> Self {
        let glossary = Glossary::load(&project_root.join("glossary").join("zh.yml")).ok();
        Self {
            config: PipelineConfig::with_root(&project_root),
            db,
            glossary,
            project_root,
        }
    }

    /// 将用户传入的相对路径解析为相对于 project_root 的路径。
    /// 绝对路径原样返回。
    fn resolve_path(&self, path: &str) -> PathBuf {
        let p = PathBuf::from(path);
        if p.is_absolute() {
            p
        } else {
            self.project_root.join(&p)
        }
    }

    /// 返回底层数据库引用，供集成测试断言 metrics 写入。
    /// 生产代码不应使用此方法；MCP 工具通过 self.db 访问。
    pub fn database(&self) -> &Arc<dyn SihDatabase> {
        &self.db
    }

    /// 验证 .sih.md 文档合规性
    #[tool(
        description = "[SiHankor] Validate a .sih.md document for compliance with SiHankor governance rules"
    )]
    pub async fn validate_sihmd(
        &self,
        Parameters(ValidateRequest { path }): Parameters<ValidateRequest>,
    ) -> String {
        let file_path = self.resolve_path(&path);
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
                        .map(|r| format!("[{}] {} ({}) - {}", r.id, r.title, r.stage, r.snippet))
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
                let nature = doc.nature.clone();
                let synthetic_path = std::path::PathBuf::from(format!("docs/{}/doc.md", nature));
                let violations = validator::validate_document(
                    &doc,
                    Some(&synthetic_path),
                    &ValidationConfig::default(),
                );
                format!(
                    "ID: {}\nStage: {}\nTitle: {}\nUpstream: {}\nStatus: {:?}\nIndexed: {}\nContent length: {} chars\nValidation: {} violations",
                    doc.id,
                    doc.stage,
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
                                n.stage,
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
                .map(|d| format!("  [{}] {} ({})", d.id, d.title, d.stage))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let warning_list = if warning_docs.is_empty() {
            "  (none)".to_string()
        } else {
            warning_docs
                .iter()
                .map(|d| format!("  [{}] {} ({})", d.id, d.title, d.stage))
                .collect::<Vec<_>>()
                .join("\n")
        };

        // ProjectSnapshot 采集：记录到 metrics 表，失败不阻断概览生成
        // 聚合最近 1000 条 ValidationCompleted 记录中的 fatal_count 之和，
        // 使 compute_snapshot_diff 的 fatal_violations_delta 能反映真实 Fatal 违规数变化
        // 使用平铺 JSON 而非 MetricEvent enum 序列化结果写入 metrics 表，
        // 以与 metrics.rs ProjectSnapshotPayload 反序列化结构匹配
        let validation_records = self
            .db
            .query_metrics("ValidationCompleted", 1000)
            .await
            .unwrap_or_default();
        let fatal_violations_total: usize = validation_records
            .iter()
            .filter_map(|r| {
                serde_json::from_str::<ValidationCompletedFatalCount>(&r.payload_json).ok()
            })
            .map(|p| p.fatal_count)
            .sum();
        let snapshot = serde_json::json!({
            "total_docs": total,
            "total_rules": crate::core::validator::RULE_COUNT,
            "docs_by_stage": by_stage,
            "docs_by_nature": by_nature,
            "fatal_violations_total": fatal_violations_total,
        });
        if let Ok(payload) = serde_json::to_string(&snapshot) {
            let _ = self.db.record_metric("ProjectSnapshot", &payload).await;
        }

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

    /// 项目简报：从已有数据源聚合上下文摘要（git 状态 + 行迹 + 文档统计）
    ///
    /// 数据源：knowledge/trails/ 目录、git 命令、.sih.md 文件扫描
    /// 输出：纯文本，不超过 2000 tokens
    #[tool(
        description = "[SiHankor] Get project brief: git status, latest trails, document stats — pure text summary for agent context"
    )]
    pub async fn sihankor_project_brief(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        use crate::observe::generate_project_brief;

        let root = self.project_root.as_path();

        generate_project_brief(root)
    }

    /// 行迹上下文：读取 knowledge/trails/ 最新 N 条，以结构化文本输出
    #[tool(
        description = "[SiHankor] Get trail context: latest trail entries from knowledge/trails/ directory"
    )]
    pub async fn sihankor_trail_context(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        use crate::observe::collect_trails;

        let root = self.project_root.as_path();

        let trails = collect_trails(root, 5);
        if trails.is_empty() {
            return "No trail entries found.".to_string();
        }

        trails
            .iter()
            .map(|t| {
                format!(
                    "[{}] {} | type: {} | anchor: {} | created: {}",
                    t.trace_id, t.summary, t.trail_type, t.anchor_doc, t.created_at
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
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

    /// 记录行迹：方向性转折 / 方法选择 / 间隙发现
    #[tool(
        description = "[SiHankor] Record a trail entry: direction shift / method selection / discovery"
    )]
    pub async fn record_trail(
        &self,
        Parameters(TrailRecord {
            anchor_doc,
            r#type,
            turning_point,
            rationale,
            consequences,
            agents_involved: _,
        }): Parameters<TrailRecord>,
    ) -> String {
        use chrono::Local;
        use std::fs;
        use std::io::Write;

        let now = Local::now();
        let trail_id = now.format("%H%M%S").to_string();
        let date = now.format("%Y-%m-%d %H:%M:%S").to_string();

        // 构造 trail 文档内容（纯 Markdown，无 frontmatter）
        // 元数据 anchor_doc / type / agents_involved 不写入文件；如需追溯，从 body 内容推断
        let mut content = String::new();
        content.push_str(&format!("# trail-{}\n\n", trail_id));
        content.push_str(&format!("_{}_\n\n", date));
        content.push_str(&format!("## 转折\n\n{}\n\n", turning_point));
        content.push_str(&format!("## 理由\n\n{}\n\n", rationale));
        content.push_str(&format!("## 后果\n\n{}\n", consequences));

        // 写入 knowledge/trails/
        let trails_dir = self.project_root.join("knowledge").join("trails");

        if let Err(e) = fs::create_dir_all(&trails_dir) {
            return format!("错误：无法创建 trails 目录: {}", e);
        }

        let filename = trails_dir.join(format!("trail-{}.md", trail_id));
        match fs::File::create(&filename) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(content.as_bytes()) {
                    format!("错误：写入行迹失败: {}", e)
                } else {
                    format!(
                        "行迹已记录：{}\n\
                         anchor_doc: {}\n\
                         type: {}",
                        filename.display(),
                        anchor_doc,
                        r#type,
                    )
                }
            }
            Err(e) => format!("错误：创建行迹文件失败: {}", e),
        }
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
                let file_path = self.resolve_path(&target);
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
                let file_path = self.resolve_path(&target);
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
        description = "[SiHankor] Verify a decision proposal through iCT five-law check: 顺因/有度/知止/损补/顺势 → pass/fail/conditional + law violation summary"
    )]
    pub async fn verify_decision(
        &self,
        Parameters(AnalyzeDocumentRequest { target }): Parameters<AnalyzeDocumentRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());

        let doc = match self.db.get_document(&target).await {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                let file_path = self.resolve_path(&target);
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
                let file_path = self.resolve_path(&target);
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
                stage: doc.stage.to_display(),
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
        let state = ProjectState::load(&self.project_root).unwrap_or_default();
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

    /// 术层编排：审阅文档并规划 stage 推进
    ///
    /// 编排 iCL 认知 + iWW 决策 + iCT 验证三机全调，产出 ProgressionPlan。
    /// 包含当前 stage、目标 stage、每条发散的修复步骤、引用更新清单、
    /// stage 推进的前置条件、五法检验结果及需人工确认项。
    #[tool(
        description = "[SiHankor] Techne orchestration: review a document and produce a ProgressionPlan for stage advancement, invoking iCL + iWW + iCT in sequence"
    )]
    pub async fn review_progression_plan(
        &self,
        Parameters(ReviewProgressionPlanRequest { doc_id }): Parameters<
            ReviewProgressionPlanRequest,
        >,
    ) -> String {
        let icl = ICL::new(self.db.clone());
        let plan = build_progression_plan(&icl, &doc_id).await;
        match serde_json::to_string_pretty(&plan) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 术层编排：规划治理链依赖操作
    ///
    /// 编排 iCL 认知 + 扩展 resolve_chain，产出 ChainPlan。
    /// 遍历上游链健康状态，找下游文档（search_docs），输出操作顺序
    /// （先修上游，再推进目标，最后更新下游引用）。
    #[tool(
        description = "[SiHankor] Techne orchestration: produce a ChainPlan that traverses the upstream chain health and downstream references, ordered for safe governance operations"
    )]
    pub async fn governance_chain_plan(
        &self,
        Parameters(ChainPlanRequest { doc_id }): Parameters<ChainPlanRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());
        let plan = build_chain_plan(&icl, self.db.as_ref(), &doc_id).await;
        match serde_json::to_string_pretty(&plan) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 术层编排：代码与治理文档一致性审计
    ///
    /// search_docs 检索匹配概念 -> 对每份匹配文档跑 iCL 认知 -> 拼装 AuditPlan。
    /// 输出治理声明到代码检查点的映射、不一致清单、修复建议及优先级。
    #[tool(
        description = "[SiHankor] Techne orchestration: produce a code AuditPlan that maps governance declarations to code checkpoints and lists inconsistencies for a given concept"
    )]
    pub async fn code_audit_plan(
        &self,
        Parameters(CodeAuditPlanRequest { concept_hint }): Parameters<CodeAuditPlanRequest>,
    ) -> String {
        let icl = ICL::new(self.db.clone());
        let plan = build_audit_plan(&icl, self.db.as_ref(), &concept_hint).await;
        match serde_json::to_string_pretty(&plan) {
            Ok(json) => json,
            Err(e) => format!("Serialization error: {}", e),
        }
    }

    /// 查询产出方差指标：基于最近 ValidationCompleted 记录计算通过率、违规分布和按 nature 分组统计
    #[tool(
        description = "[SiHankor] 查询产出方差指标，包括通过率、违规数分布和按文档类型的分组统计"
    )]
    pub async fn variance_metric(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let records = match self.db.query_metrics("ValidationCompleted", 100).await {
            Ok(r) => r,
            Err(e) => return format!("Database error: {}", e),
        };
        let metric = compute_variance_metric(&records);
        format_variance_metric(&metric)
    }

    /// 查询最近两次项目快照差异，检测治理间隙增长信号
    #[tool(description = "[SiHankor] 查询最近两次项目快照的差异，检测治理间隙增长信号")]
    pub async fn snapshot_diff(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let diff = match compute_latest_snapshot_diff(&*self.db).await {
            Ok(d) => d,
            Err(e) => return format!("Database error: {}", e),
        };
        match diff {
            None => "需要至少两次 governance overview 调用才能计算快照差异".to_string(),
            Some(d) => format_snapshot_diff(&d),
        }
    }

    /// 规则审计：统计各治理域和严格度的规则分布
    #[tool(description = "[SiHankor] 规则审计：统计各治理域的规则数分布与 Fatal 级规则占比")]
    pub fn rule_audit(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let audit = compute_rule_audit();
        format_rule_audit(&audit)
    }

    /// 规则密度：统计各 nature 的治理投入密度
    #[tool(description = "[SiHankor] 规则密度：计算各文档类型的规则密度与治理投入分布")]
    pub async fn rule_density(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let nature_counts = match self.db.count_by_nature().await {
            Ok(c) => c,
            Err(e) => return format!("Database error: {}", e),
        };
        let records = match self.db.query_metrics("ValidationCompleted", 100).await {
            Ok(r) => r,
            Err(e) => return format!("Database error: {}", e),
        };
        let density = compute_rule_density(&records, &nature_counts);
        format_rule_density(&density)
    }

    /// 权衡文档覆盖率：统计 decision 文档的 ADR 三段式覆盖率
    #[tool(
        description = "[SiHankor] 权衡文档覆盖率：统计决策文档的 ADR 三段式（背景/决策/后果）记录率"
    )]
    pub async fn tradeoff_coverage(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let docs = match self.db.get_all_documents().await {
            Ok(d) => d,
            Err(e) => return format!("Database error: {}", e),
        };
        let coverage = compute_tradeoff_coverage(&docs);
        format_tradeoff_coverage(&coverage)
    }

    /// 趋势对齐：计算审查频率-变更频率比值
    #[tool(description = "[SiHankor] 趋势对齐：计算审查频率与变更频率的比值，评估治理力度适配度")]
    pub async fn trend_alignment(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let validations = match self.db.query_metrics("ValidationCompleted", 100).await {
            Ok(r) => r,
            Err(e) => return format!("Database error: {}", e),
        };
        let indexes = match self.db.query_metrics("IndexCompleted", 100).await {
            Ok(r) => r,
            Err(e) => return format!("Database error: {}", e),
        };
        let trend = compute_trend_alignment(&validations, &indexes);
        format_trend_alignment(&trend)
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

/// 格式化产出方差指标为人类可读的文本报告
fn format_variance_metric(m: &VarianceMetric) -> String {
    let window = if m.window_start.is_empty() || m.window_end.is_empty() {
        "(no records)".to_string()
    } else {
        format!("{} -> {}", m.window_start, m.window_end)
    };

    let pass_rate_by_nature = if m.pass_rate_by_nature.is_empty() {
        "  (none)".to_string()
    } else {
        m.pass_rate_by_nature
            .iter()
            .map(|(n, r)| format!("  {}: {:.2}%", n, r * 100.0))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let avg_fatal_by_nature = if m.avg_fatal_by_nature.is_empty() {
        "  (none)".to_string()
    } else {
        m.avg_fatal_by_nature
            .iter()
            .map(|(n, r)| format!("  {}: {:.2}", n, r))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "SiHankor Variance Metric\n\
         ========================\n\
         Window: {}\n\
         Total documents: {}\n\
         Pass rate: {:.2}%\n\n\
         Average Fatal violations: {:.2}\n\
         Average Guideline violations: {:.2}\n\
         Fatal violations stddev (产出方差直接度量): {:.4}\n\n\
         Pass rate by nature:\n{}\n\n\
         Average Fatal violations by nature:\n{}",
        window,
        m.total_docs,
        m.pass_rate * 100.0,
        m.avg_fatal_count,
        m.avg_guideline_count,
        m.fatal_count_stddev,
        pass_rate_by_nature,
        avg_fatal_by_nature,
    )
}

/// 格式化快照差异为人类可读的文本报告
fn format_snapshot_diff(d: &SnapshotDiff) -> String {
    let docs_by_stage_delta = if d.docs_by_stage_delta.is_empty() {
        "  (none)".to_string()
    } else {
        d.docs_by_stage_delta
            .iter()
            .map(|(s, delta)| format!("  {}: {:+}", s, delta))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let docs_by_nature_delta = if d.docs_by_nature_delta.is_empty() {
        "  (none)".to_string()
    } else {
        d.docs_by_nature_delta
            .iter()
            .map(|(n, delta)| format!("  {}: {:+}", n, delta))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "SiHankor Snapshot Diff\n\
         ======================\n\
         Previous: {}\n\
         Current: {}\n\n\
         Documents delta: {:+}\n\
         Rules delta: {:+}\n\
         Fatal violations delta: {:+}\n\n\
         Docs by stage delta:\n{}\n\n\
         Docs by nature delta:\n{}\n\n\
         Gap signals:\n  rules_grew: {}\n  docs_grew: {}",
        d.previous_time,
        d.current_time,
        d.docs_delta,
        d.rules_delta,
        d.fatal_violations_delta,
        docs_by_stage_delta,
        docs_by_nature_delta,
        d.rules_grew,
        d.docs_grew,
    )
}

fn format_rule_audit(audit: &RuleAuditMetric) -> String {
    let domain_lines = audit
        .rules_by_domain
        .iter()
        .map(|(d, c)| format!("  {}: {} 条", d, c))
        .collect::<Vec<_>>()
        .join("\n");

    let severity_lines = audit
        .rules_by_severity
        .iter()
        .map(|(s, c)| format!("  {} 级: {} 条", s, c))
        .collect::<Vec<_>>()
        .join("\n");

    let fatal_count_approx = (audit.fatal_ratio * audit.total_rules as f64) as usize;

    format!(
        "SiHankor Rule Audit\n\
         ===================\n\
         总规则数: {}\n\
         \n\
         按治理域分布:\n\
         {}\n\
         \n\
         按严格度分布:\n\
         {}\n\
         \n\
         Fatal 级规则占比: {:.1}% ({}/{})",
        audit.total_rules,
        domain_lines,
        severity_lines,
        audit.fatal_ratio * 100.0,
        fatal_count_approx,
        audit.total_rules
    )
}

fn format_rule_density(density: &RuleDensityMetric) -> String {
    let nature_lines = if density.density_by_nature.is_empty() {
        "  (无文档)".to_string()
    } else {
        density
            .density_by_nature
            .iter()
            .map(|(n, d)| {
                format!(
                    "  {}: {:.2} ({} 条规则 / 该类型文档)",
                    n, d, density.total_rules
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let variance_lines = if density.variance_by_nature.is_empty() {
        "  (无验证记录)".to_string()
    } else {
        density
            .variance_by_nature
            .iter()
            .map(|(n, v)| format!("  {}: 平均 Fatal {:.2}", n, v))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "SiHankor Rule Density (知止/G1)\n\
         ================================\n\
         总规则数: {}\n\
         总文档数: {}\n\
         整体规则密度: {:.2}\n\
         \n\
         按文档类型密度:\n\
         {}\n\
         \n\
         按文档类型产出方差（avg_fatal）:\n\
         {}\n\
         \n\
         相关性说明: {}",
        density.total_rules,
        density.total_docs,
        density.overall_density,
        nature_lines,
        variance_lines,
        density.correlation_note
    )
}

fn format_tradeoff_coverage(coverage: &TradeoffCoverageMetric) -> String {
    format!(
        "SiHankor Tradeoff Coverage (损补/G4)\n\
         ====================================\n\
         Decision 文档总数: {}\n\
         ADR 三段式覆盖数: {}\n\
         ADR 覆盖率: {:.1}%\n\
         \n\
         规则增删说明: {}",
        coverage.total_decisions,
        coverage.adr_covered,
        coverage.adr_coverage_rate * 100.0,
        coverage.rule_changes_note
    )
}

fn format_trend_alignment(trend: &TrendAlignmentMetric) -> String {
    format!(
        "SiHankor Trend Alignment (顺势/G5)\n\
         ===================================\n\
         窗口: {} -> {}\n\
         审查次数（ValidationCompleted）: {}\n\
         变更次数（IndexCompleted）: {}\n\
         审查/变更比值: {:.2}\n\
         \n\
         解释说明: {}",
        trend.window_start,
        trend.window_end,
        trend.validation_count,
        trend.index_count,
        trend.review_change_ratio,
        trend.interpretation_note
    )
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

/// 术层产出：审阅与 stage 推进蓝图
#[derive(Debug, Clone, serde::Serialize)]
struct ProgressionPlan {
    plan_id: String,
    plan_type: String,
    target_doc: TargetDocSummary,
    current_stage: String,
    target_stage: String,
    divergence_fixes: Vec<DivergenceFix>,
    reference_updates: Vec<String>,
    prerequisites: Vec<String>,
    five_law_check: Vec<LawCheckEntry>,
    overall_verdict: String,
    human_review_items: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TargetDocSummary {
    id: String,
    title: String,
    nature: String,
    stage: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DivergenceFix {
    severity: String,
    div_type: String,
    description: String,
    fix_step: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct LawCheckEntry {
    law: String,
    result: String,
    note: String,
}

/// 术层产出：治理链操作顺序蓝图
#[derive(Debug, Clone, serde::Serialize)]
struct ChainPlan {
    plan_id: String,
    plan_type: String,
    target_doc_id: String,
    upstream_health: Vec<UpstreamHealth>,
    downstream_impact: Vec<DownstreamRef>,
    operation_order: Vec<ChainOperation>,
    success_criteria: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct UpstreamHealth {
    doc_id: String,
    stage: String,
    status: String,
    note: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct DownstreamRef {
    doc_id: String,
    title: String,
    nature: String,
    stage: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ChainOperation {
    order: u32,
    phase: String,
    action: String,
    affected_docs: Vec<String>,
    revert_step: String,
}

/// 术层产出：代码与治理文档一致性审计蓝图
#[derive(Debug, Clone, serde::Serialize)]
struct AuditPlan {
    plan_id: String,
    plan_type: String,
    concept: String,
    involved_docs: Vec<TargetDocSummary>,
    checkpoints: Vec<AuditCheckpoint>,
    inconsistencies: Vec<AuditInconsistency>,
    fix_recommendations: Vec<AuditFix>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AuditCheckpoint {
    doc_id: String,
    declaration: String,
    code_checkpoint: String,
    status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AuditInconsistency {
    doc_id: String,
    severity: String,
    description: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct AuditFix {
    priority: String,
    doc_id: String,
    recommendation: String,
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
                stage: up_doc.stage.to_display(),
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

/// 计算 stage 推进的目标 stage。
///
/// 正常推进：Propose(1/3) -> Resolve(2/3) -> Ratify(3/3)。
/// Deprecated(X) 和 Superseded(0/) 为终态，无下一阶段。
/// 已达 Ratify(3/3) 返回 None，由调用方决定 Reopen 或保持。
fn next_stage(stage: &crate::core::models::Stage) -> Option<String> {
    use crate::core::models::Stage;
    match stage {
        Stage::Propose => Some("2/3".to_string()),
        Stage::Resolve => Some("3/3".to_string()),
        Stage::Ratify => None,
        Stage::Deprecated | Stage::Superseded(_) => None,
    }
}

/// 将 iCT 五法检验结果映射为可序列化的快照（避免直接序列化依赖 iCT 内部枚举）。
fn map_law_check(checks: &[crate::mind::types::LawCheck]) -> Vec<LawCheckEntry> {
    checks
        .iter()
        .map(|c| LawCheckEntry {
            law: c.law.clone(),
            result: format!("{:?}", c.result).to_lowercase(),
            note: c.note.clone(),
        })
        .collect()
}

/// 编排 iCL 认知 + iWW 决策 + iCT 验证，生成 stage 推进蓝图。
async fn build_progression_plan(icl: &ICL, doc_id: &str) -> ProgressionPlan {
    let now = chrono::Utc::now();
    let date_part = now.format("%y%m%d-%H%M").to_string();
    let plan_id = format!("plan-progression-{}-{}", date_part, doc_id);

    let target_stage_disp;
    let mut target_doc = TargetDocSummary {
        id: String::new(),
        title: String::new(),
        nature: String::new(),
        stage: String::new(),
    };
    let mut divergence_fixes: Vec<DivergenceFix> = Vec::new();
    let mut reference_updates: Vec<String> = Vec::new();
    let mut prerequisites: Vec<String> = Vec::new();
    let mut five_law: Vec<LawCheckEntry> = Vec::new();
    let mut verdict = "pass".to_string();
    let mut human_review: Vec<String> = Vec::new();

    match icl.db().get_document(doc_id).await {
        Ok(Some(doc)) => {
            target_doc = TargetDocSummary {
                id: doc.id.clone(),
                title: doc.title.clone(),
                nature: doc.nature.clone(),
                stage: doc.stage.to_display(),
            };
            target_stage_disp = next_stage(&doc.stage).unwrap_or_else(|| "3/3".to_string());

            let cognition = icl.analyze(&doc).await;
            let proposal = IWW::propose(&cognition);
            let verification = ICT::verify(&cognition, &proposal);

            for div in &cognition.divergence_diagnosis {
                let fix = div
                    .suggestion
                    .clone()
                    .unwrap_or_else(|| "需人工决定".to_string());
                divergence_fixes.push(DivergenceFix {
                    severity: format!("{:?}", div.severity).to_lowercase(),
                    div_type: format!("{:?}", div.div_type).to_lowercase(),
                    description: div.description.clone(),
                    fix_step: fix,
                });
            }

            reference_updates = cognition.relation_graph.references.clone();
            if !cognition.relation_graph.gaps.is_empty() {
                prerequisites.push(format!("补齐缺失引用：{:?}", cognition.relation_graph.gaps));
            }

            if let Some(ref up) = doc.upstream {
                prerequisites.push(format!(
                    "确认上游 '{}' stage 在可引用范围（2/3 或 3/3）",
                    up
                ));
            }
            if target_stage_disp == "2/3" {
                prerequisites.push("完成发散修复并通过五法检验".into());
            }
            if target_stage_disp == "3/3" {
                prerequisites.push("resolved 阶段文档已沉淀证据，五法检验需无 Fail".into());
            }

            five_law = map_law_check(&verification.five_law_check);
            verdict = format!("{:?}", verification.overall).to_lowercase();
            for check in &verification.five_law_check {
                if check.result != crate::mind::types::LawCheckResult::Pass {
                    human_review.push(format!(
                        "[{}] {:?}: {}",
                        check.law, check.result, check.note
                    ));
                }
            }
            if verification.law_violation_summary.is_empty() {
                human_review.push("最终推进前需 Agent 复核 verification.overall".into());
            }
        }
        Ok(None) => {
            target_stage_disp = "1/3".to_string();
            prerequisites.push(format!("文档 '{}' 未索引，需先 index_rebuild", doc_id));
            human_review.push(format!("文档 '{}' 不存在，无法推进 stage", doc_id));
        }
        Err(e) => {
            target_stage_disp = "1/3".to_string();
            prerequisites.push(format!("数据库错误：{}", e));
            human_review.push("数据库不可达，请稍后重试".into());
        }
    }

    ProgressionPlan {
        plan_id,
        plan_type: "review".to_string(),
        target_doc: target_doc.clone(),
        current_stage: target_doc.stage,
        target_stage: target_stage_disp,
        divergence_fixes,
        reference_updates,
        prerequisites,
        five_law_check: five_law,
        overall_verdict: verdict,
        human_review_items: human_review,
    }
}

/// 编排 iCL 认知 + resolve_chain 遍历，生成治理链操作顺序蓝图。
async fn build_chain_plan(
    icl: &ICL,
    db: &dyn crate::core::database::SihDatabase,
    doc_id: &str,
) -> ChainPlan {
    let now = chrono::Utc::now();
    let date_part = now.format("%y%m%d-%H%M").to_string();
    let plan_id = format!("plan-chain-{}-{}", date_part, doc_id);

    let mut upstream_health: Vec<UpstreamHealth> = Vec::new();
    let mut downstream_impact: Vec<DownstreamRef> = Vec::new();
    let mut operation_order: Vec<ChainOperation> = Vec::new();

    match db.resolve_chain(doc_id, 20).await {
        Ok(nodes) => {
            for node in &nodes {
                if node.id == *doc_id {
                    continue;
                }
                let status = if node.stage.is_referenceable() {
                    "ratified"
                } else {
                    "not_referenceable"
                };
                let note = match status {
                    "ratified" => "上游已 ratify，可作为合法授权源".to_string(),
                    _ => format!("上游 stage {} 不可直接引用，需先推进", node.stage),
                };
                upstream_health.push(UpstreamHealth {
                    doc_id: node.id.clone(),
                    stage: node.stage.to_display(),
                    status: status.to_string(),
                    note,
                });
            }
        }
        Err(e) => {
            upstream_health.push(UpstreamHealth {
                doc_id: "chain".to_string(),
                stage: "?".to_string(),
                status: "error".to_string(),
                note: format!("resolve_chain 失败：{}", e),
            });
        }
    }

    let unhealthy: Vec<String> = upstream_health
        .iter()
        .filter(|u| u.status == "not_referenceable")
        .map(|u| u.doc_id.clone())
        .collect();

    match db.search_content(doc_id).await {
        Ok(results) => {
            for r in &results {
                if r.id == *doc_id {
                    continue;
                }
                downstream_impact.push(DownstreamRef {
                    doc_id: r.id.clone(),
                    title: r.title.clone(),
                    nature: "unknown".to_string(),
                    stage: r.stage.to_display(),
                });
            }
        }
        Err(e) => {
            operation_order.push(ChainOperation {
                order: 0,
                phase: "diagnose".to_string(),
                action: format!("下游文档搜索失败：{}", e),
                affected_docs: vec![],
                revert_step: "无需回退".to_string(),
            });
        }
    }

    let mut order = 1u32;
    for up_id in &unhealthy {
        operation_order.push(ChainOperation {
            order,
            phase: "fix_upstream".to_string(),
            action: format!("先推进上游 '{}' 到 2/3 或 3/3", up_id),
            affected_docs: vec![up_id.clone()],
            revert_step: "git revert 上游 stage 推进".to_string(),
        });
        order += 1;
    }

    operation_order.push(ChainOperation {
        order,
        phase: "advance_target".to_string(),
        action: format!("推进目标文档 '{}' 的 stage", doc_id),
        affected_docs: vec![doc_id.to_string()],
        revert_step: "git revert 目标 stage 变更".to_string(),
    });
    order += 1;

    if !downstream_impact.is_empty() {
        let downstream_ids: Vec<String> =
            downstream_impact.iter().map(|d| d.doc_id.clone()).collect();
        operation_order.push(ChainOperation {
            order,
            phase: "update_downstream".to_string(),
            action: "通知下游文档更新 upstream 引用".to_string(),
            affected_docs: downstream_ids,
            revert_step: "git revert 下游引用变更".to_string(),
        });
    }

    let success_criteria = vec![
        "上游链全部 ratified 或 resolved".to_string(),
        "目标 stage 通过 validate_sihmd 和 full_analysis".to_string(),
        "下游文档引用更新完毕".to_string(),
    ];

    let _ = icl;
    ChainPlan {
        plan_id,
        plan_type: "chain".to_string(),
        target_doc_id: doc_id.to_string(),
        upstream_health,
        downstream_impact,
        operation_order,
        success_criteria,
    }
}

/// 编排 search_docs + 对每份文档跑 iCL，生成代码与治理文档一致性审计蓝图。
async fn build_audit_plan(
    icl: &ICL,
    db: &dyn crate::core::database::SihDatabase,
    concept_hint: &str,
) -> AuditPlan {
    let now = chrono::Utc::now();
    let date_part = now.format("%y%m%d-%H%M").to_string();
    let slug = concept_hint
        .to_lowercase()
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join("-");
    let plan_id = format!("plan-audit-{}-{}", date_part, slug);

    let mut involved: Vec<TargetDocSummary> = Vec::new();
    let mut checkpoints: Vec<AuditCheckpoint> = Vec::new();
    let mut inconsistencies: Vec<AuditInconsistency> = Vec::new();
    let mut fixes: Vec<AuditFix> = Vec::new();

    let search_results = match db.search_content(concept_hint).await {
        Ok(r) => r,
        Err(e) => {
            involved.push(TargetDocSummary {
                id: "search-error".to_string(),
                title: format!("search_content failed: {}", e),
                nature: "diagnostic".to_string(),
                stage: "?".to_string(),
            });
            return AuditPlan {
                plan_id,
                plan_type: "audit".to_string(),
                concept: concept_hint.to_string(),
                involved_docs: involved,
                checkpoints,
                inconsistencies,
                fix_recommendations: fixes,
            };
        }
    };

    for r in &search_results {
        let doc_opt = match db.get_document(&r.id).await {
            Ok(o) => o,
            Err(_) => None,
        };
        let Some(doc) = doc_opt else { continue };
        involved.push(TargetDocSummary {
            id: doc.id.clone(),
            title: doc.title.clone(),
            nature: doc.nature.clone(),
            stage: doc.stage.to_display(),
        });

        let declaration = format!("# {} ({})", doc.title, doc.stage);
        let code_checkpoint = infer_code_checkpoint(&doc.nature);
        let status = if doc.nature.is_empty() || doc.nature == "unknown" {
            "unmapped"
        } else {
            "mapped"
        };
        checkpoints.push(AuditCheckpoint {
            doc_id: doc.id.clone(),
            declaration,
            code_checkpoint: code_checkpoint.clone(),
            status: status.to_string(),
        });

        let cognition = icl.analyze(&doc).await;
        for div in &cognition.divergence_diagnosis {
            inconsistencies.push(AuditInconsistency {
                doc_id: doc.id.clone(),
                severity: format!("{:?}", div.severity).to_lowercase(),
                description: div.description.clone(),
            });
            fixes.push(AuditFix {
                priority: format!("{:?}", div.severity).to_lowercase(),
                doc_id: doc.id.clone(),
                recommendation: div
                    .suggestion
                    .clone()
                    .unwrap_or_else(|| "需人工审查".to_string()),
            });
        }

        for gap in &cognition.relation_graph.gaps {
            inconsistencies.push(AuditInconsistency {
                doc_id: doc.id.clone(),
                severity: "warning".to_string(),
                description: format!("引用 '{}' 缺失，可能影响代码 checkpoint 一致性", gap),
            });
            fixes.push(AuditFix {
                priority: "warning".to_string(),
                doc_id: doc.id.clone(),
                recommendation: format!("补齐文档 '{}' 或更正引用", gap),
            });
        }
    }

    fixes.sort_by(|a, b| priority_rank(&b.priority).cmp(&priority_rank(&a.priority)));

    AuditPlan {
        plan_id,
        plan_type: "audit".to_string(),
        concept: concept_hint.to_string(),
        involved_docs: involved,
        checkpoints,
        inconsistencies,
        fix_recommendations: fixes,
    }
}

/// 根据文档 nature 推断代码检查点（治理声明到代码实现的映射占位）。
fn infer_code_checkpoint(nature: &str) -> String {
    match nature {
        "spec" => "src/core/ 下的 validator / models 字段定义".to_string(),
        "decision" => "src/mcp_server/ 下的工具行为 / src/core/database.rs trait 实现".to_string(),
        "proposal" => "尚未映射代码（proposal 阶段）".to_string(),
        "reference" => "src/ 与 glossary/ 字段命名一致性".to_string(),
        "note" => "无代码检查点（note 是经验记录）".to_string(),
        _ => format!("未知 nature '{}'，无法映射代码检查点", nature),
    }
}

/// 优先级排序权重，Critical > Warning > Info。
fn priority_rank(p: &str) -> u32 {
    match p {
        "critical" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::SqliteBackend;
    use crate::core::models::{DocStatus, Document, Frontmatter, Stage};
    use std::sync::Arc;

    fn make_service() -> SihankorService {
        let db = SqliteBackend::open_in_memory().unwrap();
        SihankorService::new(Arc::new(db), PathBuf::from("."))
    }

    fn make_test_doc(id: &str, nature: &str, stage: &str) -> Document {
        Document {
            id: id.to_string(),
            stage: Stage::from_str(stage).unwrap(),
            title: "Test Doc".to_string(),
            upstream: None,
            frontmatter: Frontmatter {
                id: id.to_string(),
                stage: Stage::from_str(stage).unwrap(),
                upstream: None,
                decided_by: None,
                extra: serde_json::Value::Null,
            },
            content: "test content".to_string(),
            status: DocStatus::Ok,
            indexed_at: chrono::Utc::now(),
            nature: nature.to_string(),
        }
    }

    #[tokio::test]
    async fn test_variance_metric_empty() {
        let svc = make_service();
        let result = svc.variance_metric(Parameters(EmptyParams {})).await;
        assert!(
            result.contains("Total documents: 0"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("产出方差直接度量"),
            "result was: {}",
            result
        );
        assert!(result.contains("(no records)"), "result was: {}", result);
    }

    #[tokio::test]
    async fn test_variance_metric_with_records() {
        let svc = make_service();
        let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":2,"judgment_count":1,"passed":true}"#;
        let p2 = r#"{"doc_id":"d2","nature":"proposal","stage":"1/3","fatal_count":1,"guideline_count":0,"judgment_count":1,"passed":false}"#;
        svc.db
            .record_metric("ValidationCompleted", p1)
            .await
            .unwrap();
        svc.db
            .record_metric("ValidationCompleted", p2)
            .await
            .unwrap();
        let result = svc.variance_metric(Parameters(EmptyParams {})).await;
        assert!(
            result.contains("Total documents: 2"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("Pass rate: 50.00%"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("Average Fatal violations: 0.50"),
            "result was: {}",
            result
        );
        // 按 nature 分组：spec 通过率 100%，proposal 通过率 0%
        assert!(result.contains("spec: 100.00%"), "result was: {}", result);
        assert!(result.contains("proposal: 0.00%"), "result was: {}", result);
    }

    #[tokio::test]
    async fn test_snapshot_diff_insufficient() {
        let svc = make_service();
        // 空数据库：不足两条快照
        let result = svc.snapshot_diff(Parameters(EmptyParams {})).await;
        assert!(result.contains("需要至少两次"), "result was: {}", result);

        // 仅一条快照：仍不足
        let s1 = r#"{"total_docs":10,"total_rules":14,"docs_by_stage":[["1/3",5]],"docs_by_nature":[["spec",6]],"fatal_violations_total":2}"#;
        svc.db.record_metric("ProjectSnapshot", s1).await.unwrap();
        let result = svc.snapshot_diff(Parameters(EmptyParams {})).await;
        assert!(result.contains("需要至少两次"), "result was: {}", result);
    }

    #[tokio::test]
    async fn test_snapshot_diff_with_two_snapshots() {
        let svc = make_service();
        let s1 = r#"{"total_docs":10,"total_rules":14,"docs_by_stage":[["1/3",5],["2/3",3],["3/3",2]],"docs_by_nature":[["spec",6],["decision",4]],"fatal_violations_total":2}"#;
        let s2 = r#"{"total_docs":12,"total_rules":14,"docs_by_stage":[["1/3",6],["2/3",4],["3/3",2]],"docs_by_nature":[["spec",7],["decision",5]],"fatal_violations_total":1}"#;
        svc.db.record_metric("ProjectSnapshot", s1).await.unwrap();
        svc.db.record_metric("ProjectSnapshot", s2).await.unwrap();
        let result = svc.snapshot_diff(Parameters(EmptyParams {})).await;
        // 不应返回提示文本，应返回格式化报告
        assert!(!result.contains("需要至少两次"), "result was: {}", result);
        assert!(
            result.contains("SiHankor Snapshot Diff"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("Documents delta:"),
            "result was: {}",
            result
        );
        assert!(result.contains("Gap signals:"), "result was: {}", result);
    }

    #[test]
    fn test_format_snapshot_diff_known_values() {
        let d = SnapshotDiff {
            previous_time: "2024-01-01T00:00:00Z".to_string(),
            current_time: "2024-01-02T00:00:00Z".to_string(),
            docs_delta: 2,
            rules_delta: 0,
            docs_by_stage_delta: vec![
                ("1/3".to_string(), 1),
                ("2/3".to_string(), 1),
                ("3/3".to_string(), 0),
            ],
            docs_by_nature_delta: vec![("decision".to_string(), 2), ("spec".to_string(), 0)],
            fatal_violations_delta: -1,
            rules_grew: false,
            docs_grew: true,
        };
        let result = format_snapshot_diff(&d);
        assert!(
            result.contains("Documents delta: +2"),
            "result was: {}",
            result
        );
        assert!(result.contains("Rules delta: +0"), "result was: {}", result);
        assert!(
            result.contains("Fatal violations delta: -1"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("rules_grew: false"),
            "result was: {}",
            result
        );
        assert!(result.contains("docs_grew: true"), "result was: {}", result);
        assert!(result.contains("1/3: +1"), "result was: {}", result);
    }

    #[test]
    fn test_format_variance_metric_known_values() {
        let m = VarianceMetric {
            total_docs: 3,
            pass_rate: 2.0 / 3.0,
            avg_fatal_count: 1.0 / 3.0,
            avg_guideline_count: 1.0,
            fatal_count_stddev: (2.0_f64 / 9.0_f64).sqrt(),
            pass_rate_by_nature: vec![("proposal".to_string(), 0.0), ("spec".to_string(), 1.0)],
            avg_fatal_by_nature: vec![("proposal".to_string(), 1.0), ("spec".to_string(), 0.0)],
            window_start: "2024-01-01T00:00:00Z".to_string(),
            window_end: "2024-01-03T00:00:00Z".to_string(),
        };
        let result = format_variance_metric(&m);
        assert!(
            result.contains("Total documents: 3"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("Pass rate: 66.67%"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("Average Fatal violations: 0.33"),
            "result was: {}",
            result
        );
        assert!(
            result.contains("产出方差直接度量"),
            "result was: {}",
            result
        );
        assert!(result.contains("spec: 100.00%"), "result was: {}", result);
        assert!(result.contains("proposal: 0.00%"), "result was: {}", result);
    }

    /// Verify rule_audit tool returns valid data
    #[test]
    fn test_rule_audit_tool() {
        let svc = make_service();
        let result = svc.rule_audit(Parameters(EmptyParams {}));
        assert!(result.contains("SiHankor Rule Audit"));
        assert!(result.contains("总规则数"));
        assert!(result.contains("Frontmatter"));
        assert!(result.contains("Structure"));
        assert!(result.contains("Fatal 级规则占比"));
    }

    /// Verify rule_density tool returns valid data (empty db)
    #[tokio::test]
    async fn test_rule_density_tool_empty() {
        let svc = make_service();
        let result = svc.rule_density(Parameters(EmptyParams {})).await;
        assert!(result.contains("SiHankor Rule Density"));
        assert!(result.contains("样本不足"));
    }

    /// Verify rule_density tool with seeded data
    #[tokio::test]
    async fn test_rule_density_tool_with_data() {
        let svc = make_service();

        let vc = serde_json::json!({
            "doc_id": "d1", "nature": "spec", "stage": "1/3",
            "fatal_count": 0, "guideline_count": 2,
            "judgment_count": 1, "passed": true
        });
        svc.db
            .record_metric("ValidationCompleted", &vc.to_string())
            .await
            .unwrap();
        svc.db
            .upsert_document(make_test_doc("d1", "spec", "1/3"))
            .await
            .unwrap();

        let result = svc.rule_density(Parameters(EmptyParams {})).await;
        assert!(result.contains("SiHankor Rule Density"));
        assert!(result.contains("总规则数"));
    }

    /// Verify tradeoff_coverage tool returns valid data (empty db)
    #[tokio::test]
    async fn test_tradeoff_coverage_tool_empty() {
        let svc = make_service();
        let result = svc.tradeoff_coverage(Parameters(EmptyParams {})).await;
        assert!(result.contains("SiHankor Tradeoff Coverage"));
        assert!(result.contains("无 decision 文档"));
    }

    /// Verify trend_alignment tool returns valid data (empty db)
    #[tokio::test]
    async fn test_trend_alignment_tool_empty() {
        let svc = make_service();
        let result = svc.trend_alignment(Parameters(EmptyParams {})).await;
        assert!(result.contains("SiHankor Trend Alignment"));
        assert!(result.contains("仅覆盖时势维度"));
    }

    /// Verify trend_alignment tool with seeded data
    #[tokio::test]
    async fn test_trend_alignment_tool_with_data() {
        let svc = make_service();

        let vc = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":0,"judgment_count":0,"passed":true}"#;
        let ic = r#"{"doc_id":"d1","nature":"spec"}"#;
        svc.db
            .record_metric("ValidationCompleted", vc)
            .await
            .unwrap();
        svc.db.record_metric("IndexCompleted", ic).await.unwrap();

        let result = svc.trend_alignment(Parameters(EmptyParams {})).await;
        assert!(result.contains("SiHankor Trend Alignment"));
        assert!(result.contains("审查次数"));
        assert!(result.contains("变更次数"));
    }

    fn make_test_doc_with_upstream(
        id: &str,
        nature: &str,
        stage: &str,
        upstream: Option<&str>,
    ) -> Document {
        let stage_enum = Stage::from_str(stage).unwrap();
        Document {
            id: id.to_string(),
            stage: stage_enum.clone(),
            title: format!("Test Doc {}", id),
            upstream: upstream.map(String::from),
            frontmatter: Frontmatter {
                id: id.to_string(),
                stage: stage_enum,
                upstream: upstream.map(String::from),
                decided_by: None,
                extra: serde_json::Value::Null,
            },
            content: format!("content for {} matching concept techne", id),
            status: DocStatus::Ok,
            indexed_at: chrono::Utc::now(),
            nature: nature.to_string(),
        }
    }

    /// 验证 review_progression_plan 对已索引文档返回目标 stage = 2/3
    #[tokio::test]
    async fn test_review_progression_plan_existing_doc() {
        let svc = make_service();
        svc.db
            .upsert_document(make_test_doc_with_upstream(
                "260616-1615-techne-spec",
                "spec",
                "1/3",
                Some("240610-1030-on-sihankor-canon"),
            ))
            .await
            .unwrap();

        let req = ReviewProgressionPlanRequest {
            doc_id: "260616-1615-techne-spec".to_string(),
        };
        let result = svc.review_progression_plan(Parameters(req)).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["plan_type"], "review");
        assert_eq!(parsed["current_stage"], "1/3");
        assert_eq!(parsed["target_stage"], "2/3");
        assert_eq!(parsed["target_doc"]["id"], "260616-1615-techne-spec");
        let five_law = parsed["five_law_check"].as_array().unwrap();
        assert_eq!(five_law.len(), 5);
    }

    /// 验证 review_progression_plan 对缺失文档返回明确错误前提
    #[tokio::test]
    async fn test_review_progression_plan_missing_doc() {
        let svc = make_service();
        let req = ReviewProgressionPlanRequest {
            doc_id: "999999-0000-missing".to_string(),
        };
        let result = svc.review_progression_plan(Parameters(req)).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["current_stage"], "");
        assert_eq!(parsed["target_stage"], "1/3");
        let prereqs = parsed["prerequisites"].as_array().unwrap();
        assert!(
            prereqs
                .iter()
                .any(|p| p.as_str().unwrap().contains("未索引")),
            "result was: {}",
            result
        );
    }

    /// 验证 governance_chain_plan 输出包含上游健康状态和操作顺序
    #[tokio::test]
    async fn test_governance_chain_plan_basic() {
        let svc = make_service();
        svc.db
            .upsert_document(make_test_doc_with_upstream(
                "240610-1030-on-sihankor-canon",
                "spec",
                "3/3",
                None,
            ))
            .await
            .unwrap();
        svc.db
            .upsert_document(make_test_doc_with_upstream(
                "260616-1615-techne-spec",
                "spec",
                "1/3",
                Some("240610-1030-on-sihankor-canon"),
            ))
            .await
            .unwrap();

        let req = ChainPlanRequest {
            doc_id: "260616-1615-techne-spec".to_string(),
        };
        let result = svc.governance_chain_plan(Parameters(req)).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["plan_type"], "chain");
        assert_eq!(parsed["target_doc_id"], "260616-1615-techne-spec");
        let upstream = parsed["upstream_health"].as_array().unwrap();
        assert!(
            upstream.iter().any(
                |u| u["doc_id"] == "240610-1030-on-sihankor-canon" && u["status"] == "ratified"
            ),
            "result was: {}",
            result
        );
        let ops = parsed["operation_order"].as_array().unwrap();
        assert!(
            ops.iter().any(|o| o["phase"] == "advance_target"
                && o["affected_docs"]
                    .as_array()
                    .map(|a| a.iter().any(|d| d == "260616-1615-techne-spec"))
                    .unwrap_or(false)),
            "result was: {}",
            result
        );
    }

    /// 验证 code_audit_plan 对搜索命中的文档输出声明到代码 checkpoint 映射
    #[tokio::test]
    async fn test_code_audit_plan_basic() {
        let svc = make_service();
        svc.db
            .upsert_document(make_test_doc_with_upstream(
                "260616-1615-techne-spec",
                "spec",
                "1/3",
                None,
            ))
            .await
            .unwrap();
        svc.db
            .upsert_document(make_test_doc_with_upstream(
                "260616-1700-techne-orchestration-proposal",
                "proposal",
                "1/3",
                Some("260616-1615-techne-spec"),
            ))
            .await
            .unwrap();

        let req = CodeAuditPlanRequest {
            concept_hint: "techne".to_string(),
        };
        let result = svc.code_audit_plan(Parameters(req)).await;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["plan_type"], "audit");
        assert_eq!(parsed["concept"], "techne");
        let involved = parsed["involved_docs"].as_array().unwrap();
        assert!(
            involved.len() >= 2,
            "expected at least 2 docs, result was: {}",
            result
        );
        let checkpoints = parsed["checkpoints"].as_array().unwrap();
        assert!(
            !checkpoints.is_empty(),
            "expected checkpoints to be non-empty"
        );
        assert!(
            checkpoints
                .iter()
                .any(|c| c["code_checkpoint"].as_str().unwrap().contains("validator")),
            "expected spec checkpoint to mention validator"
        );
    }

    /// 验证 next_stage 辅助函数的四种 stage 行为
    #[test]
    fn test_next_stage_helper() {
        use crate::core::models::Stage;
        assert_eq!(next_stage(&Stage::Propose), Some("2/3".to_string()));
        assert_eq!(next_stage(&Stage::Resolve), Some("3/3".to_string()));
        assert_eq!(next_stage(&Stage::Ratify), None);
        assert_eq!(next_stage(&Stage::Deprecated), None);
        assert_eq!(next_stage(&Stage::Superseded("next-id".to_string())), None);
    }

    /// 验证 priority_rank 排序权重
    #[test]
    fn test_priority_rank_ordering() {
        assert!(priority_rank("critical") > priority_rank("warning"));
        assert!(priority_rank("warning") > priority_rank("info"));
        assert_eq!(priority_rank("unknown"), 0);
    }
}
