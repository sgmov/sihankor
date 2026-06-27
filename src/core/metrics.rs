use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::database::{DatabaseError, SihDatabase};
use crate::core::validator::RULE_REGISTRY;

/// 度量事件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricEvent {
    /// 文档验证完成
    ValidationCompleted {
        doc_id: String,
        nature: String,
        stage: String,
        fatal_count: usize,
        guideline_count: usize,
        judgment_count: usize,
        passed: bool,
    },
    /// 文档索引完成
    IndexCompleted { doc_id: String, nature: String },
    /// Stage 转换
    StageTransition {
        doc_id: String,
        from_stage: String,
        to_stage: String,
    },
    /// 项目快照(规则数、文档数等聚合)
    ProjectSnapshot {
        total_docs: usize,
        total_rules: usize,
        docs_by_stage: Vec<(String, usize)>,
        docs_by_nature: Vec<(String, usize)>,
        fatal_violations_total: usize,
    },
}

/// 存储在数据库中的度量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecord {
    pub id: i64,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: String,
}

/// 产出方差指标结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceMetric {
    /// 统计窗口内的文档总数
    pub total_docs: usize,
    /// 通过验证的文档比例 (0.0-1.0)
    pub pass_rate: f64,
    /// 平均 Fatal 违规数
    pub avg_fatal_count: f64,
    /// 平均 Guideline 违规数
    pub avg_guideline_count: f64,
    /// Fatal 违规数的标准差（产出方差的直接度量）
    pub fatal_count_stddev: f64,
    /// 按 nature 分组的通过率
    pub pass_rate_by_nature: Vec<(String, f64)>,
    /// 按 nature 分组的平均 Fatal 违规数
    pub avg_fatal_by_nature: Vec<(String, f64)>,
    /// 统计窗口起始时间
    pub window_start: String,
    /// 统计窗口结束时间
    pub window_end: String,
}

/// ValidationCompleted 事件的 payload 反序列化结构
#[allow(dead_code)]
#[derive(Deserialize)]
struct ValidationCompletedPayload {
    doc_id: String,
    nature: String,
    stage: String,
    fatal_count: usize,
    guideline_count: usize,
    judgment_count: usize,
    passed: bool,
}

/// 从 ValidationCompleted 历史记录计算产出方差指标
/// 输入：MetricRecord 列表（event_type 应为 "ValidationCompleted"）
/// 输出：聚合后的方差指标
pub fn compute_variance_metric(records: &[MetricRecord]) -> VarianceMetric {
    // 反序列化有效记录，失败则跳过
    let payloads: Vec<ValidationCompletedPayload> = records
        .iter()
        .filter_map(|r| serde_json::from_str::<ValidationCompletedPayload>(&r.payload_json).ok())
        .collect();

    // 空记录处理：返回全零指标
    if payloads.is_empty() {
        return VarianceMetric {
            total_docs: 0,
            pass_rate: 0.0,
            avg_fatal_count: 0.0,
            avg_guideline_count: 0.0,
            fatal_count_stddev: 0.0,
            pass_rate_by_nature: Vec::new(),
            avg_fatal_by_nature: Vec::new(),
            window_start: String::new(),
            window_end: String::new(),
        };
    }

    let total = payloads.len();
    let passed_count = payloads.iter().filter(|p| p.passed).count();
    let sum_fatal: usize = payloads.iter().map(|p| p.fatal_count).sum();
    let sum_guideline: usize = payloads.iter().map(|p| p.guideline_count).sum();

    let pass_rate = passed_count as f64 / total as f64;
    let avg_fatal_count = sum_fatal as f64 / total as f64;
    let avg_guideline_count = sum_guideline as f64 / total as f64;

    // 总体标准差（除以 N），描述性统计
    let variance: f64 = payloads
        .iter()
        .map(|p| {
            let diff = p.fatal_count as f64 - avg_fatal_count;
            diff * diff
        })
        .sum::<f64>()
        / total as f64;
    let fatal_count_stddev = variance.sqrt();

    // 按 nature 分组聚合：(总数, 通过数, Fatal 违规总和)
    let mut by_nature: HashMap<String, (usize, usize, usize)> = HashMap::new();
    for p in &payloads {
        let entry = by_nature.entry(p.nature.clone()).or_insert((0, 0, 0));
        entry.0 += 1;
        if p.passed {
            entry.1 += 1;
        }
        entry.2 += p.fatal_count;
    }

    let mut pass_rate_by_nature: Vec<(String, f64)> = by_nature
        .iter()
        .map(|(k, (cnt, passed, _))| (k.clone(), *passed as f64 / *cnt as f64))
        .collect();
    pass_rate_by_nature.sort_by(|a, b| a.0.cmp(&b.0));

    let mut avg_fatal_by_nature: Vec<(String, f64)> = by_nature
        .iter()
        .map(|(k, (cnt, _, sum_f))| (k.clone(), *sum_f as f64 / *cnt as f64))
        .collect();
    avg_fatal_by_nature.sort_by(|a, b| a.0.cmp(&b.0));

    // 统计窗口：取所有记录 created_at 的最小/最大值
    let window_start = records
        .iter()
        .map(|r| r.created_at.as_str())
        .min()
        .unwrap_or("")
        .to_string();
    let window_end = records
        .iter()
        .map(|r| r.created_at.as_str())
        .max()
        .unwrap_or("")
        .to_string();

    VarianceMetric {
        total_docs: total,
        pass_rate,
        avg_fatal_count,
        avg_guideline_count,
        fatal_count_stddev,
        pass_rate_by_nature,
        avg_fatal_by_nature,
        window_start,
        window_end,
    }
}

/// 跨版本快照差异
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    /// 前一次快照时间
    pub previous_time: String,
    /// 后一次快照时间
    pub current_time: String,
    /// 文档数变化（当前 - 前次）
    pub docs_delta: i64,
    /// 规则数变化（当前 - 前次）
    pub rules_delta: i64,
    /// 各 stage 文档数变化
    pub docs_by_stage_delta: Vec<(String, i64)>,
    /// 各 nature 文档数变化
    pub docs_by_nature_delta: Vec<(String, i64)>,
    /// Fatal 违规总数变化
    pub fatal_violations_delta: i64,
    /// 间隙信号：规则数是否增长（true = 增长，支持道四b）
    pub rules_grew: bool,
    /// 间隙信号：文档数是否增长（true = 增长，可能扩大治理间隙）
    pub docs_grew: bool,
}

/// ProjectSnapshot 事件的 payload 反序列化结构
#[derive(Deserialize)]
struct ProjectSnapshotPayload {
    total_docs: usize,
    total_rules: usize,
    docs_by_stage: Vec<(String, usize)>,
    docs_by_nature: Vec<(String, usize)>,
    fatal_violations_total: usize,
}

/// 将两组 (key, count) 合并为按 key 排序的差值列表（current - previous）
/// 不存在的 key 视为 0，使用 HashMap 聚合后排序确保结果确定性
fn compute_kv_delta(
    previous: &[(String, usize)],
    current: &[(String, usize)],
) -> Vec<(String, i64)> {
    let mut map: HashMap<String, (i64, i64)> = HashMap::new();
    for (k, v) in previous {
        map.entry(k.clone()).or_insert((0, 0)).0 += *v as i64;
    }
    for (k, v) in current {
        map.entry(k.clone()).or_insert((0, 0)).1 += *v as i64;
    }
    let mut result: Vec<(String, i64)> = map
        .into_iter()
        .map(|(k, (prev, curr))| (k, curr - prev))
        .collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

/// 比较两次 ProjectSnapshot 的差异
/// 输入：两条 MetricRecord（event_type 应为 "ProjectSnapshot"）
/// 输出：差异结果
/// 如果任一记录的 payload_json 无法反序列化，返回 None
pub fn compute_snapshot_diff(
    previous: &MetricRecord,
    current: &MetricRecord,
) -> Option<SnapshotDiff> {
    let prev: ProjectSnapshotPayload =
        serde_json::from_str(&previous.payload_json).ok()?;
    let curr: ProjectSnapshotPayload =
        serde_json::from_str(&current.payload_json).ok()?;

    let docs_delta = curr.total_docs as i64 - prev.total_docs as i64;
    let rules_delta = curr.total_rules as i64 - prev.total_rules as i64;
    let fatal_violations_delta =
        curr.fatal_violations_total as i64 - prev.fatal_violations_total as i64;

    let docs_by_stage_delta = compute_kv_delta(&prev.docs_by_stage, &curr.docs_by_stage);
    let docs_by_nature_delta = compute_kv_delta(&prev.docs_by_nature, &curr.docs_by_nature);

    Some(SnapshotDiff {
        previous_time: previous.created_at.clone(),
        current_time: current.created_at.clone(),
        docs_delta,
        rules_delta,
        docs_by_stage_delta,
        docs_by_nature_delta,
        fatal_violations_delta,
        rules_grew: rules_delta > 0,
        docs_grew: docs_delta > 0,
    })
}

/// 从数据库查询最近的两次快照并计算差异
/// 需要传入 SihDatabase 引用
/// 如果不足两条快照，返回 Ok(None)
pub async fn compute_latest_snapshot_diff(
    db: &dyn SihDatabase,
) -> Result<Option<SnapshotDiff>, DatabaseError> {
    let records = db.query_metrics("ProjectSnapshot", 2).await?;
    if records.len() < 2 {
        return Ok(None);
    }
    // query_metrics 按 created_at DESC 排序：records[0] 最新，records[1] 前一次
    let diff = compute_snapshot_diff(&records[1], &records[0]);
    Ok(diff)
}

/// 规则审计指标（有度/G3：规则数分布与 Fatal 占比）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAuditMetric {
    pub total_rules: usize,
    pub rules_by_domain: Vec<(String, usize)>,
    pub rules_by_severity: Vec<(String, usize)>,
    pub fatal_ratio: f64,
}

/// 从 RULE_REGISTRY 聚合规则审计指标（纯函数，不查数据库）
pub fn compute_rule_audit() -> RuleAuditMetric {
    use std::collections::HashMap;

    let total_rules = RULE_REGISTRY.len();

    let mut domain_map: HashMap<String, usize> = HashMap::new();
    let mut severity_map: HashMap<String, usize> = HashMap::new();
    let mut fatal_count: usize = 0;

    for entry in RULE_REGISTRY {
        let domain = format!("{:?}", entry.domain);
        *domain_map.entry(domain).or_insert(0) += 1;
        *severity_map.entry(entry.severity.as_str().to_string()).or_insert(0) += 1;
        if entry.severity == crate::core::models::ViolationSeverity::Fatal {
            fatal_count += 1;
        }
    }

    let mut rules_by_domain: Vec<(String, usize)> = domain_map.into_iter().collect();
    rules_by_domain.sort_by(|a, b| a.0.cmp(&b.0));

    let mut rules_by_severity: Vec<(String, usize)> = severity_map.into_iter().collect();
    let severity_order = |s: &str| match s {
        "F" => 0,
        "G" => 1,
        "J" => 2,
        _ => 3,
    };
    rules_by_severity.sort_by_key(|(s, _)| severity_order(s));

    let fatal_ratio = if total_rules > 0 {
        fatal_count as f64 / total_rules as f64
    } else {
        0.0
    };

    RuleAuditMetric {
        total_rules,
        rules_by_domain,
        rules_by_severity,
        fatal_ratio,
    }
}

/// 规则密度指标（知止/G1：治理投入 vs 产出方差）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDensityMetric {
    pub total_rules: usize,
    pub total_docs: usize,
    pub overall_density: f64,
    pub density_by_nature: Vec<(String, f64)>,
    pub variance_by_nature: Vec<(String, f64)>,
    pub correlation_note: String,
}

/// 从规则注册表 + 文档分布 + 验证记录计算规则密度
///
/// 规则密度 = total_rules / total_docs。按 nature 分组的密度中，
/// 所有 nature 共享同一规则池，密度差异仅反映文档分布。
/// variance_by_nature 取自 ValidationCompleted 记录的 avg_fatal_by_nature。
pub fn compute_rule_density(
    validation_records: &[MetricRecord],
    nature_counts: &[(String, usize)],
) -> RuleDensityMetric {
    let total_rules = RULE_REGISTRY.len();
    let total_docs: usize = nature_counts.iter().map(|(_, c)| c).sum();

    let overall_density = if total_docs > 0 {
        total_rules as f64 / total_docs as f64
    } else {
        0.0
    };

    let mut density_by_nature: Vec<(String, f64)> = nature_counts
        .iter()
        .map(|(n, c)| {
            let d = if *c > 0 {
                total_rules as f64 / *c as f64
            } else {
                0.0
            };
            (n.clone(), d)
        })
        .collect();
    density_by_nature.sort_by(|a, b| a.0.cmp(&b.0));

    // 从 ValidationCompleted 提取 avg_fatal_by_nature
    let variance = compute_variance_metric(validation_records);
    let variance_by_nature = variance.avg_fatal_by_nature;

    let correlation_note = format!(
        "样本不足：当前仅有 {} 个 nature 类别，{} 条验证记录，不足以计算统计相关性。\
         需积累更多数据后重新检验规则密度与产出方差的关系。",
        nature_counts.len(),
        validation_records.len()
    );

    RuleDensityMetric {
        total_rules,
        total_docs,
        overall_density,
        density_by_nature,
        variance_by_nature,
        correlation_note,
    }
}

/// 权衡文档覆盖率指标（损补/G4：ADR 覆盖率）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeoffCoverageMetric {
    pub total_decisions: usize,
    pub adr_covered: usize,
    pub adr_coverage_rate: f64,
    pub rule_changes_note: String,
}

/// 扫描 decision 文档内容，检测 ADR 三段式覆盖率
///
/// 检测 `## 背景`、`## 决策`、`## 后果` 三个 Markdown 二级标题，
/// 标题下方必须有非空内容（非仅空白行）。
pub fn compute_tradeoff_coverage(docs: &[crate::core::models::Document]) -> TradeoffCoverageMetric {
    let decisions: Vec<&crate::core::models::Document> = docs
        .iter()
        .filter(|d| d.nature == "decision")
        .collect();

    let total_decisions = decisions.len();

    let adr_covered = decisions
        .iter()
        .filter(|d| has_adr_sections(&d.content))
        .count();

    let adr_coverage_rate = if total_decisions > 0 {
        adr_covered as f64 / total_decisions as f64
    } else {
        0.0
    };

    let rule_changes_note = if total_decisions > 0 {
        "规则增删比率需累积 ProjectSnapshot 历史数据，当前不可计算。\
         建议持续调用 project_status 以积累快照数据。".to_string()
    } else {
        "当前无 decision 文档，不可计算 ADR 覆盖率。".to_string()
    };

    TradeoffCoverageMetric {
        total_decisions,
        adr_covered,
        adr_coverage_rate,
        rule_changes_note,
    }
}

/// 检测文档内容是否包含 ADR 三段式结构
fn has_adr_sections(content: &str) -> bool {
    let has_background = has_non_empty_section(content, "## 背景");
    let has_decision = has_non_empty_section(content, "## 决策");
    let has_consequences = has_non_empty_section(content, "## 后果");
    has_background && has_decision && has_consequences
}

/// 检测 Markdown 二级标题 `## {section_name}` 下方是否有非空内容
fn has_non_empty_section(content: &str, section_name: &str) -> bool {
    let mut found_heading = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if found_heading {
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                return false;
            }
            return true;
        }
        if trimmed == section_name {
            found_heading = true;
        }
    }
    false
}

/// 趋势对齐指标（顺势/G5：审查频率-变更频率比值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAlignmentMetric {
    pub validation_count: usize,
    pub index_count: usize,
    pub review_change_ratio: f64,
    pub window_start: String,
    pub window_end: String,
    pub interpretation_note: String,
}

/// 从 ValidationCompleted 和 IndexCompleted 记录计算审查/变更比值
///
/// review_change_ratio = validation_count / index_count。
/// 接近 1 表示审查与变更同步，远小于 1 表示审查滞后于变更。
/// 仅覆盖时势维度，地势与人势维度未操作化。
pub fn compute_trend_alignment(
    validation_records: &[MetricRecord],
    index_records: &[MetricRecord],
) -> TrendAlignmentMetric {
    let validation_count = validation_records.len();
    let index_count = index_records.len();

    let review_change_ratio = if index_count > 0 {
        validation_count as f64 / index_count as f64
    } else {
        0.0
    };

    let window_start = validation_records
        .iter()
        .chain(index_records.iter())
        .map(|r| r.created_at.as_str())
        .min()
        .unwrap_or("")
        .to_string();

    let window_end = validation_records
        .iter()
        .chain(index_records.iter())
        .map(|r| r.created_at.as_str())
        .max()
        .unwrap_or("")
        .to_string();

    let interpretation_note = format!(
        "审查频率-变更频率比值为 {:.2}（{} 次审查 / {} 次变更）。\
         比值接近 1 表示审查与变更同步，远小于 1 表示审查滞后。\
         仅覆盖时势维度（项目阶段），地势（代码区域差异）和人势（认知源数量变化）维度未操作化。\
         建议积累 >= 3 个月数据后计算滑动窗口比值。",
        review_change_ratio, validation_count, index_count
    );

    TrendAlignmentMetric {
        validation_count,
        index_count,
        review_change_ratio,
        window_start,
        window_end,
        interpretation_note,
    }
}

#[cfg(test)]
mod tests {
    use super::compute_rule_audit;
    use super::compute_rule_density;
    use super::compute_snapshot_diff;
    use super::compute_tradeoff_coverage;
    use super::compute_trend_alignment;
    use super::compute_variance_metric;
    use super::MetricRecord;
    use crate::core::database::SihDatabase;
    use crate::core::database::SqliteBackend;

    #[tokio::test]
    async fn test_record_and_query_metrics() {
        let db = SqliteBackend::open_in_memory().unwrap();

        // 记录一条度量
        let payload = r#"{"test": true}"#;
        db.record_metric("TestEvent", payload).await.unwrap();

        // 查询
        let records = db.query_metrics("TestEvent", 10).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].event_type, "TestEvent");
        assert_eq!(records[0].payload_json, payload);
    }

    #[tokio::test]
    async fn test_query_empty_metrics() {
        let db = SqliteBackend::open_in_memory().unwrap();
        let records = db.query_metrics("NonExistent", 10).await.unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn test_latest_snapshot_empty() {
        let db = SqliteBackend::open_in_memory().unwrap();
        let snapshot = db.get_latest_snapshot().await.unwrap();
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_compute_variance_empty() {
        let records: Vec<MetricRecord> = Vec::new();
        let m = compute_variance_metric(&records);
        assert_eq!(m.total_docs, 0);
        assert_eq!(m.pass_rate, 0.0);
        assert_eq!(m.avg_fatal_count, 0.0);
        assert_eq!(m.avg_guideline_count, 0.0);
        assert_eq!(m.fatal_count_stddev, 0.0);
        assert!(m.pass_rate_by_nature.is_empty());
        assert!(m.avg_fatal_by_nature.is_empty());
        assert_eq!(m.window_start, "");
        assert_eq!(m.window_end, "");
    }

    #[test]
    fn test_compute_variance_basic() {
        let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":2,"judgment_count":1,"passed":true}"#;
        let p2 = r#"{"doc_id":"d2","nature":"spec","stage":"2/3","fatal_count":0,"guideline_count":1,"judgment_count":0,"passed":true}"#;
        let p3 = r#"{"doc_id":"d3","nature":"proposal","stage":"1/3","fatal_count":1,"guideline_count":0,"judgment_count":1,"passed":false}"#;
        let records = vec![
            MetricRecord {
                id: 1,
                event_type: "ValidationCompleted".to_string(),
                payload_json: p1.to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            MetricRecord {
                id: 2,
                event_type: "ValidationCompleted".to_string(),
                payload_json: p2.to_string(),
                created_at: "2024-01-02T00:00:00Z".to_string(),
            },
            MetricRecord {
                id: 3,
                event_type: "ValidationCompleted".to_string(),
                payload_json: p3.to_string(),
                created_at: "2024-01-03T00:00:00Z".to_string(),
            },
        ];
        let m = compute_variance_metric(&records);
        assert_eq!(m.total_docs, 3);
        assert!((m.pass_rate - 2.0 / 3.0).abs() < 1e-9);
        assert!((m.avg_fatal_count - 1.0 / 3.0).abs() < 1e-9);
        assert!((m.fatal_count_stddev - (2.0_f64 / 9.0_f64).sqrt()).abs() < 1e-9);
        // pass_rate_by_nature 按 key 排序：proposal -> 0.0, spec -> 1.0
        assert_eq!(m.pass_rate_by_nature.len(), 2);
        assert_eq!(m.pass_rate_by_nature[0], ("proposal".to_string(), 0.0));
        assert_eq!(m.pass_rate_by_nature[1], ("spec".to_string(), 1.0));
        // avg_fatal_by_nature 按 key 排序：proposal -> 1.0, spec -> 0.0
        assert_eq!(m.avg_fatal_by_nature.len(), 2);
        assert_eq!(m.avg_fatal_by_nature[0], ("proposal".to_string(), 1.0));
        assert_eq!(m.avg_fatal_by_nature[1], ("spec".to_string(), 0.0));
    }

    #[test]
    fn test_compute_variance_ignores_invalid_payload() {
        let valid = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":2,"judgment_count":1,"passed":true}"#;
        let invalid = r#"{"this is not valid":}"#;
        let records = vec![
            MetricRecord {
                id: 1,
                event_type: "ValidationCompleted".to_string(),
                payload_json: valid.to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            MetricRecord {
                id: 2,
                event_type: "ValidationCompleted".to_string(),
                payload_json: invalid.to_string(),
                created_at: "2024-01-02T00:00:00Z".to_string(),
            },
        ];
        let m = compute_variance_metric(&records);
        assert_eq!(m.total_docs, 1);
        assert_eq!(m.pass_rate, 1.0);
        assert_eq!(m.avg_fatal_count, 0.0);
    }

    /// 构造一条 ProjectSnapshot MetricRecord
    fn snapshot_record(
        id: i64,
        created_at: &str,
        total_docs: usize,
        total_rules: usize,
        docs_by_stage: &[(&str, usize)],
        docs_by_nature: &[(&str, usize)],
        fatal_violations_total: usize,
    ) -> MetricRecord {
        let stage: Vec<(String, usize)> = docs_by_stage
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let nature: Vec<(String, usize)> = docs_by_nature
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let payload = serde_json::json!({
            "total_docs": total_docs,
            "total_rules": total_rules,
            "docs_by_stage": stage,
            "docs_by_nature": nature,
            "fatal_violations_total": fatal_violations_total,
        })
        .to_string();
        MetricRecord {
            id,
            event_type: "ProjectSnapshot".to_string(),
            payload_json: payload,
            created_at: created_at.to_string(),
        }
    }

    #[test]
    fn test_snapshot_diff_basic() {
        let prev = snapshot_record(
            1,
            "2024-01-01T00:00:00Z",
            10,
            14,
            &[("1/3", 5), ("2/3", 3), ("3/3", 2)],
            &[("spec", 6), ("decision", 4)],
            2,
        );
        let curr = snapshot_record(
            2,
            "2024-01-02T00:00:00Z",
            12,
            14,
            &[("1/3", 6), ("2/3", 4), ("3/3", 2)],
            &[("spec", 7), ("decision", 5)],
            1,
        );
        let diff = compute_snapshot_diff(&prev, &curr).expect("should compute diff");
        assert_eq!(diff.previous_time, "2024-01-01T00:00:00Z");
        assert_eq!(diff.current_time, "2024-01-02T00:00:00Z");
        assert_eq!(diff.docs_delta, 2);
        assert_eq!(diff.rules_delta, 0);
        assert_eq!(diff.fatal_violations_delta, -1);
        assert!(!diff.rules_grew);
        assert!(diff.docs_grew);
        // docs_by_stage_delta 按 key 排序：1/3, 2/3, 3/3
        assert_eq!(diff.docs_by_stage_delta.len(), 3);
        assert_eq!(diff.docs_by_stage_delta[0], ("1/3".to_string(), 1));
        assert_eq!(diff.docs_by_stage_delta[1], ("2/3".to_string(), 1));
        assert_eq!(diff.docs_by_stage_delta[2], ("3/3".to_string(), 0));
    }

    #[test]
    fn test_snapshot_diff_invalid_payload() {
        let prev = MetricRecord {
            id: 1,
            event_type: "ProjectSnapshot".to_string(),
            payload_json: r#"{"this is not valid":}"#.to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let curr = snapshot_record(
            2,
            "2024-01-02T00:00:00Z",
            12,
            14,
            &[("1/3", 6)],
            &[("spec", 7)],
            1,
        );
        let diff = compute_snapshot_diff(&prev, &curr);
        assert!(diff.is_none());
    }

    #[test]
    fn test_snapshot_diff_new_nature() {
        let prev = snapshot_record(
            1,
            "2024-01-01T00:00:00Z",
            10,
            5,
            &[("1/3", 10)],
            &[("spec", 10)],
            0,
        );
        let curr = snapshot_record(
            2,
            "2024-01-02T00:00:00Z",
            12,
            5,
            &[("1/3", 12)],
            &[("spec", 10), ("decision", 2)],
            0,
        );
        let diff = compute_snapshot_diff(&prev, &curr).expect("should compute diff");
        // docs_by_nature_delta 按 key 排序：decision, spec
        assert_eq!(diff.docs_by_nature_delta.len(), 2);
        assert_eq!(diff.docs_by_nature_delta[0], ("decision".to_string(), 2));
        assert_eq!(diff.docs_by_nature_delta[1], ("spec".to_string(), 0));
    }

    #[test]
    fn test_compute_rule_audit_basic() {
        let audit = compute_rule_audit();
        assert_eq!(audit.total_rules, 14);
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Frontmatter"));
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Structure"));
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Reference"));
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Governance"));
        assert!(audit.rules_by_severity.iter().any(|(s, _)| s == "F"));
        assert!(audit.rules_by_severity.iter().any(|(s, _)| s == "G"));
        assert!(audit.rules_by_severity.iter().any(|(s, _)| s == "J"));
        assert!(audit.fatal_ratio > 0.0 && audit.fatal_ratio < 1.0);
    }

    #[test]
    fn test_compute_rule_audit_registry_consistency() {
        let audit = compute_rule_audit();
        assert_eq!(audit.total_rules, crate::core::validator::RULE_COUNT);
    }

    #[test]
    fn test_compute_rule_density_empty() {
        let records: Vec<MetricRecord> = Vec::new();
        let nature_counts: Vec<(String, usize)> = Vec::new();
        let m = compute_rule_density(&records, &nature_counts);
        assert_eq!(m.total_rules, 14);
        assert_eq!(m.total_docs, 0);
        assert_eq!(m.overall_density, 0.0);
        assert!(m.correlation_note.contains("样本不足"));
    }

    #[test]
    fn test_compute_rule_density_basic() {
        let vc = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":2,"judgment_count":1,"passed":true}"#;
        let records = vec![
            MetricRecord {
                id: 1, event_type: "ValidationCompleted".to_string(),
                payload_json: vc.to_string(), created_at: "2024-01-01T00:00:00Z".to_string(),
            },
        ];
        let nature_counts = vec![("spec".to_string(), 3usize), ("proposal".to_string(), 2usize)];
        let m = compute_rule_density(&records, &nature_counts);
        assert_eq!(m.total_rules, 14);
        assert_eq!(m.total_docs, 5);
        assert!((m.overall_density - 14.0 / 5.0).abs() < 1e-9);
        assert_eq!(m.density_by_nature.len(), 2);
        assert!((m.density_by_nature[0].1 - 14.0 / 2.0).abs() < 1e-9);
        assert!((m.density_by_nature[1].1 - 14.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_compute_tradeoff_coverage_empty() {
        let docs: Vec<crate::core::models::Document> = Vec::new();
        let m = compute_tradeoff_coverage(&docs);
        assert_eq!(m.total_decisions, 0);
        assert_eq!(m.adr_covered, 0);
        assert_eq!(m.adr_coverage_rate, 0.0);
        assert!(m.rule_changes_note.contains("不可计算"));
    }

    #[test]
    fn test_compute_tradeoff_coverage_basic() {
        use crate::core::models::{DocStatus, Document, Frontmatter, Stage};
        use chrono::Utc;
        let doc_with_adr = Document {
            id: "test-01".into(), stage: Stage::Resolve, title: "Test Decision".into(),
            upstream: None,
            frontmatter: Frontmatter { id: "test-01".into(), stage: Stage::Resolve, upstream: None, decided_by: Some("human".into()), extra: serde_json::Value::Null },
            content: "## 背景\n\nSome background\n\n## 决策\n\nSome decision\n\n## 后果\n\nSome consequences".into(),
            status: DocStatus::Ok, indexed_at: Utc::now(), nature: "decision".into(),
        };
        let doc_without_adr = Document {
            id: "test-02".into(), stage: Stage::Propose, title: "Test Proposal".into(),
            upstream: None,
            frontmatter: Frontmatter { id: "test-02".into(), stage: Stage::Propose, upstream: None, decided_by: None, extra: serde_json::Value::Null },
            content: "Just some content without ADR sections".into(),
            status: DocStatus::Ok, indexed_at: Utc::now(), nature: "decision".into(),
        };
        let m = compute_tradeoff_coverage(&[doc_with_adr, doc_without_adr]);
        assert_eq!(m.total_decisions, 2);
        assert_eq!(m.adr_covered, 1);
        assert!((m.adr_coverage_rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_compute_trend_alignment_empty() {
        let validations: Vec<MetricRecord> = Vec::new();
        let indexes: Vec<MetricRecord> = Vec::new();
        let m = compute_trend_alignment(&validations, &indexes);
        assert_eq!(m.validation_count, 0);
        assert_eq!(m.index_count, 0);
        assert_eq!(m.review_change_ratio, 0.0);
        assert!(m.interpretation_note.contains("仅覆盖时势维度"));
    }

    #[test]
    fn test_compute_trend_alignment_basic() {
        let vc = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":0,"judgment_count":0,"passed":true}"#;
        let ic = r#"{"doc_id":"d1","nature":"spec"}"#;
        let validations = vec![
            MetricRecord { id: 1, event_type: "ValidationCompleted".to_string(), payload_json: vc.to_string(), created_at: "2024-01-01T00:00:00Z".to_string() },
            MetricRecord { id: 2, event_type: "ValidationCompleted".to_string(), payload_json: vc.to_string(), created_at: "2024-01-02T00:00:00Z".to_string() },
        ];
        let indexes = vec![
            MetricRecord { id: 3, event_type: "IndexCompleted".to_string(), payload_json: ic.to_string(), created_at: "2024-01-01T00:00:00Z".to_string() },
            MetricRecord { id: 4, event_type: "IndexCompleted".to_string(), payload_json: ic.to_string(), created_at: "2024-01-02T00:00:00Z".to_string() },
            MetricRecord { id: 5, event_type: "IndexCompleted".to_string(), payload_json: ic.to_string(), created_at: "2024-01-03T00:00:00Z".to_string() },
        ];
        let m = compute_trend_alignment(&validations, &indexes);
        assert_eq!(m.validation_count, 2);
        assert_eq!(m.index_count, 3);
        assert!((m.review_change_ratio - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(m.window_start, "2024-01-01T00:00:00Z");
        assert_eq!(m.window_end, "2024-01-03T00:00:00Z");
    }
}
