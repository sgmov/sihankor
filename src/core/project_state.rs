// ProjectState — 项目级流程推进状态
//
// 与 Document.stage（文档生命周期）构成两层状态机。
// 持久化到 .sih/project.json，为 suggest_next_action 提供数据源。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// ProjectState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub current_stage: String,
    pub active_proposal: Option<String>,
    pub last_action: String,
    pub pending_checkpoints: Vec<Checkpoint>,
    /// Per-document validation pass history for trust tracking
    pub doc_history: Vec<DocValidationRecord>,
    pub last_updated: DateTime<Utc>,
}

/// Tracks how many times a document has passed validation without errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocValidationRecord {
    pub doc_id: String,
    /// Consecutive validation passes (0 errors, 0 fatal)
    pub consecutive_passes: u32,
    /// Current trust tier: 0=untrusted, 1=verified, 2=trusted
    pub trust_tier: u8,
    pub last_validated: DateTime<Utc>,
}

impl DocValidationRecord {
    pub fn new(doc_id: &str) -> Self {
        Self {
            doc_id: doc_id.to_string(),
            consecutive_passes: 0,
            trust_tier: 0,
            last_validated: Utc::now(),
        }
    }

    /// Record a pass - no errors/warnings at F/J level
    pub fn record_pass(&mut self) {
        self.consecutive_passes += 1;
        self.last_validated = Utc::now();
        // 3 consecutive passes → verified, 10 → trusted
        self.trust_tier = if self.consecutive_passes >= 10 {
            2
        } else if self.consecutive_passes >= 3 {
            1
        } else {
            0
        };
    }

    /// Record a failure - resets trust
    pub fn record_fail(&mut self) {
        self.consecutive_passes = 0;
        self.trust_tier = 0;
        self.last_validated = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub label: String,
    pub document_id: String,
    pub decision: Option<String>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            current_stage: "spec".into(),
            active_proposal: None,
            last_action: "project initialized".into(),
            pending_checkpoints: Vec::new(),
            doc_history: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    /// 加载 .sih/project.json
    pub fn load(project_root: &Path) -> Result<Self, String> {
        let path = project_root.join(".sih").join("project.json");
        if !path.exists() {
            return Ok(Self::new());
        }
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("read project.json: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("parse project.json: {}", e))
    }

    /// 保存到 .sih/project.json
    pub fn save(&mut self, project_root: &Path) -> Result<(), String> {
        let dir = project_root.join(".sih");
        std::fs::create_dir_all(&dir).map_err(|e| format!("create .sih dir: {}", e))?;
        let path = dir.join("project.json");
        self.last_updated = Utc::now();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize project.json: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("write project.json: {}", e))
    }

    /// 添加待确认检查点
    pub fn add_checkpoint(&mut self, label: &str, document_id: &str) {
        let id = format!("cp-{}", self.pending_checkpoints.len() + 1);
        self.pending_checkpoints.push(Checkpoint {
            id,
            label: label.into(),
            document_id: document_id.into(),
            decision: None,
        });
    }

    /// 确认检查点
    pub fn confirm_checkpoint(&mut self, checkpoint_id: &str) {
        for cp in &mut self.pending_checkpoints {
            if cp.id == checkpoint_id {
                cp.decision = Some("confirmed".into());
                break;
            }
        }
    }

    /// 设置活跃提案
    pub fn set_active_proposal(&mut self, proposal_id: &str) {
        self.active_proposal = Some(proposal_id.into());
        self.last_action = format!("activated proposal {}", proposal_id);
    }

    /// 推进阶段
    pub fn advance_stage(&mut self, new_stage: &str) {
        self.current_stage = new_stage.into();
        self.last_action = format!("advanced to stage {}", new_stage);
    }

    /// 获取或创建文档的验证记录
    pub fn get_or_create_doc_record(&mut self, doc_id: &str) -> &mut DocValidationRecord {
        if !self.doc_history.iter().any(|r| r.doc_id == doc_id) {
            self.doc_history.push(DocValidationRecord::new(doc_id));
        }
        self.doc_history
            .iter_mut()
            .find(|r| r.doc_id == doc_id)
            .unwrap()
    }

    /// 记录文档验证通过（无 F/J 级违规）
    pub fn record_doc_pass(&mut self, doc_id: &str) {
        self.get_or_create_doc_record(doc_id).record_pass();
    }

    /// 记录文档验证失败（存在 F 或 J 级违规）
    pub fn record_doc_fail(&mut self, doc_id: &str) {
        self.get_or_create_doc_record(doc_id).record_fail();
    }

    /// 查询文档的信任层级
    pub fn doc_trust_tier(&self, doc_id: &str) -> u8 {
        self.doc_history
            .iter()
            .find(|r| r.doc_id == doc_id)
            .map(|r| r.trust_tier)
            .unwrap_or(0)
    }
}

impl Default for ProjectState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// suggest_next_action
// ---------------------------------------------------------------------------

/// 操作建议
#[derive(Debug, Clone, Serialize)]
pub struct ActionSuggestion {
    pub priority: u8, // 1 = 立即, 2 = 建议, 3 = 可选
    pub action: String,
    pub reason: String,
    pub target_id: Option<String>,
}

/// 基于 ProjectState + 文档状态推导下一步建议
pub fn suggest_next_action(
    state: &ProjectState,
    _db: &dyn crate::core::database::SihDatabase,
) -> Vec<ActionSuggestion> {
    let mut suggestions = Vec::new();

    // Rule 0: pending correction tasks (from dashboard) — highest priority
    let corr_dir = std::path::Path::new(".sih/corrections");
    if corr_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(corr_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(task) = serde_json::from_str::<serde_json::Value>(&content) {
                        let doc_id = task["doc_id"].as_str().unwrap_or("?");
                        let title = task["doc_title"].as_str().unwrap_or("?");
                        let issues: Vec<&str> = task["issues"]
                            .as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                            .unwrap_or_default();
                        suggestions.push(ActionSuggestion {
                            priority: 1,
                            action: format!(
                                "修正文档 {}: {} — 问题: {}",
                                doc_id,
                                title,
                                issues.join(", ")
                            ),
                            reason: "有待处理的合道修正任务".into(),
                            target_id: Some(doc_id.to_string()),
                        });
                    }
                }
            }
        }
    }

    // Rule 1: pending checkpoints
    let pending: Vec<_> = state
        .pending_checkpoints
        .iter()
        .filter(|cp| cp.decision.is_none())
        .collect();
    if !pending.is_empty() {
        for cp in &pending {
            suggestions.push(ActionSuggestion {
                priority: 1,
                action: format!("确认检查点: {}", cp.label),
                reason: "有待确认的检查点".into(),
                target_id: Some(cp.document_id.clone()),
            });
        }
    }

    // Rule 2: 1/3 documents whose upstreams are all >= 2/3 — suggest advancement
    // (async context not available in sync fn, skip DB queries for now;
    //  this will be enhanced when suggest_next_action moves to async)
    suggestions.push(ActionSuggestion {
        priority: 2,
        action: "查看 1/3 文档，检查是否有 upstream 已就绪可推进的".into(),
        reason: "上游就绪的 1/3 文档可以推进到 2/3".into(),
        target_id: None,
    });

    // Rule 3: 2/3 proposals without decisions
    suggestions.push(ActionSuggestion {
        priority: 2,
        action: "检查 2/3 proposal 是否有对应的 decision".into(),
        reason: "2/3 proposal 通过审阅后应起草 decision".into(),
        target_id: state.active_proposal.clone(),
    });

    // Rule 4: stage progression hint
    let next_stage = match state.current_stage.as_str() {
        "spec" => "起草 proposal 将 spec 落地为变更方案",
        "proposal" => "起草 decision 记录架构决策",
        "decision" => "进入 code 阶段实现",
        "code" => "进入 verify 阶段验证",
        "verify" => "所有文档 3/3，治理链完整",
        _ => "查看 kanban 了解整体状态",
    };
    suggestions.push(ActionSuggestion {
        priority: 3,
        action: format!("当前阶段: {} → {}", state.current_stage, next_stage),
        reason: format!("项目当前处于 {} 阶段", state.current_stage),
        target_id: None,
    });

    suggestions
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_state_new() {
        let state = ProjectState::new();
        assert_eq!(state.current_stage, "spec");
        assert!(state.active_proposal.is_none());
        assert!(state.pending_checkpoints.is_empty());
    }

    #[test]
    fn test_add_and_confirm_checkpoint() {
        let mut state = ProjectState::new();
        state.add_checkpoint("test checkpoint", "doc-1");
        assert_eq!(state.pending_checkpoints.len(), 1);
        assert!(state.pending_checkpoints[0].decision.is_none());

        let cp_id = state.pending_checkpoints[0].id.clone();
        state.confirm_checkpoint(&cp_id);
        assert_eq!(
            state.pending_checkpoints[0].decision,
            Some("confirmed".into())
        );
    }

    #[test]
    fn test_set_active_proposal() {
        let mut state = ProjectState::new();
        state.set_active_proposal("prop-001");
        assert_eq!(state.active_proposal, Some("prop-001".into()));
    }

    #[test]
    fn test_advance_stage() {
        let mut state = ProjectState::new();
        state.advance_stage("proposal");
        assert_eq!(state.current_stage, "proposal");
    }

    #[test]
    fn test_save_and_load() {
        let tmp = std::env::temp_dir().join("sihankor-test-project-state");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let mut state = ProjectState::new();
        state.set_active_proposal("prop-001");
        state.add_checkpoint("review spec", "doc-1");
        state.save(&tmp).unwrap();

        let loaded = ProjectState::load(&tmp).unwrap();
        assert_eq!(loaded.current_stage, "spec");
        assert_eq!(loaded.active_proposal, Some("prop-001".into()));
        assert_eq!(loaded.pending_checkpoints.len(), 1);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_load_missing_returns_default() {
        let tmp = std::env::temp_dir().join("sihankor-test-nonexistent");
        let state = ProjectState::load(&tmp).unwrap();
        assert_eq!(state.current_stage, "spec");
    }
}
