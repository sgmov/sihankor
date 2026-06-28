//! 跨文档治理检查
//!
//! 现有 validator.rs 是 per-document 检查：每篇文档独立验证 frontmatter / structure / content。
//! 本模块提供 cross-document 检查：需要完整的文档列表才能判断的治理规则。
//!
//! 知止边界：只检查现有规则未能表达的 cross-cutting 问题，不引入新的 per-document 规则。
//! 现有 V-F-04（upstream 必填）和 V-G-08（X 文档不可引用）只检查单篇文档的状态，
//! 不检查引用关系是否完整或合法——这正是本模块要补全的盲区。

use std::collections::HashMap;

use super::models::{Document, Stage};

/// 上游链完整性问题
///
/// 描述一篇文档的 upstream 引用存在的具体问题。
/// upstream 必填性（V-F-04）已由 validator 处理，本结构只关注**完整性**：
/// 引用的文档是否存在、是否可引用、是否存在循环。
#[derive(Debug, Clone)]
pub struct UpstreamChainIssue {
    /// 问题文档的 id
    pub doc_id: String,
    /// 文档的当前 stage（用于定位）
    pub doc_stage: String,
    /// 引用的 upstream id
    pub upstream_id: String,
    /// 问题原因
    pub reason: UpstreamChainIssueKind,
}

/// 上游链完整性问题的具体原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpstreamChainIssueKind {
    /// 引用的 upstream id 在文档集合中不存在
    NotFound,
    /// upstream 存在但处于 1/3（Propose），尚未定稿，不可被引用
    NotReferenceable,
    /// upstream 处于 X（Deprecated），不应被新文档引用
    Deprecated,
    /// upstream 处于 0/<id>（Superseded），本身已被替代
    Superseded,
    /// upstream 处于 0/<id>，但替代它的 successor id 也不存在或不可引用
    SupersededChainBroken,
}

impl UpstreamChainIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotFound => "upstream_not_found",
            Self::NotReferenceable => "upstream_not_referenceable",
            Self::Deprecated => "upstream_deprecated",
            Self::Superseded => "upstream_superseded",
            Self::SupersededChainBroken => "upstream_superseded_chain_broken",
        }
    }
}

/// stage 转换合法性问题
///
/// 描述跨文档的 stage 关系问题：废弃文档被引用、Propose 文档被引用、Superseded 指向不存在。
#[derive(Debug, Clone)]
pub struct StageTransitionIssue {
    /// 问题文档的 id
    pub doc_id: String,
    /// 文档的当前 stage
    pub current_stage: String,
    /// 问题原因
    pub reason: StageTransitionIssueKind,
}

/// stage 转换合法性问题的具体原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageTransitionIssueKind {
    /// 0/<id> 文档引用的 successor id 不存在
    SupersededTargetNotFound,
    /// 处于 X 的文档被其他活跃文档引用（应迁移引用）
    ReferencedWhileDeprecated,
    /// 处于 1/3 的文档被其他文档引用（V-G-07 spirit：Propose 不可被引用）
    ReferencedWhilePropose,
}

impl StageTransitionIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SupersededTargetNotFound => "superseded_target_not_found",
            Self::ReferencedWhileDeprecated => "referenced_while_deprecated",
            Self::ReferencedWhilePropose => "referenced_while_propose",
        }
    }
}

/// 索引文档集合便于按 id 查询
fn index_docs(docs: &[Document]) -> HashMap<&str, &Document> {
    docs.iter().map(|d| (d.id.as_str(), d)).collect()
}

/// 上游链完整性检查
///
/// 对每篇**非 note** 文档（note 文档 V-F-04 允许 upstream 为空）：
/// - 如果 upstream == self.id：root doc，合法
/// - 否则检查 upstream 是否存在、是否处于可引用状态
pub fn check_upstream_chain(docs: &[Document]) -> Vec<UpstreamChainIssue> {
    let mut issues = Vec::new();
    let by_id = index_docs(docs);

    for doc in docs {
        // note 文档允许 upstream 为空（V-F-04 例外）
        if doc.nature == "note" {
            continue;
        }

        // trail 文档有自己的知止边界（trail 不强制 upstream）
        if doc.nature == "trail" {
            continue;
        }

        // session_summary 文档不强制 upstream
        if doc.nature == "session_summary" {
            continue;
        }

        let upstream_id = match &doc.upstream {
            Some(id) => id,
            None => continue, // V-F-04 已处理必填性
        };

        // root doc: upstream 自指向
        if upstream_id == &doc.id {
            continue;
        }

        // 检查 upstream 存在性
        let upstream = match by_id.get(upstream_id.as_str()) {
            Some(d) => d,
            None => {
                issues.push(UpstreamChainIssue {
                    doc_id: doc.id.clone(),
                    doc_stage: doc.stage.to_display(),
                    upstream_id: upstream_id.clone(),
                    reason: UpstreamChainIssueKind::NotFound,
                });
                continue;
            }
        };

        // 检查 upstream 是否可引用
        match &upstream.stage {
            Stage::Resolve | Stage::Ratify => {
                // 可引用，OK
            }
            Stage::Propose => {
                issues.push(UpstreamChainIssue {
                    doc_id: doc.id.clone(),
                    doc_stage: doc.stage.to_display(),
                    upstream_id: upstream_id.clone(),
                    reason: UpstreamChainIssueKind::NotReferenceable,
                });
            }
            Stage::Deprecated => {
                issues.push(UpstreamChainIssue {
                    doc_id: doc.id.clone(),
                    doc_stage: doc.stage.to_display(),
                    upstream_id: upstream_id.clone(),
                    reason: UpstreamChainIssueKind::Deprecated,
                });
            }
            Stage::Superseded(successor_id) => {
                // upstream 本身被替代。检查 successor 是否可解析。
                match by_id.get(successor_id.as_str()) {
                    Some(succ) if succ.stage.is_referenceable() => {
                        // 替代链完整，警告但不阻断
                        issues.push(UpstreamChainIssue {
                            doc_id: doc.id.clone(),
                            doc_stage: doc.stage.to_display(),
                            upstream_id: upstream_id.clone(),
                            reason: UpstreamChainIssueKind::Superseded,
                        });
                    }
                    _ => {
                        // 替代链断裂
                        issues.push(UpstreamChainIssue {
                            doc_id: doc.id.clone(),
                            doc_stage: doc.stage.to_display(),
                            upstream_id: upstream_id.clone(),
                            reason: UpstreamChainIssueKind::SupersededChainBroken,
                        });
                    }
                }
            }
        }
    }

    issues
}

/// stage 转换合法性检查
///
/// 三类跨文档问题：
/// 1. Superseded 文档的 successor id 不存在
/// 2. X 文档被其他文档引用（应迁移）
/// 3. Propose 文档被其他文档引用（V-G-07 spirit）
pub fn check_stage_transitions(docs: &[Document]) -> Vec<StageTransitionIssue> {
    let mut issues = Vec::new();
    let by_id = index_docs(docs);

    // 反向索引：哪些文档引用了给定的 id
    let mut referenced_by: HashMap<&str, Vec<&Document>> = HashMap::new();
    for doc in docs {
        if let Some(upstream) = &doc.upstream {
            if upstream != &doc.id {
                referenced_by
                    .entry(upstream.as_str())
                    .or_default()
                    .push(doc);
            }
        }
    }

    for doc in docs {
        // 1. Superseded 文档的 successor id 必须存在
        if let Stage::Superseded(successor_id) = &doc.stage {
            if !by_id.contains_key(successor_id.as_str()) {
                issues.push(StageTransitionIssue {
                    doc_id: doc.id.clone(),
                    current_stage: doc.stage.to_display(),
                    reason: StageTransitionIssueKind::SupersededTargetNotFound,
                });
            }
        }

        // 2. X 文档被其他文档引用
        if matches!(doc.stage, Stage::Deprecated) {
            if let Some(referrers) = referenced_by.get(doc.id.as_str()) {
                for referrer in referrers {
                    // 排除自身（已在 index_docs 中处理过）
                    if referrer.id != doc.id {
                        issues.push(StageTransitionIssue {
                            doc_id: referrer.id.clone(),
                            current_stage: referrer.stage.to_display(),
                            reason: StageTransitionIssueKind::ReferencedWhileDeprecated,
                        });
                    }
                }
            }
        }

        // 3. Propose 文档被其他文档引用（V-G-07 spirit：1/3 不可被引用）
        if matches!(doc.stage, Stage::Propose) {
            if let Some(referrers) = referenced_by.get(doc.id.as_str()) {
                for referrer in referrers {
                    if referrer.id != doc.id {
                        issues.push(StageTransitionIssue {
                            doc_id: referrer.id.clone(),
                            current_stage: referrer.stage.to_display(),
                            reason: StageTransitionIssueKind::ReferencedWhilePropose,
                        });
                    }
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{DocStatus, Frontmatter, Stage};
    use chrono::Utc;

    fn make_doc(id: &str, stage: Stage, upstream: Option<&str>, nature: &str) -> Document {
        Document {
            id: id.to_string(),
            stage: stage.clone(),
            title: format!("Test {}", id),
            upstream: upstream.map(|s| s.to_string()),
            frontmatter: Frontmatter {
                id: id.to_string(),
                stage,
                upstream: upstream.map(|s| s.to_string()),
                decided_by: None,
                extra: serde_json::Value::Null,
            },
            content: String::new(),
            status: DocStatus::Ok,
            indexed_at: Utc::now(),
            nature: nature.to_string(),
        }
    }

    // ── upstream chain ──

    #[test]
    fn test_upstream_chain_root_self_reference_ok() {
        let docs = vec![make_doc("root-1", Stage::Ratify, Some("root-1"), "spec")];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_upstream_chain_valid_reference_ok() {
        let docs = vec![
            make_doc("root", Stage::Ratify, Some("root"), "spec"),
            make_doc("child", Stage::Propose, Some("root"), "proposal"),
        ];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_upstream_chain_not_found() {
        let docs = vec![make_doc(
            "orphan",
            Stage::Propose,
            Some("ghost"),
            "proposal",
        )];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].reason, UpstreamChainIssueKind::NotFound);
    }

    #[test]
    fn test_upstream_chain_not_referenceable() {
        let docs = vec![
            make_doc("proposer", Stage::Propose, Some("proposer"), "proposal"),
            make_doc("child", Stage::Propose, Some("proposer"), "decision"),
        ];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].reason, UpstreamChainIssueKind::NotReferenceable);
    }

    #[test]
    fn test_upstream_chain_deprecated() {
        let docs = vec![
            make_doc("old", Stage::Deprecated, Some("old"), "spec"),
            make_doc("child", Stage::Propose, Some("old"), "decision"),
        ];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].reason, UpstreamChainIssueKind::Deprecated);
    }

    #[test]
    fn test_upstream_chain_superseded_with_valid_successor_warns() {
        let docs = vec![
            make_doc("old", Stage::Superseded("new".into()), Some("old"), "spec"),
            make_doc("new", Stage::Ratify, Some("new"), "spec"),
            make_doc("child", Stage::Propose, Some("old"), "decision"),
        ];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].reason, UpstreamChainIssueKind::Superseded);
    }

    #[test]
    fn test_upstream_chain_superseded_broken_chain() {
        let docs = vec![
            make_doc(
                "old",
                Stage::Superseded("ghost".into()),
                Some("old"),
                "spec",
            ),
            make_doc("child", Stage::Propose, Some("old"), "decision"),
        ];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].reason,
            UpstreamChainIssueKind::SupersededChainBroken
        );
    }

    #[test]
    fn test_upstream_chain_note_skipped() {
        // note 文档允许 upstream 为空
        let docs = vec![make_doc("note-1", Stage::Propose, None, "note")];
        let issues = check_upstream_chain(&docs);
        assert_eq!(issues.len(), 0);
    }

    // ── stage transitions ──

    #[test]
    fn test_stage_transition_superseded_target_not_found() {
        let docs = vec![make_doc(
            "old",
            Stage::Superseded("ghost".into()),
            Some("old"),
            "spec",
        )];
        let issues = check_stage_transitions(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].reason,
            StageTransitionIssueKind::SupersededTargetNotFound
        );
    }

    #[test]
    fn test_stage_transition_superseded_target_found_ok() {
        let docs = vec![
            make_doc("old", Stage::Superseded("new".into()), Some("old"), "spec"),
            make_doc("new", Stage::Ratify, Some("new"), "spec"),
        ];
        let issues = check_stage_transitions(&docs);
        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_stage_transition_referenced_while_deprecated() {
        let docs = vec![
            make_doc("old", Stage::Deprecated, Some("old"), "spec"),
            make_doc("child", Stage::Propose, Some("old"), "decision"),
        ];
        let issues = check_stage_transitions(&docs);
        // child 引用了 deprecated 文档
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].reason,
            StageTransitionIssueKind::ReferencedWhileDeprecated
        );
        assert_eq!(issues[0].doc_id, "child");
    }

    #[test]
    fn test_stage_transition_referenced_while_propose() {
        let docs = vec![
            make_doc("proposer", Stage::Propose, Some("proposer"), "spec"),
            make_doc("child", Stage::Propose, Some("proposer"), "decision"),
        ];
        let issues = check_stage_transitions(&docs);
        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].reason,
            StageTransitionIssueKind::ReferencedWhilePropose
        );
        assert_eq!(issues[0].doc_id, "child");
    }
}
