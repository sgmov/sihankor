use super::types::{
    Action, ActionKind, AffectedDocs, Alternative, Cognition, DecisionProposal, Divergence,
    DivergenceSeverity, DivergenceType, Rationale,
};

/// iWW 消息机 —— 三机第二机
///
/// 从 iCL 认知产出中生成决策建议。
/// 规则驱动（非 LLM）：按发散类型映射到推荐行动，生成多样替代方案。
pub struct IWW;

impl IWW {
    /// 从 cognition 生成 decision_proposal
    pub fn propose(cognition: &Cognition) -> DecisionProposal {
        let divs = &cognition.divergence_diagnosis;

        // 无发散 → no_action
        if divs.is_empty() {
            return DecisionProposal {
                recommended_action: Action {
                    kind: ActionKind::NoAction,
                    description: "未检测到发散。文档治理健康。".into(),
                    revert_steps: "无需回退".into(),
                },
                rationale: Rationale {
                    dao_basis: "道一：发散归向自然收敛".into(),
                    fa_basis: "有度：不加多余的修复建议".into(),
                },
                alternatives: vec![Alternative {
                    action: ActionKind::HumanReview,
                    pros: "人工复核确认无遗漏".into(),
                    cons: "不必要的审查成本".into(),
                }],
                affected_documents: AffectedDocs {
                    direct: vec![],
                    indirect: vec![],
                    before_after: vec![],
                },
            };
        }

        // 收集所有受影响的文档
        let all_refs = &cognition.relation_graph.references;
        let gaps = &cognition.relation_graph.gaps;
        let duplicates = &cognition.relation_graph.duplicates;

        let mut affected_direct: Vec<String> = all_refs.clone();
        // 去重
        affected_direct.sort();
        affected_direct.dedup();

        // 判断主导发散类型来决定推荐行动
        let has_critical = divs
            .iter()
            .any(|d| d.severity == DivergenceSeverity::Critical);
        let has_gap = divs
            .iter()
            .any(|d| d.div_type == DivergenceType::ReferenceBreak);
        let has_dup = divs
            .iter()
            .any(|d| d.div_type == DivergenceType::Duplication);

        let (action_kind, desc) = if has_critical && has_gap {
            (
                ActionKind::HumanReview,
                format!(
                    "存在引用断裂（缺失 {} 个文档）和严重发散，需人工确定修复方向。gaps: {:?}",
                    gaps.len(),
                    gaps
                ),
            )
        } else if has_dup {
            (
                ActionKind::Merge,
                format!(
                    "检测到 {} 个高重叠文档，建议合并。重叠文档：{:?}",
                    duplicates.len(),
                    duplicates
                        .iter()
                        .map(|d| d.doc_id.clone())
                        .collect::<Vec<_>>()
                ),
            )
        } else if has_critical {
            (
                ActionKind::HumanReview,
                format!(
                    "{} 个临界级发散需人工决策",
                    divs.iter()
                        .filter(|d| d.severity == DivergenceSeverity::Critical)
                        .count()
                ),
            )
        } else {
            (
                ActionKind::NoAction,
                format!("{} 个 info/warning 级发散，建议观望", divs.len()),
            )
        };

        let recommended_action = Action {
            kind: action_kind.clone(),
            description: desc,
            revert_steps: match action_kind {
                ActionKind::Merge => "merge 前备份所有涉及文档".into(),
                ActionKind::HumanReview => "无需回退（人工决策尚未执行）".into(),
                ActionKind::NoAction => "无需回退".into(),
                _ => "git revert".into(),
            },
        };

        // 生成 2-3 个替代方案
        let alternatives = Self::generate_alternatives(&action_kind, cognition);

        DecisionProposal {
            recommended_action,
            rationale: Rationale {
                dao_basis: Self::dao_basis_for(divs),
                fa_basis: Self::fa_basis_for(divs),
            },
            alternatives,
            affected_documents: AffectedDocs {
                direct: affected_direct,
                indirect: vec![],
                before_after: vec![],
            },
        }
    }

    fn generate_alternatives(recommended: &ActionKind, cognition: &Cognition) -> Vec<Alternative> {
        let mut alts = Vec::new();
        let divs = &cognition.divergence_diagnosis;

        match recommended {
            ActionKind::HumanReview => {
                alts.push(Alternative {
                    action: ActionKind::NoAction,
                    pros: "零修改成本，保持现状".into(),
                    cons: "发散可能恶化".into(),
                });
                alts.push(Alternative {
                    action: ActionKind::Archive,
                    pros: "归档出问题的文档以阻断传播".into(),
                    cons: "文档丧失效力".into(),
                });
            }
            ActionKind::Merge => {
                alts.push(Alternative {
                    action: ActionKind::NoAction,
                    pros: "保持多视角并存的丰富性".into(),
                    cons: "重复维护成本".into(),
                });
                alts.push(Alternative {
                    action: ActionKind::Archive,
                    pros: "归档旧版，保留最新版".into(),
                    cons: "丧失历史多视角".into(),
                });
                if divs
                    .iter()
                    .any(|d| d.div_type == DivergenceType::IntentDrift)
                {
                    alts.push(Alternative {
                        action: ActionKind::HumanReview,
                        pros: "让人类判断哪个版本是正确的意图".into(),
                        cons: "拖慢修复周期".into(),
                    });
                }
            }
            ActionKind::NoAction => {
                alts.push(Alternative {
                    action: ActionKind::HumanReview,
                    pros: "确认无遗漏".into(),
                    cons: "不必要的审查成本".into(),
                });
            }
            _ => {
                alts.push(Alternative {
                    action: ActionKind::HumanReview,
                    pros: "由人类最终决定".into(),
                    cons: "额外时间成本".into(),
                });
            }
        }

        alts
    }

    fn dao_basis_for(divs: &[Divergence]) -> String {
        let types: Vec<&str> = divs
            .iter()
            .map(|d| match d.div_type {
                DivergenceType::IntentDrift => "道二（意图先于代码）",
                DivergenceType::ReferenceBreak => "道三（代码自晦，意图必复）",
                DivergenceType::Duplication => "道一（发散自-然，收敛必-为）",
                DivergenceType::Gap => "道三（跨文档关系不可见）",
                DivergenceType::BenignDivergence => "道一（自然发散无需强制收敛）",
            })
            .collect();
        let mut unique: Vec<&str> = types.into_iter().collect();
        unique.sort();
        unique.dedup();
        unique.join(" + ")
    }

    fn fa_basis_for(divs: &[Divergence]) -> String {
        let has_critical = divs
            .iter()
            .any(|d| d.severity == DivergenceSeverity::Critical);
        let has_dup = divs
            .iter()
            .any(|d| d.div_type == DivergenceType::Duplication);

        if has_critical {
            "损补（损有余补不足）".into()
        } else if has_dup {
            "有度（收敛恰到好处）+ 损补".into()
        } else {
            "有度（不过度反应）".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::types::{ChainRole, GovPosition, RelationGraph};

    fn make_test_cognition(divs: Vec<Divergence>) -> Cognition {
        Cognition {
            governance_position: GovPosition {
                nature: "spec".into(),
                stage: "2/3".into(),
                upstream_chain: vec!["root-id".into()],
                role_in_chain: ChainRole::Derive,
            },
            relation_graph: RelationGraph {
                references: vec!["260616-1200-sihankor-dev-governance".into()],
                duplicates: vec![],
                conflicts: vec![],
                gaps: vec![],
            },
            divergence_diagnosis: divs,
        }
    }

    #[test]
    fn test_no_divergence_no_action() {
        let cog = make_test_cognition(vec![]);
        let proposal = IWW::propose(&cog);
        assert_eq!(proposal.recommended_action.kind, ActionKind::NoAction);
        assert!(!proposal.alternatives.is_empty());
    }

    #[test]
    fn test_reference_break_human_review() {
        let cog = make_test_cognition(vec![Divergence {
            div_type: DivergenceType::ReferenceBreak,
            severity: DivergenceSeverity::Critical,
            confidence: 0.95,
            description: "missing upstream".into(),
            suggestion: Some("fix upstream".into()),
        }]);
        let proposal = IWW::propose(&cog);
        assert_eq!(proposal.recommended_action.kind, ActionKind::HumanReview);
        assert!(proposal.alternatives.len() >= 2);
    }

    #[test]
    fn test_duplication_merge() {
        let cog = make_test_cognition(vec![Divergence {
            div_type: DivergenceType::Duplication,
            severity: DivergenceSeverity::Warning,
            confidence: 0.85,
            description: "two docs say same thing".into(),
            suggestion: Some("merge".into()),
        }]);
        let proposal = IWW::propose(&cog);
        assert_eq!(proposal.recommended_action.kind, ActionKind::Merge);
        // merge 应有至少 3 个替代方案
        assert!(proposal.alternatives.len() >= 2);
    }
}
