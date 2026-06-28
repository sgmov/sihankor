use super::types::{
    Action, ActionKind, ChainRole, Cognition, DecisionProposal, Divergence,
    DivergenceSeverity, DivergenceType, GovPosition, LawCheck, LawCheckResult,
    LawViolationSummary, OverlapDegree, TrailContext, Verdict, Verification,
};

/// iCT 方圆机 —— 三机第三机
///
/// 对 decision_proposal 执行五法检验（顺因/有度/知止/损补/顺势），
/// 产出 Verification（逐条结果 + overall verdict + law_violation_summary）。
pub struct ICT;

impl ICT {
    /// 对单个 proposal 执行全五法检验
    pub fn verify(cognition: &Cognition, proposal: &DecisionProposal) -> Verification {
        let checks = vec![
            Self::check_shunyin(cognition, proposal),
            Self::check_youdou(cognition, proposal),
            Self::check_zhizhi(cognition, proposal),
            Self::check_sunbu(cognition, proposal),
            Self::check_shunshi(cognition, proposal),
        ];

        let overall = Self::overall_verdict(&checks);
        let law_violation_summary = Self::build_law_violation_summary(&checks, cognition, proposal);

        Verification {
            five_law_check: checks,
            overall,
            law_violation_summary,
        }
    }

    // ------------------------------------------------------------------
    // 顺因
    // ------------------------------------------------------------------

    fn check_shunyin(cognition: &Cognition, proposal: &DecisionProposal) -> LawCheck {
        let action = &proposal.recommended_action;
        let pos = &cognition.governance_position;

        // R1: 逆写检查
        if Self::modifies_upstream(action, &pos.upstream_chain)
            && action.kind != ActionKind::HumanReview
        {
            return LawCheck {
                law: "顺因".into(),
                result: LawCheckResult::Fail,
                note: format!(
                    "逆写上游：action 试图修改 upstream_chain 中的文档 {:?}，且非 HumanReview 模式",
                    pos.upstream_chain
                ),
            };
        }

        // R2: 越级检查
        if pos.stage == "1/3" && Self::targets_ratify_level(action) {
            return LawCheck {
                law: "顺因".into(),
                result: LawCheckResult::Fail,
                note: "越级操作：对 stage 1/3 文档执行 ratify 级操作".into(),
            };
        }
        if pos.nature == "proposal" && Self::targets_spec_verification(action) {
            return LawCheck {
                law: "顺因".into(),
                result: LawCheckResult::Fail,
                note: "越级操作：对 proposal 文档执行 spec 级验证要求".into(),
            };
        }

        // R3: 引用方向 —— no upstream change in current action model, skip for MVP
        // (future: check action.new_upstream vs nature chain legality)

        // Boundary: HumanReview + upstream modification
        if action.kind == ActionKind::HumanReview
            && Self::modifies_upstream(action, &pos.upstream_chain)
        {
            return LawCheck {
                law: "顺因".into(),
                result: LawCheckResult::Conditional,
                note: "HumanReview 模式下的上游修改需人工判断".into(),
            };
        }

        LawCheck {
            law: "顺因".into(),
            result: LawCheckResult::Pass,
            note: "因果方向合规".into(),
        }
    }

    /// 判断 action 是否修改上游文档
    fn modifies_upstream(action: &Action, upstream_chain: &[String]) -> bool {
        if upstream_chain.is_empty() {
            return false;
        }
        let desc = &action.description.to_lowercase();
        upstream_chain
            .iter()
            .any(|up| desc.contains(&up.to_lowercase()))
    }

    /// 判断 action 是否达到 ratify 级操作力度
    const fn targets_ratify_level(action: &Action) -> bool {
        // ratify 级操作：Archive（对低 stage 文档）, Merge 到上游
        matches!(action.kind, ActionKind::Archive | ActionKind::Merge)
    }

    /// 判断 action 是否对 proposal 文档施加 spec 级要求
    const fn targets_spec_verification(action: &Action) -> bool {
        // spec 级操作：重命名（改名是 spec 的 formalization 操作）
        matches!(action.kind, ActionKind::Rename)
    }

    // ------------------------------------------------------------------
    // 有度
    // ------------------------------------------------------------------

    fn check_youdou(cognition: &Cognition, proposal: &DecisionProposal) -> LawCheck {
        let action = &proposal.recommended_action;
        let divs = &cognition.divergence_diagnosis;

        let max_severity = max_divergence_severity(divs);
        let has_low_confidence = divs.iter().any(|d| d.confidence < 0.7);
        let has_mixed = has_mixed_severity(divs);

        // R1: 力度不足
        if max_severity == Some(DivergenceSeverity::Critical) && action.kind == ActionKind::NoAction
        {
            return LawCheck {
                law: "有度".into(),
                result: LawCheckResult::Fail,
                note: "力度不足：Critical 级发散不应被 NoAction 忽视".into(),
            };
        }

        // R2: 力度过度
        if max_severity == Some(DivergenceSeverity::Info)
            && matches!(action.kind, ActionKind::Archive)
        {
            return LawCheck {
                law: "有度".into(),
                result: LawCheckResult::Fail,
                note: "力度过度：Info 级发散不应触发 Archive 级操作".into(),
            };
        }

        // Boundary: low confidence Warning
        if max_severity == Some(DivergenceSeverity::Warning) && has_low_confidence {
            return LawCheck {
                law: "有度".into(),
                result: LawCheckResult::Conditional,
                note: "低 confidence Warning，力度判定存在不确定性".into(),
            };
        }

        if has_mixed {
            return LawCheck {
                law: "有度".into(),
                result: LawCheckResult::Conditional,
                note: "混合严重度发散，单一 action 力度可能不完全精确".into(),
            };
        }

        LawCheck {
            law: "有度".into(),
            result: LawCheckResult::Pass,
            note: "力度匹配合理".into(),
        }
    }

    // ------------------------------------------------------------------
    // 知止
    // ------------------------------------------------------------------

    fn check_zhizhi(cognition: &Cognition, proposal: &DecisionProposal) -> LawCheck {
        let action = &proposal.recommended_action;
        let pos = &cognition.governance_position;

        // R1: 不判断哲学对错
        if contains_philosophical_negation(&action.description) && is_philosophy_doc(pos) {
            return LawCheck {
                law: "知止".into(),
                result: LawCheckResult::Fail,
                note: "越权判断：不可对哲学文档做出否定性判断".into(),
            };
        }

        // R2: 不修改非 .sih.md 文件（仅检查实际文件路径，跳过文档 ID）
        for path in &proposal.affected_documents.direct {
            // Document IDs (YYMMDD format) are governance references, not file modifications
            let is_doc_id = path.len() >= 10 && path.chars().take(6).all(|c| c.is_ascii_digit());
            if !is_doc_id && !path.ends_with(".sih.md") && !path.starts_with(".sih/") {
                return LawCheck {
                    law: "知止".into(),
                    result: LawCheckResult::Fail,
                    note: format!("越权操作：不可直接修改非治约文件 '{}'", path),
                };
            }
        }

        // R3: 不管理 drafts/ 文档
        if targets_drafts(pos) {
            return LawCheck {
                law: "知止".into(),
                result: LawCheckResult::Fail,
                note: "越权操作：drafts/ 中的文档不可纳入治理链".into(),
            };
        }

        // Boundary: indirect 影响包含非治约文件（仅检查实际文件路径）
        for path in &proposal.affected_documents.indirect {
            let is_doc_id = path.len() >= 10 && path.chars().take(6).all(|c| c.is_ascii_digit());
            if !is_doc_id && !path.ends_with(".sih.md") && !path.starts_with(".sih/") {
                return LawCheck {
                    law: "知止".into(),
                    result: LawCheckResult::Conditional,
                    note: format!("间接影响包含非治约文件 '{}'，需人类确认影响范围", path),
                };
            }
        }

        LawCheck {
            law: "知止".into(),
            result: LawCheckResult::Pass,
            note: "未逾越 Mind 边界".into(),
        }
    }

    // ------------------------------------------------------------------
    // 损补
    // ------------------------------------------------------------------

    fn check_sunbu(cognition: &Cognition, proposal: &DecisionProposal) -> LawCheck {
        let action = &proposal.recommended_action;
        let divs = &cognition.divergence_diagnosis;
        let graph = &cognition.relation_graph;

        let has_high_dups = graph
            .duplicates
            .iter()
            .any(|d| matches!(d.overlap, OverlapDegree::Exact | OverlapDegree::High));

        let has_gap_div = divs.iter().any(|d| {
            d.div_type == DivergenceType::ReferenceBreak || d.div_type == DivergenceType::Gap
        });

        let has_partial_dups = graph
            .duplicates
            .iter()
            .any(|d| d.overlap == OverlapDegree::Partial);

        // R1: 方向反置
        if has_high_dups && action.kind == ActionKind::NoAction {
            return LawCheck {
                law: "损补".into(),
                result: LawCheckResult::Fail,
                note: "方向反置：存在高重叠重复但建议 NoAction（该损不损）".into(),
            };
        }

        if has_gap_div && action.kind == ActionKind::Archive {
            return LawCheck {
                law: "损补".into(),
                result: LawCheckResult::Fail,
                note: "方向反置：存在 Gap 发散但建议 Archive（该补却损）".into(),
            };
        }

        // R2: 损补冲突
        if has_conflicting_merge_archive(proposal) {
            return LawCheck {
                law: "损补".into(),
                result: LawCheckResult::Fail,
                note: "损补冲突：对同一文档同时建议 merge 和 archive".into(),
            };
        }

        // Boundary: Partial overlap
        if has_partial_dups && action.kind == ActionKind::Merge {
            return LawCheck {
                law: "损补".into(),
                result: LawCheckResult::Conditional,
                note: "Partial 重叠的重复，merge 可能过度".into(),
            };
        }

        LawCheck {
            law: "损补".into(),
            result: LawCheckResult::Pass,
            note: "损补方向正确".into(),
        }
    }

    // ------------------------------------------------------------------
    // 顺势 (G5 Trend Alignment)
    //
    // TODO: 变更率度量采集
    // 顺势之法要求追踪"规则审查频率 vs 代码变更频率的比值"。
    // 此处应查询 metrics 表中最近的 ValidationCompleted 记录数（作为代码
    // 变更频率的代理），构造一条度量事件记录"审查次数/变更次数"比值，
    // 并调用 db.record_metric。采集失败不应阻断检验流程。
    //
    // 当前限制：verify 是纯同步关联函数，签名中无 db 参数，无法执行
    // 异步数据库操作。待后续重构（将 db 注入 verify 或在调用方采集）
    // 时补全。
    // ------------------------------------------------------------------

    fn check_shunshi(cognition: &Cognition, proposal: &DecisionProposal) -> LawCheck {
        let action = &proposal.recommended_action;
        let pos = &cognition.governance_position;
        let stage = &pos.stage;

        // R1: 措辞匹配
        if stage == "3/3" && contains_weak_hedging(&action.description) {
            return LawCheck {
                law: "顺势".into(),
                result: LawCheckResult::Fail,
                note: "措辞失当：ratify 文档应使用确定性措辞，不应使用'可能'等暧昧表达".into(),
            };
        }
        if stage == "1/3" && contains_mandatory_language(&action.description) {
            return LawCheck {
                law: "顺势".into(),
                result: LawCheckResult::Fail,
                note: "措辞失当：propose 文档不应使用'必须'等强制措辞".into(),
            };
        }

        // R2: root 保护
        if pos.role_in_chain == ChainRole::Root && action.kind == ActionKind::Merge {
            return LawCheck {
                law: "顺势".into(),
                result: LawCheckResult::Fail,
                note: "root 保护：不应将 root 文档 merge 到其他文档".into(),
            };
        }

        // R3: 环检测 —— check if upstream change would create cycle
        if (action.kind == ActionKind::Move || action.kind == ActionKind::Merge)
            && would_create_cycle(action, &pos.upstream_chain)
        {
            return LawCheck {
                law: "顺势".into(),
                result: LawCheckResult::Fail,
                note: "引用环：upstream 变更将形成循环依赖".into(),
            };
        }

        // R4: archive 文档
        if stage == "X"
            && action.kind != ActionKind::NoAction
            && action.kind != ActionKind::HumanReview
        {
            return LawCheck {
                law: "顺势".into(),
                result: LawCheckResult::Fail,
                note: "不应修改已归档（stage X）文档".into(),
            };
        }

        // Boundary: 2/3 transitional
        if stage == "2/3" {
            return LawCheck {
                law: "顺势".into(),
                result: LawCheckResult::Conditional,
                note: "2/3 阶段措辞力度存在弹性空间".into(),
            };
        }

        LawCheck {
            law: "顺势".into(),
            result: LawCheckResult::Pass,
            note: "力度适配合理".into(),
        }
    }

    // ------------------------------------------------------------------
    // overall
    // ------------------------------------------------------------------

    fn overall_verdict(checks: &[LawCheck]) -> Verdict {
        if checks.iter().any(|c| c.result == LawCheckResult::Fail) {
            return Verdict::Fail;
        }
        if checks
            .iter()
            .any(|c| c.result == LawCheckResult::Conditional)
        {
            return Verdict::Conditional;
        }
        Verdict::Pass
    }

    fn build_law_violation_summary(
        checks: &[LawCheck],
        cognition: &Cognition,
        proposal: &DecisionProposal,
    ) -> Vec<LawViolationSummary> {
        let mut summaries = Vec::new();

        for c in checks {
            if c.result == LawCheckResult::Pass {
                continue;
            }
            let (laws, detail) = match c.law.as_str() {
                "顺因" => (
                    "顺因",
                    format!(
                        "逆因果链：action '{}' 可能修改上游文档",
                        proposal.recommended_action.description
                    ),
                ),
                "有度" => (
                    "有度",
                    "力度失配：action severity vs 发散 severity".to_string(),
                ),
                "知止" => (
                    "知止",
                    format!(
                        "逾矩：action '{}' 超出治理边界",
                        proposal.recommended_action.description
                    ),
                ),
                "损补" => (
                    "损补",
                    format!(
                        "方向错误：action '{}' 的损补方向可能不正确",
                        proposal.recommended_action.description
                    ),
                ),
                "顺势" => (
                    "顺势",
                    format!(
                        "力度失时：action '{}' 在 stage {} 力度不匹配",
                        proposal.recommended_action.description,
                        cognition.governance_position.stage
                    ),
                ),
                _ => ("未分类", "未分类的合道偏差".into()),
            };
            summaries.push(LawViolationSummary {
                laws: laws.into(),
                detail,
            });
        }

        summaries
    }
}

// ---------------------------------------------------------------------------
// Helper functions (shared logic, pure)
// ---------------------------------------------------------------------------

fn max_divergence_severity(divs: &[Divergence]) -> Option<DivergenceSeverity> {
    divs.iter().fold(None, |acc, d| match (&acc, &d.severity) {
        (None, s) => Some(s.clone()),
        (Some(DivergenceSeverity::Critical), _) => acc,
        (_, DivergenceSeverity::Critical) => Some(DivergenceSeverity::Critical),
        (Some(DivergenceSeverity::Warning), _) => acc,
        (_, DivergenceSeverity::Warning) => Some(DivergenceSeverity::Warning),
        _ => acc,
    })
}

fn has_mixed_severity(divs: &[Divergence]) -> bool {
    let has_critical = divs
        .iter()
        .any(|d| d.severity == DivergenceSeverity::Critical);
    let has_info = divs.iter().any(|d| d.severity == DivergenceSeverity::Info);
    has_critical && has_info
}

fn contains_philosophical_negation(desc: &str) -> bool {
    let neg_markers = [
        "是错误的",
        "不正确",
        "有问题",
        "矛盾",
        "不成立",
        "重新定义",
        "推翻了",
    ];
    neg_markers.iter().any(|m| desc.contains(m))
}

fn is_philosophy_doc(pos: &GovPosition) -> bool {
    pos.nature == "spec"
        && pos
            .upstream_chain
            .iter()
            .any(|u| u.contains("canon") || u.contains("tao") || u.contains("arche"))
}

const fn targets_drafts(_pos: &GovPosition) -> bool {
    // 当前 iCT 无法从 nature 直接判断是否在 drafts/ —— 由调用方传入上下文
    // MVP 实现：不做 drafts 检查（nature 系统无 "draft" 值），返回 false
    false
}

fn has_conflicting_merge_archive(proposal: &DecisionProposal) -> bool {
    let main_is_merge = proposal.recommended_action.kind == ActionKind::Merge;
    let main_is_archive = proposal.recommended_action.kind == ActionKind::Archive;
    let alt_has_opposite = proposal.alternatives.iter().any(|a| {
        (main_is_merge && a.action == ActionKind::Archive)
            || (main_is_archive && a.action == ActionKind::Merge)
    });
    (main_is_archive || main_is_merge) && alt_has_opposite
}

fn contains_weak_hedging(desc: &str) -> bool {
    let hedge_words = ["可能", "或许", "大概", "考虑", "也许", "似乎", "倾向于"];
    hedge_words.iter().any(|w| desc.contains(w))
}

fn contains_mandatory_language(desc: &str) -> bool {
    let mandatory_words = ["必须", "务必", "一定", "强制", "不可不", "绝不能"];
    mandatory_words.iter().any(|w| desc.contains(w))
}

fn would_create_cycle(action: &Action, upstream_chain: &[String]) -> bool {
    // 检查 action 描述中是否引用了上游链中的文档 ID，若引用则形成环
    // 注意：此检查依赖 ID 在描述中被引用，未来可实现 resolve_chain DB 查询做精确检测
    if upstream_chain.is_empty() {
        return false;
    }
    upstream_chain
        .iter()
        .any(|upstream_id| action.description.contains(upstream_id.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mind::types::{AffectedDocs, ChainRole, GovPosition, Rationale, RelationGraph};

    fn make_test_context(
        stage: &str,
        nature: &str,
        role: ChainRole,
        divs: Vec<Divergence>,
        action_kind: ActionKind,
        action_desc: &str,
    ) -> (Cognition, DecisionProposal) {
        let cognition = Cognition {
            governance_position: GovPosition {
                nature: nature.into(),
                stage: stage.into(),
                upstream_chain: vec!["upstream-doc-id".into()],
                role_in_chain: role,
            },
            relation_graph: RelationGraph {
                references: vec![],
                duplicates: vec![],
                conflicts: vec![],
                gaps: vec![],
            },
            divergence_diagnosis: divs,
            trail_context: TrailContext {
                trails: vec![],
                trail_count: 0,
                has_open_trails: false,
            },
        };

        let proposal = DecisionProposal {
            recommended_action: Action {
                kind: action_kind,
                description: action_desc.into(),
                revert_steps: "git revert".into(),
            },
            rationale: Rationale {
                dao_basis: "道一".into(),
                fa_basis: "有度".into(),
            },
            alternatives: vec![],
            affected_documents: AffectedDocs {
                direct: vec![],
                indirect: vec![],
                before_after: vec![],
            },
        };

        (cognition, proposal)
    }

    #[test]
    fn test_all_pass() {
        let (cog, prop) = make_test_context(
            "2/3",
            "spec",
            ChainRole::Derive,
            vec![],
            ActionKind::NoAction,
            "文档治理健康，无需操作",
        );
        let v = ICT::verify(&cog, &prop);
        assert_eq!(v.overall, Verdict::Conditional); // 2/3 → Conditional per 顺势
        assert_eq!(v.five_law_check.len(), 5);
    }

    #[test]
    fn test_shunyin_fail_skip_stage() {
        let (cog, prop) = make_test_context(
            "1/3",
            "proposal",
            ChainRole::Derive,
            vec![],
            ActionKind::Archive,
            "归档此文档",
        );
        let v = ICT::verify(&cog, &prop);
        let shunyin = v.five_law_check.iter().find(|c| c.law == "顺因").unwrap();
        assert_eq!(shunyin.result, LawCheckResult::Fail);
        assert_eq!(v.overall, Verdict::Fail);
    }

    #[test]
    fn test_youdou_fail_under_reaction() {
        let (cog, prop) = make_test_context(
            "2/3",
            "spec",
            ChainRole::Derive,
            vec![Divergence {
                div_type: DivergenceType::IntentDrift,
                severity: DivergenceSeverity::Critical,
                confidence: 0.90,
                description: "严重意图漂移".into(),
                suggestion: Some("HumanReview".into()),
            }],
            ActionKind::NoAction,
            "不做任何处理",
        );
        let v = ICT::verify(&cog, &prop);
        let youdou = v.five_law_check.iter().find(|c| c.law == "有度").unwrap();
        assert_eq!(youdou.result, LawCheckResult::Fail);
    }

    #[test]
    fn test_zhizhi_fail_modify_non_sihmd() {
        let (cog, prop) = make_test_context(
            "2/3",
            "spec",
            ChainRole::Derive,
            vec![],
            ActionKind::Merge,
            "合并文档",
        );
        let prop_with_bad_file = DecisionProposal {
            affected_documents: AffectedDocs {
                direct: vec!["src/main.rs".into()],
                indirect: vec![],
                before_after: vec![],
            },
            ..prop
        };
        let v = ICT::verify(&cog, &prop_with_bad_file);
        let zhizhi = v.five_law_check.iter().find(|c| c.law == "知止").unwrap();
        assert_eq!(zhizhi.result, LawCheckResult::Fail);
    }

    #[test]
    fn test_weak_hedging_detection() {
        assert!(contains_weak_hedging("可能需要进行调整"));
        assert!(!contains_weak_hedging("应修改上游引用"));
    }

    #[test]
    fn test_mandatory_detection() {
        assert!(contains_mandatory_language("必须修改此字段"));
        assert!(!contains_mandatory_language("建议修改此字段"));
    }

    #[test]
    fn test_all_pass_on_clean_proposal() {
        let (cog, prop) = make_test_context(
            "3/3",
            "spec",
            ChainRole::Derive,
            vec![],
            ActionKind::NoAction,
            "文档状态正常，无需变更",
        );
        let v = ICT::verify(&cog, &prop);
        assert_eq!(v.overall, Verdict::Pass);
    }
}
