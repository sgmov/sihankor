use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 顶层分析结果
// ---------------------------------------------------------------------------

/// 完整分析报告（三机流转的最终产物）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub schema_version: String,
    pub analysis_id: String,
    pub analysis_target: AnalysisTarget,

    /// iCL 产出
    pub cognition: Cognition,

    /// iWW 产出（仅 full_analysis/propose_decision 必填）
    pub decision_proposal: Option<DecisionProposal>,

    /// iCT 产出（仅 full_analysis/verify_decision 必填）
    pub verification: Option<Verification>,

    /// 道四：必填
    pub limitations: Vec<Limitation>,
    pub self_question: String,
    pub human_review_required: Vec<String>,
}

/// 分析目标描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTarget {
    pub id: String,
    pub title: String,
    pub nature: String,
    pub stage: String,
}

// ---------------------------------------------------------------------------
// iCL 明晰机 —— 认知产出
// ---------------------------------------------------------------------------

/// 三机第一机：认知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cognition {
    pub governance_position: GovPosition,
    pub relation_graph: RelationGraph,
    pub divergence_diagnosis: Vec<Divergence>,
}

/// 文档治理定位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovPosition {
    pub nature: String,
    pub stage: String,
    pub upstream_chain: Vec<String>,
    /// 在治理链中的角色（auth/derive/leaf）
    pub role_in_chain: ChainRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChainRole {
    /// 根文档（无上游）
    Root,
    /// 授权源（有下游依赖）
    Auth,
    /// 派生文档（有上游）
    Derive,
    /// 叶节点（有上游，无下游）
    Leaf,
}

/// 跨文档关系图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationGraph {
    /// 被本文档明确引用的文档 id 列表
    pub references: Vec<String>,
    /// 与本文档讨论同一主题的其他文档
    pub duplicates: Vec<DuplicateInfo>,
    /// 与本文档声明相矛盾的文档
    pub conflicts: Vec<ConflictInfo>,
    /// 被引用但目标缺失的文档 id
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateInfo {
    pub doc_id: String,
    pub overlap: OverlapDegree,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlapDegree {
    Exact,
    High,
    Partial,
    Negligible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub doc_id: String,
    pub claim: String,
    pub counter_claim: String,
}

/// 发散诊断项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub div_type: DivergenceType,
    pub severity: DivergenceSeverity,
    pub confidence: f64,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivergenceType {
    /// 意图漂移：文档内容偏离 upstream 意图
    IntentDrift,
    /// 引用断裂：被引用的文档不存在或 stage 不可引用
    ReferenceBreak,
    /// 重复冗余：多个文档讨论同一主题
    Duplication,
    /// 空白缺口：缺失必要的下游文档
    Gap,
    /// 良性发散：多视角讨论，不需要修复
    BenignDivergence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DivergenceSeverity {
    Critical,
    Warning,
    Info,
}

// ---------------------------------------------------------------------------
// iWW 消息机 —— 决策产出
// ---------------------------------------------------------------------------

/// 三机第二机：决策建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionProposal {
    pub recommended_action: Action,
    pub rationale: Rationale,
    pub alternatives: Vec<Alternative>,
    pub affected_documents: AffectedDocs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub kind: ActionKind,
    pub description: String,
    pub revert_steps: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Merge,
    Move,
    Rename,
    Archive,
    NoAction,
    HumanReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rationale {
    pub dao_basis: String,
    pub fa_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub action: ActionKind,
    pub pros: String,
    pub cons: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedDocs {
    pub direct: Vec<String>,
    pub indirect: Vec<String>,
    pub before_after: Vec<BeforeAfter>,
}

/// dry-run 变更预览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeforeAfter {
    pub doc_id: String,
    pub field: String,
    pub before: String,
    pub after: String,
}

// ---------------------------------------------------------------------------
// iCT 方圆机 —— 验证产出
// ---------------------------------------------------------------------------

/// 三机第三机：合道验证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub five_law_check: Vec<LawCheck>,
    pub overall: Verdict,
    pub law_violation_summary: Vec<LawViolationSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawCheck {
    pub law: String,
    pub result: LawCheckResult,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LawCheckResult {
    Pass,
    Fail,
    Conditional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pass,
    Fail,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawViolationSummary {
    pub laws: String,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// 通用
// ---------------------------------------------------------------------------

/// 认知盲区声明（道四强制）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limitation {
    pub aspect: String,
    pub reason: String,
    pub confidence: f64,
}
