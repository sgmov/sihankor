use std::sync::Arc;

use crate::core::database::SihDatabase;
use crate::core::models::Document;

use super::types::{Cognition, GovPosition, ChainRole, RelationGraph, DuplicateInfo, OverlapDegree, ConflictInfo, Divergence, DivergenceType, DivergenceSeverity};

/// iCL 明晰机 —— 三机第一机
///
/// 对输入文档执行四步分析法前三步（意图定位/关系照见/发散诊断）。
/// 不写文件，不做决策——只产出 Cognition。
pub struct ICL {
    db: Arc<dyn SihDatabase>,
}

impl ICL {
    pub fn new(db: Arc<dyn SihDatabase>) -> Self {
        Self { db }
    }

    /// 暴露数据库引用（供术层编排工具使用）
    pub fn db(&self) -> &Arc<dyn SihDatabase> {
        &self.db
    }

    /// 对单个文档执行认知分析
    pub async fn analyze(&self, doc: &Document) -> Cognition {
        let governance_position = self.governance_position(doc).await;
        let relation_graph = self.relation_graph(doc).await;
        let divergence_diagnosis = self.diagnose_divergences(doc, &governance_position, &relation_graph);

        Cognition {
            governance_position,
            relation_graph,
            divergence_diagnosis,
        }
    }

    // ------------------------------------------------------------------
    // ① 意图定位
    // ------------------------------------------------------------------

    async fn governance_position(&self, doc: &Document) -> GovPosition {
        let upstream_chain = self.resolve_upstream_chain(&doc.id).await;
        let downstream = self.find_downstream_docs(&doc.id).await;

        let role_in_chain = match (upstream_chain.is_empty(), downstream.is_empty()) {
            (true, true) => ChainRole::Root,
            (true, false) => ChainRole::Auth,
            (false, true) => ChainRole::Leaf,
            (false, false) => ChainRole::Derive,
        };

        GovPosition {
            nature: doc.nature.clone(),
            stage: doc.stage.0.clone(),
            upstream_chain,
            role_in_chain,
        }
    }

    async fn resolve_upstream_chain(&self, id: &str) -> Vec<String> {
        match self.db.resolve_chain(id, 20).await {
            Ok(nodes) => nodes
                .iter()
                .filter(|n| n.id != id)  // 排除自引用
                .map(|n| n.id.clone())
                .collect(),
            Err(_) => vec![],
        }
    }

    async fn find_downstream_docs(&self, id: &str) -> Vec<String> {
        // 全文搜索引用该 id 的文档（实用近似：其他文档 content 中包含 id 字符串）
        match self.db.search_content(id).await {
            Ok(results) => results
                .iter()
                .filter(|r| r.id != id)  // 排除自身
                .map(|r| r.id.clone())
                .collect(),
            Err(_) => vec![],
        }
    }

    // ------------------------------------------------------------------
    // ② 关系照见
    // ------------------------------------------------------------------

    async fn relation_graph(&self, doc: &Document) -> RelationGraph {
        let references = self.extract_references(&doc.content);
        let duplicates = self.find_duplicates(&doc.id, &doc.title).await;
        let conflicts = self.find_conflicts(doc).await;
        let gaps = self.find_gaps(&references).await;

        RelationGraph {
            references,
            duplicates,
            conflicts,
            gaps,
        }
    }

    /// 从文档内容中提取引用：识别形如 `[YYYYMMDD-HHMM-...]` 的文档 id
    fn extract_references(&self, content: &str) -> Vec<String> {
        let mut ids = Vec::new();
        // 匹配 YYMMDD-HHMM[-NNN]-slug 格式
        let re = regex_lite::Regex::new(r"\b(\d{6}-\d{4}(?:-\d{1,5})?-[a-z][a-z0-9-]+)\b");
        if let Ok(re) = re {
            for m in re.find_iter(content) {
                let id = m.as_str().to_string();
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        ids
    }

    async fn find_duplicates(&self, current_id: &str, title: &str) -> Vec<DuplicateInfo> {
        let mut dupes = Vec::new();
        // 用标题中的关键词搜索
        let keywords: Vec<&str> = title
            .split(|c: char| c.is_whitespace() || c == '-' || c == '：' || c == ':')
            .filter(|w| w.len() > 3)
            .take(3)
            .collect();

        for kw in &keywords {
            if let Ok(results) = self.db.search_content(kw).await {
                for r in results {
                    if r.id == current_id {
                        continue;
                    }
                    // 简单启发：标题相似度
                    let overlap = if r.title == title {
                        OverlapDegree::Exact
                    } else if r.title.contains(title) || title.contains(&r.title) {
                        OverlapDegree::High
                    } else {
                        OverlapDegree::Partial
                    };
                    if overlap != OverlapDegree::Partial {
                        dupes.push(DuplicateInfo {
                            doc_id: r.id,
                            overlap,
                            description: format!("标题相似：'{}' vs '{}'", title, r.title),
                        });
                    }
                }
            }
        }

        dupes
    }

    async fn find_conflicts(&self, doc: &Document) -> Vec<ConflictInfo> {
        let mut conflicts = Vec::new();

        // 检查上游文档 stage 是否可引用
        if let Some(ref upstream) = doc.upstream
            && let Ok(Some(up_doc)) = self.db.get_document(upstream).await
                && !up_doc.stage.is_referenceable() {
                    conflicts.push(ConflictInfo {
                        doc_id: upstream.clone(),
                        claim: format!("stage {} 是有效的引用来源", up_doc.stage.0),
                        counter_claim: format!(
                            "stage {} 不在可引用范围内（需 2/3 或 3/3）",
                            up_doc.stage.0
                        ),
                    });
                }

        // 检查 nature 一致性：上游与下游 nature 是否构成合法治理链
        // 合法链：spec → proposal → decision → spec → ...
        // 简化版：上游为 spec 时下游应为 proposal/decision/spec
        //         上游为 proposal 时下游应为 decision
        if let Some(ref upstream) = doc.upstream
            && let Ok(Some(up_doc)) = self.db.get_document(upstream).await {
                let legal = Self::is_legal_chain(&up_doc.nature, &doc.nature);
                if !legal {
                    conflicts.push(ConflictInfo {
                        doc_id: upstream.clone(),
                        claim: format!("nature '{}' 是 '{}' 的合法上游", up_doc.nature, doc.nature),
                        counter_claim: format!(
                            "nature '{}' 不能直接下游到 '{}'",
                            up_doc.nature, doc.nature
                        ),
                    });
                }
            }

        conflicts
    }

    fn is_legal_chain(upstream_nature: &str, downstream_nature: &str) -> bool {
        use std::collections::HashMap;
        let valid: HashMap<&str, Vec<&str>> = vec![
            ("spec", vec!["proposal", "decision", "spec", "reference"]),
            ("proposal", vec!["decision"]),
            ("decision", vec!["spec", "reference"]),
            ("note", vec!["proposal", "decision", "spec", "reference", "note"]),
            ("reference", vec!["spec", "proposal", "decision"]),
        ]
        .into_iter()
        .collect();

        valid
            .get(upstream_nature)
            .map(|downstreams| downstreams.contains(&downstream_nature))
            .unwrap_or(true)
    }

    async fn find_gaps(&self, references: &[String]) -> Vec<String> {
        let mut gaps = Vec::new();
        for id in references {
            if let Ok(None) = self.db.get_document(id).await {
                gaps.push(id.clone());
            }
        }
        gaps
    }

    // ------------------------------------------------------------------
    // ③ 发散诊断
    // ------------------------------------------------------------------

    fn diagnose_divergences(
        &self,
        doc: &Document,
        pos: &GovPosition,
        graph: &RelationGraph,
    ) -> Vec<Divergence> {
        let mut divs = Vec::new();

        // 未来扩展：pos.role_in_chain 可用于判断发散模式
        let _ = pos;

        // 意图漂移：上游 stage 不可引用
        if let Some(ref upstream) = doc.upstream {
            // 上游不存在
            if graph.gaps.contains(upstream) {
                divs.push(Divergence {
                    div_type: DivergenceType::ReferenceBreak,
                    severity: DivergenceSeverity::Critical,
                    confidence: 0.95,
                    description: format!("上游 '{}' 在索引中不存在", upstream),
                    suggestion: Some("归档上游文档，或更正 upstream 引用".into()),
                });
            }
        }

        // 引用断裂：gaps 中的文档被引用但不存在
        for gap_id in &graph.gaps {
            if Some(gap_id.as_str()) != doc.upstream.as_deref() {
                divs.push(Divergence {
                    div_type: DivergenceType::ReferenceBreak,
                    severity: DivergenceSeverity::Warning,
                    confidence: 0.90,
                    description: format!("引用的文档 '{}' 不存在于索引", gap_id),
                    suggestion: Some("检查引用 id 是否正确，或补充缺失文档".into()),
                });
            }
        }

        // 重复冗余
        for dup in &graph.duplicates {
            if dup.overlap == OverlapDegree::Exact || dup.overlap == OverlapDegree::High {
                divs.push(Divergence {
                    div_type: DivergenceType::Duplication,
                    severity: DivergenceSeverity::Warning,
                    confidence: 0.85,
                    description: dup.description.clone(),
                    suggestion: Some("考虑 merge 或明确区分两者的治理角色".into()),
                });
            }
        }

        // 冲突
        for conflict in &graph.conflicts {
            divs.push(Divergence {
                div_type: DivergenceType::IntentDrift,
                severity: DivergenceSeverity::Critical,
                confidence: 0.80,
                description: format!("与 '{}' 冲突：{} vs {}", conflict.doc_id, conflict.claim, conflict.counter_claim),
                suggestion: Some("review 治理链合法性，修正 upstream 或 nature".into()),
            });
        }

        divs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_references() {
        let icl = ICL {
            db: std::sync::Arc::new(crate::core::database::SqliteBackend::open_in_memory().unwrap()),
        };

        let content = "参考 [240610-1030-on-sihankor-canon] 和 260616-1800-plan-semantic-split-decision 的内容。";
        let refs = icl.extract_references(content);
        assert_eq!(refs.len(), 2);
        assert!(refs.contains(&"240610-1030-on-sihankor-canon".to_string()));
        assert!(refs.contains(&"260616-1800-plan-semantic-split-decision".to_string()));
    }

    #[test]
    fn test_is_legal_chain() {
        assert!(ICL::is_legal_chain("proposal", "decision"));
        assert!(ICL::is_legal_chain("spec", "proposal"));
        assert!(!ICL::is_legal_chain("proposal", "spec"));
        assert!(ICL::is_legal_chain("note", "proposal"));
    }
}
