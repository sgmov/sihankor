use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 文档生命周期阶段
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Stage {
    /// 1/3 — 提议阶段
    Propose,
    /// 2/3 — 决议阶段
    Resolve,
    /// 3/3 — 定稿阶段
    Ratify,
    /// X — 废弃
    Deprecated,
    /// 0/<successor-id> — 已被替代
    Superseded(String),
}

impl Stage {
    pub fn propose() -> Self { Stage::Propose }
    pub fn resolve() -> Self { Stage::Resolve }
    pub fn ratify() -> Self { Stage::Ratify }
    pub fn deprecated() -> Self { Stage::Deprecated }

    pub fn as_str(&self) -> &str {
        match self {
            Stage::Propose => "1/3",
            Stage::Resolve => "2/3",
            Stage::Ratify => "3/3",
            Stage::Deprecated => "X",
            Stage::Superseded(_) => "0/",
        }
    }

    pub fn to_display(&self) -> String {
        match self {
            Stage::Propose => "1/3".into(),
            Stage::Resolve => "2/3".into(),
            Stage::Ratify => "3/3".into(),
            Stage::Deprecated => "X".into(),
            Stage::Superseded(id) => format!("0/{}", id),
        }
    }

    pub fn is_valid(&self) -> bool { true }

    pub fn is_referenceable(&self) -> bool {
        matches!(self, Stage::Resolve | Stage::Ratify)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Stage::Deprecated | Stage::Superseded(_))
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1/3" => Some(Stage::Propose),
            "2/3" => Some(Stage::Resolve),
            "3/3" => Some(Stage::Ratify),
            "X" => Some(Stage::Deprecated),
            s if s.starts_with("0/") => Some(Stage::Superseded(s[2..].to_string())),
            _ => None,
        }
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_display())
    }
}

impl Serialize for Stage {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_display())
    }
}

impl<'de> Deserialize<'de> for Stage {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Stage::from_str(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid stage: {}", s)))
    }
}

/// 文档解析/验证状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    Ok,
    Warning,
    Error,
}

impl DocStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DocStatus::Ok => "Ok",
            DocStatus::Warning => "Warning",
            DocStatus::Error => "Error",
        }
    }

    pub fn from_status_str(s: &str) -> Option<Self> {
        match s {
            "Ok" => Some(DocStatus::Ok),
            "Warning" => Some(DocStatus::Warning),
            "Error" => Some(DocStatus::Error),
            _ => None,
        }
    }
}

/// Frontmatter 结构体：展开引擎需要的字段 + extra 兜底
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub id: String,
    pub stage: Stage,
    pub upstream: Option<String>,
    pub decided_by: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// 文档：引擎治理的基本单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub stage: Stage,
    pub title: String,
    pub upstream: Option<String>,
    pub frontmatter: Frontmatter,
    pub content: String,
    pub status: DocStatus,
    pub indexed_at: DateTime<Utc>,
    pub nature: String,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub stage: Stage,
    pub title: String,
    pub snippet: String,
    pub relevance: f64,
}

/// 授权链节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNode {
    pub id: String,
    pub stage: Stage,
    pub title: String,
    pub upstream: Option<String>,
    pub depth: u32,
}

/// 验证违规
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub location: String,
    /// 修复建议（面向开发者的可操作指引）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_suggestion: Option<String>,
}

/// 违规力度级别：F（戒）/ G（规）/ J（矩）
///
/// J 语义决策说明：旧哲学层（archive/philosophy-v1/SiHankor-Engineering-Mapping.sih.md）
/// 将 J（矩）定义为"精确判定 pass/fail"的强机械判定。代码实现选择将其反转为
/// "静默记录"（Judgment severity，仅计数不阻断），这是更合理的设计：J 级规则
/// 属于风格性判断，强行 pass/fail 会产生噪声阻断。本实现以代码语义为准，
/// J = 静默记录。详见 validator::ValidationResult::to_structured_report 的
/// J 语义决策说明与 R4 工程映射审计 4.7 冲突二。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Fatal,
    Guideline,
    Judgment,
}

impl ViolationSeverity {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ViolationSeverity::Fatal => "F",
            ViolationSeverity::Guideline => "G",
            ViolationSeverity::Judgment => "J",
        }
    }
}
