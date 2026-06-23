use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 文档生命周期阶段
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stage(pub String);

impl Stage {
    pub fn propose() -> Self {
        Stage("1/3".to_string())
    }
    pub fn resolve() -> Self {
        Stage("2/3".to_string())
    }
    pub fn ratify() -> Self {
        Stage("3/3".to_string())
    }
    pub fn deprecated() -> Self {
        Stage("X".to_string())
    }

    /// 是否为有效阶段编码
    pub fn is_valid(&self) -> bool {
        let s = &self.0;
        s == "1/3" || s == "2/3" || s == "3/3" || s == "X" || s.starts_with("0/")
    }

    /// 阶段是否可被引用（1/3 不可引用，X 禁止引用）
    pub fn is_referenceable(&self) -> bool {
        let s = &self.0;
        s == "2/3" || s == "3/3"
    }

    /// 是否为终止状态
    pub fn is_terminal(&self) -> bool {
        self.0 == "X" || self.0.starts_with("0/")
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
    pub fn as_str(&self) -> &'static str {
        match self {
            DocStatus::Ok => "Ok",
            DocStatus::Warning => "Warning",
            DocStatus::Error => "Error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
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
    /// 道法追溯：此规则对应哪个司衡道法维度（知止/顺因/有度/损补/顺势）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dao_trace: Option<String>,
}

/// 违规力度级别：F（戒）/ G（规）/ J（矩）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Fatal,
    Guideline,
    Judgment,
}

impl ViolationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViolationSeverity::Fatal => "F",
            ViolationSeverity::Guideline => "G",
            ViolationSeverity::Judgment => "J",
        }
    }
}
