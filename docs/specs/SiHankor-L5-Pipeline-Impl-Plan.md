---
id: 260628-2100-l5-pipeline-impl-plan
stage: 2/3
upstream: 260628-1800-l5-pipeline-design
---
# L5 度量管道补全实现计划

> **For agentic workers:** Implement this plan task-by-task using subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 4 条 L5 映射链（链6/8/9/10）从 L5 升级到 L2，实现规则注册表 + 4 个度量计算函数 + 4 个 MCP 工具。

**Architecture:** 不新增文件，修改 3 个源文件 + 1 个文档。规则注册表在 validator.rs 作为编译期常量数组，计算函数在 metrics.rs 遵循既有纯函数模式，MCP 工具在 governance.rs 遵循 EmptyParams 模式。与 iCT 完全解耦。

**Tech Stack:** Rust, rusqlite, rmcp, serde_json, tokio. No new dependencies.

**Design spec:** `docs/specs/SiHankor-L5-Pipeline-Design.md`

---

### Task 1: 规则注册表 RULE_REGISTRY

**Files:**
- Modify: `src/core/validator.rs:1-8` (添加 RULE_REGISTRY, 修改 RULE_COUNT)
- Modify: `src/core/metrics.rs:1-5` (添加 RuleAuditMetric 及 compute_rule_audit)

**Key files to reference:**
- `src/core/validator.rs` — 已有的 RULE_COUNT, ValidationDomain, ViolationSeverity
- `src/core/metrics.rs` — 已有的 VarianceMetric, compute_variance_metric 模式

- [ ] **Step 1: Add RuleRegistryEntry struct and RULE_REGISTRY to validator.rs**

Replace the current `pub const RULE_COUNT: usize = 14;` and the comment above it with:

```rust
/// 规则注册表条目：每条实际发射 violation 的规则在编译期注册
///
/// 注册表是治理规则的"意图"显式声明。顺因（道二）要求意图先于代码：
/// 规则的治理域和严格度不应隐含在 if-else 控制流中，而应在此处显式声明。
///
/// 注意：仅包含实际发射 violation 的 14 条工程规则。
/// V-G-07 和 V-G-10 是概念规则（声明了 ID 但不发射 violation），
/// 未来实现时再加入注册表，避免引入"幽灵规则"导致统计失真。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRegistryEntry {
    pub rule_id: &'static str,
    pub severity: ViolationSeverity,
    pub domain: ValidationDomain,
    pub description: &'static str,
}

/// 当前 validator 中实际发射 violation 的 14 条规则
pub const RULE_REGISTRY: &[RuleRegistryEntry] = &[
    RuleRegistryEntry {
        rule_id: "V-F-01",
        severity: ViolationSeverity::Fatal,
        domain: ValidationDomain::Frontmatter,
        description: "id 格式校验（YYMMDD-HHMM[-NNN]-语义短名）",
    },
    RuleRegistryEntry {
        rule_id: "V-F-03",
        severity: ViolationSeverity::Fatal,
        domain: ValidationDomain::Frontmatter,
        description: "stage 必须是合法编码（1/3, 2/3, 3/3, X, 0/<id>）",
    },
    RuleRegistryEntry {
        rule_id: "V-F-04",
        severity: ViolationSeverity::Fatal,
        domain: ValidationDomain::Frontmatter,
        description: "upstream 必填性由 nature 决定：非 note 文档必须填写",
    },
    RuleRegistryEntry {
        rule_id: "V-F-05",
        severity: ViolationSeverity::Fatal,
        domain: ValidationDomain::Structure,
        description: "正文禁止 --- 水平线（仅 frontmatter 分隔符可用）",
    },
    RuleRegistryEntry {
        rule_id: "V-F-06",
        severity: ViolationSeverity::Fatal,
        domain: ValidationDomain::Governance,
        description: "decided-by 不得使用 ai 前缀值（ai-assist 等）",
    },
    RuleRegistryEntry {
        rule_id: "V-F-07",
        severity: ViolationSeverity::Fatal,
        domain: ValidationDomain::Governance,
        description: "非 decisions/ 目录的文档不得有 decided-by 字段",
    },
    RuleRegistryEntry {
        rule_id: "V-G-02",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Structure,
        description: "文档必须位于合法治理目录下",
    },
    RuleRegistryEntry {
        rule_id: "V-G-03",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Structure,
        description: "子目录深度 <= 4",
    },
    RuleRegistryEntry {
        rule_id: "V-G-04",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Structure,
        description: "表格列数 <= 3",
    },
    RuleRegistryEntry {
        rule_id: "V-G-05",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Structure,
        description: "代码块必须声明语言标签",
    },
    RuleRegistryEntry {
        rule_id: "V-G-06",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Structure,
        description: "禁止 emoji 和非 ASCII/CJK 符号",
    },
    RuleRegistryEntry {
        rule_id: "V-G-08",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Reference,
        description: "stage X（废弃）文档不可被引用",
    },
    RuleRegistryEntry {
        rule_id: "V-G-09",
        severity: ViolationSeverity::Guideline,
        domain: ValidationDomain::Governance,
        description: "stage 2/3 或 3/3 的 decisions/ 文档应有 decided-by",
    },
    RuleRegistryEntry {
        rule_id: "V-J-01",
        severity: ViolationSeverity::Judgment,
        domain: ValidationDomain::Structure,
        description: "列表嵌套不超过 2 层",
    },
];

/// 当前 validator 中定义的规则总数
/// 从 RULE_REGISTRY 派生，增删规则时自动同步，无需手动修改
pub const RULE_COUNT: usize = RULE_REGISTRY.len();
```

- [ ] **Step 2: Run tests to verify no regression**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test 2>&1 | tail -5
```

Expected: 122 passed (or more if tests added); all existing tests pass.

- [ ] **Step 3: Commit**

```bash
cd /Users/moc/projects/SiHankor/sihankor
git add src/core/validator.rs
git commit -m "refactor: add RULE_REGISTRY as single source of truth for rule metadata

Export 14 rule entries as compile-time constant array with rule_id,
severity, domain, and description. RULE_COUNT now derived from
RULE_REGISTRY.len() instead of being a hardcoded constant.

顺因：规则的治理意图（domain+severity）从代码控制流中提取
为显式声明，消除手动同步点。"
```

---

### Task 2: 度量计算函数 (metrics.rs)

**Files:**
- Modify: `src/core/metrics.rs` (末尾添加 4 个 struct + 4 个 compute 函数 + 单元测试)

**Key files to reference:**
- `src/core/validator.rs` — RULE_REGISTRY (新增于 Task 1)
- `src/core/metrics.rs` — VarianceMetric, SnapshotDiff, compute_variance_metric, compute_snapshot_diff 模式
- `src/core/database.rs` — SihDatabase trait (get_all_documents, query_metrics, count_by_nature)

- [ ] **Step 1: Write failing test for compute_rule_audit**

Append to `src/core/metrics.rs`, inside the existing `#[cfg(test)] mod tests {` block:

```rust
    #[test]
    fn test_compute_rule_audit_basic() {
        let audit = compute_rule_audit();
        assert_eq!(audit.total_rules, 14);
        // Verify domain counts match the registry
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Frontmatter"));
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Structure"));
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Reference"));
        assert!(audit.rules_by_domain.iter().any(|(d, _)| d == "Governance"));
        // Verify severity counts
        assert!(audit.rules_by_severity.iter().any(|(s, _)| s == "F"));
        assert!(audit.rules_by_severity.iter().any(|(s, _)| s == "G"));
        assert!(audit.rules_by_severity.iter().any(|(s, _)| s == "J"));
        // fatal_ratio should be between 0 and 1
        assert!(audit.fatal_ratio > 0.0 && audit.fatal_ratio < 1.0);
    }

    #[test]
    fn test_compute_rule_audit_registry_consistency() {
        // Verify RULE_REGISTRY.len() == RULE_COUNT
        let audit = compute_rule_audit();
        assert_eq!(audit.total_rules, crate::core::validator::RULE_COUNT);
    }
```

Read the metrics.rs file to find the exact end-of-test-module location.

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test test_compute_rule_audit 2>&1 | tail -5
```

Expected: FAIL with "cannot find function `compute_rule_audit`"

- [ ] **Step 3: Add imports, RuleAuditMetric struct, and compute_rule_audit function**

At the top of `src/core/metrics.rs`, add:

```rust
use crate::core::validator::{RULE_REGISTRY, RuleRegistryEntry};
```

Before the `#[cfg(test)]` block, append:

```rust
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

    // rules_by_domain
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
    // Sort: F, G, J
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
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test test_compute_rule_audit 2>&1 | tail -10
```

Expected: PASS for both tests.

- [ ] **Step 5: Write failing tests for compute_rule_density, compute_tradeoff_coverage, compute_trend_alignment**

Append these 5 test functions to the test module:

```rust
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
        // density_by_nature sorted by key: proposal, spec
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
```

- [ ] **Step 6: Run test to verify they fail**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test test_compute_rule_density test_compute_tradeoff test_compute_trend 2>&1 | tail -10
```

Expected: FAIL with "function not found" for compute_rule_density, compute_tradeoff_coverage, compute_trend_alignment.

- [ ] **Step 7: Add RuleDensityMetric, compute_rule_density**

Append before the test block:

```rust
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

    // 样本不足，无法做相关性检验
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
```

- [ ] **Step 8: Add TradeoffCoverageMetric, compute_tradeoff_coverage**

Append:

```rust
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
        "当前无 decision 文档，无需计算 ADR 覆盖率。".to_string()
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
            // 已找到目标标题，检查下一非空行是否为内容（不是另一个标题）
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                return false; // 紧接着另一个标题，说明该 section 无内容
            }
            return true; // 有非空、非标题内容
        }
        if trimmed == section_name {
            found_heading = true;
        }
    }
    false
}
```

- [ ] **Step 9: Add TrendAlignmentMetric, compute_trend_alignment**

Append:

```rust
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
```

- [ ] **Step 10: Run all new tests**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test compute_rule_audit compute_rule_density compute_tradeoff compute_trend 2>&1 | tail -20
```

Expected: All 7 new tests PASS.

- [ ] **Step 11: Run full test suite**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test 2>&1 | grep "test result"
```

Expected: All tests pass (should be 129+ unit tests + 5 integration).

- [ ] **Step 12: Commit**

```bash
cd /Users/moc/projects/SiHankor/sihankor
git add src/core/metrics.rs
git commit -m "feat: add 4 metric computation functions for chains 6/8/9/10

- compute_rule_audit (链8/G3): rule count by domain/severity + fatal_ratio
- compute_rule_density (链6/G1): rule density by nature + variance correlation note
- compute_tradeoff_coverage (链9/G4): ADR 三段式 coverage rate
- compute_trend_alignment (链10/G5): review/change frequency ratio

All functions follow existing variance/snapshot pure-function pattern.
7 unit tests for new computation functions."
```

---

### Task 3: MCP 工具注册 (governance.rs)

**Files:**
- Modify: `src/mcp_server/governance.rs` (添加 4 个工具 + 4 个 format 辅助 + 6 个集成测试)

**Key files to reference:**
- `src/mcp_server/governance.rs` — variance_metric, snapshot_diff 的 EmptyParams 模式 (line 707-729)
- `src/mcp_server/governance.rs` — format_variance_metric 格式模式 (line 769)
- `src/mcp_server/governance.rs` — 已有集成测试模式 (test_variance_metric, test_snapshot_diff 等)

- [ ] **Step 1: Update imports in governance.rs**

Replace the existing metrics import line (line 9-11):

```rust
use crate::core::metrics::{
    compute_latest_snapshot_diff, compute_variance_metric, MetricEvent, SnapshotDiff, VarianceMetric,
};
```

With:

```rust
use crate::core::metrics::{
    compute_latest_snapshot_diff, compute_rule_audit, compute_rule_density,
    compute_tradeoff_coverage, compute_trend_alignment, compute_variance_metric, MetricEvent,
    RuleAuditMetric, RuleDensityMetric, SnapshotDiff, TradeoffCoverageMetric, TrendAlignmentMetric,
    VarianceMetric,
};
```

- [ ] **Step 2: Add 4 MCP tool methods to SihankorService impl block**

After `snapshot_diff` method (line 729), before the closing `}` of the impl block, add:

```rust
    /// 规则审计：统计各治理域和严格度的规则分布
    #[tool(
        description = "[SiHankor] 规则审计：统计各治理域的规则数分布与 Fatal 级规则占比"
    )]
    pub fn rule_audit(&self, Parameters(_): Parameters<EmptyParams>) -> String {
        let audit = compute_rule_audit();
        format_rule_audit(&audit)
    }

    /// 规则密度：统计各 nature 的治理投入密度
    #[tool(
        description = "[SiHankor] 规则密度：计算各文档类型的规则密度与治理投入分布"
    )]
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
    #[tool(
        description = "[SiHankor] 趋势对齐：计算审查频率与变更频率的比值，评估治理力度适配度"
    )]
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
```

Note: `rule_audit` is `pub fn` (not `pub async fn`) because `compute_rule_audit()` is a pure function that reads from a compile-time constant. All others are `pub async fn` because they query the database.

- [ ] **Step 3: Add 4 format helper functions**

After `format_snapshot_diff` function (or at the end of the file before tests), add:

```rust
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
         Fatal 级规则占比: {:.1}% ({}/{}）",
        audit.total_rules,
        domain_lines,
        severity_lines,
        audit.fatal_ratio * 100.0,
        (audit.fatal_ratio * audit.total_rules as f64) as usize,
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
            .map(|(n, d)| format!("  {}: {:.2} ({} 条规则 / {} 篇文档)", n, d, density.total_rules, n))
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
```

- [ ] **Step 4: Run tests to verify compilation**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo build 2>&1 | tail -5
```

Expected: Compile success.

- [ ] **Step 5: Write integration tests for new tools**

Find the existing test module in governance.rs and add these 6 tests:

```rust
    /// Verify rule_audit tool returns valid data
    #[tokio::test]
    async fn test_rule_audit_tool() {
        let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
        let service = make_service(db.clone());

        let result = service.rule_audit(Parameters(EmptyParams)).await.unwrap();
        assert!(result.contains("SiHankor Rule Audit"));
        assert!(result.contains("总规则数"));
        assert!(result.contains("Frontmatter"));
        assert!(result.contains("Structure"));
        assert!(result.contains("Fatal 级规则占比"));
    }

    /// Verify rule_density tool returns valid data (empty db)
    #[tokio::test]
    async fn test_rule_density_tool_empty() {
        let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
        let service = make_service(db.clone());

        let result = service.rule_density(Parameters(EmptyParams)).await.unwrap();
        assert!(result.contains("SiHankor Rule Density"));
        assert!(result.contains("样本不足"));
    }

    /// Verify rule_density tool with seeded data
    #[tokio::test]
    async fn test_rule_density_tool_with_data() {
        let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
        let service = make_service(db.clone());

        // Seed: index a document to generate ValidationCompleted
        let vc = serde_json::json!({
            "doc_id": "d1", "nature": "spec", "stage": "1/3",
            "fatal_count": 0, "guideline_count": 2,
            "judgment_count": 1, "passed": true
        });
        db.record_metric("ValidationCompleted", &vc.to_string()).await.unwrap();
        db.upsert_document(make_test_doc("d1", "spec", "1/3")).await.unwrap();

        let result = service.rule_density(Parameters(EmptyParams)).await.unwrap();
        assert!(result.contains("SiHankor Rule Density"));
        assert!(result.contains("总规则数"));
    }

    /// Verify tradeoff_coverage tool returns valid data (empty db)
    #[tokio::test]
    async fn test_tradeoff_coverage_tool_empty() {
        let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
        let service = make_service(db.clone());

        let result = service.tradeoff_coverage(Parameters(EmptyParams)).await.unwrap();
        assert!(result.contains("SiHankor Tradeoff Coverage"));
        assert!(result.contains("无 decision 文档"));
    }

    /// Verify trend_alignment tool returns valid data (empty db)
    #[tokio::test]
    async fn test_trend_alignment_tool_empty() {
        let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
        let service = make_service(db.clone());

        let result = service.trend_alignment(Parameters(EmptyParams)).await.unwrap();
        assert!(result.contains("SiHankor Trend Alignment"));
        assert!(result.contains("仅覆盖时势维度"));
    }

    /// Verify trend_alignment tool with seeded data
    #[tokio::test]
    async fn test_trend_alignment_tool_with_data() {
        let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
        let service = make_service(db.clone());

        let vc = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":0,"judgment_count":0,"passed":true}"#;
        let ic = r#"{"doc_id":"d1","nature":"spec"}"#;
        db.record_metric("ValidationCompleted", vc).await.unwrap();
        db.record_metric("IndexCompleted", ic).await.unwrap();

        let result = service.trend_alignment(Parameters(EmptyParams)).await.unwrap();
        assert!(result.contains("SiHankor Trend Alignment"));
        assert!(result.contains("审查次数"));
        assert!(result.contains("变更次数"));
    }
```

Note: These tests use `make_service` helper which should already exist in the test module. If it doesn't exist, you need to add:

```rust
    fn make_service(db: Arc<dyn SihDatabase>) -> SihankorService {
        SihankorService {
            db,
            config: PipelineConfig::default(),
            glossary: None,
        }
    }

    fn make_test_doc(id: &str, nature: &str, stage: &str) -> Document {
        use crate::core::models::{DocStatus, Frontmatter, Stage};
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
```

- [ ] **Step 6: Run new MCP tool tests**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test rule_audit rule_density tradeoff_coverage trend_alignment 2>&1 | tail -20
```

Expected: All 6 new tests PASS.

- [ ] **Step 7: Run full test suite**

```bash
cd /Users/moc/projects/SiHankor/sihankor && cargo test 2>&1 | grep "test result"
```

Expected: All tests pass (should be 135+ unit tests + 5 integration).

- [ ] **Step 8: Commit**

```bash
cd /Users/moc/projects/SiHankor/sihankor
git add src/mcp_server/governance.rs
git commit -m "feat: add 4 MCP tools for chain 6/8/9/10 metric queries

- rule_audit: static rule distribution by domain + fatal_ratio
- rule_density: per-nature rule density + variance correlation note
- tradeoff_coverage: ADR section coverage in decision docs
- trend_alignment: review/change frequency ratio (temporal only)

All tools follow variance_metric/snapshot_diff EmptyParams pattern.
6 integration tests for new tools."
```

---

### Task 4: 工程映射 L 级别更新

**Files:**
- Modify: `docs/specs/engineering/Engineering-Mapping.sih.md` (更新链6/8/9/10 L级别 + 总览表 + 代码引用)

**Key files to reference:**
- `docs/specs/engineering/Engineering-Mapping.sih.md` — 当前 L 级别表、链6-10 详细节
- `docs/specs/SiHankor-L5-Pipeline-Design.md` — 目标 L 级别

- [ ] **Step 1: Update overview table**

Replace the 4 rows in the overview table (lines 35-38):

```
| 6 | 知止 -> G1 Scope Boundary | L5 / L3 |
| 7 | 顺因 -> G2 Causal Alignment | L2 |
| 8 | 有度 -> G3 Proportionality | L5 / L3 |
| 9 | 损补 -> G4 Trade-off Management | L5 / L3 |
| 10 | 顺势 -> G5 Trend Alignment | L5 / L2 |
```

With:

```
| 6 | 知止 -> G1 Scope Boundary | L2 / L3 |
| 7 | 顺因 -> G2 Causal Alignment | L2 |
| 8 | 有度 -> G3 Proportionality | L2 / L3 |
| 9 | 损补 -> G4 Trade-off Management | L2 / L3 |
| 10 | 顺势 -> G5 Trend Alignment | L2 / L2 |
```

- [ ] **Step 2: Update 链6 detail section**

Replace the section starting at line ~119 (`### 链 6：知止 -> G1 Scope Boundary`).

The current content is:

```
| G1 度量管道 | L5 |
| iCT zhizhi 检查 | L3 |
```

Replace with:

```
| G1 度量管道 | L2 |
| iCT zhizhi 检查 | L3 |
```

Add after the iCT description:

```
G1 度量管道已实现：`compute_rule_density`（metrics.rs 第 XX 行）计算各 nature 的规则密度，与链 1 的产出方差（`avg_fatal_by_nature`）并列展示，为相关性检验累积数据。当前规则数不足以做统计显著性检验（仅 6 个 nature），`correlation_note` 诚实声明样本不足。

MCP 工具 `rule_density`（governance.rs 第 XX 行）已暴露查询能力。该工具查询 `count_by_nature` 和最近 100 条 `ValidationCompleted` 记录，调用 `compute_rule_density` 计算指标，格式化为人类可读文本报告返回。

偏差可量化为两项：(1) 规则密度仅反映投入维度，相关性分析需依赖链 1 数据引入传递误差；(2) 规则当前不按 nature 分配，所有 nature 共享同一规则池，`density_by_nature` 的差异仅反映文档分布。L2 而非 L1 的原因见操作化规格。
```

- [ ] **Step 3: Update 链8 detail section**

Replace the section starting at line ~140 (`### 链 8：有度 -> G3 Proportionality`).

```
| G3 度量管道 | L5 |
| iCT youdou 检查 | L3 |
```

With:

```
| G3 度量管道 | L2 |
| iCT youdou 检查 | L3 |
```

Add after the iCT description:

```
G3 度量管道已实现：`compute_rule_audit`（metrics.rs 第 XX 行）从 `RULE_REGISTRY` 聚合规则数分布（按治理域、按严格度）和 Fatal 级规则占比。纯函数，不查数据库，无外部依赖。

MCP 工具 `rule_audit`（governance.rs 第 XX 行）已暴露查询能力。该工具调用 `compute_rule_audit` 获取规则审计指标，格式化为人类可读文本报告。

偏差可量化为两项：(1) 规则审计仅反映定义侧严格度，不反映执行侧触发率；(2) 各治理域风险等级的评估依赖间接推断，未独立操作化。L2 而非 L1 的原因见操作化规格。
```

- [ ] **Step 4: Update 链9 detail section**

Replace the section starting at line ~152 (`### 链 9：损补 -> G4 Trade-off Management`).

```
| G4 文档要求 | L5 |
| iCT sunbu 检查 | L3 |
```

With:

```
| G4 文档要求 | L2 |
| iCT sunbu 检查 | L3 |
```

Add after the iCT description:

```
G4 度量管道已实现：`compute_tradeoff_coverage`（metrics.rs 第 XX 行）扫描 decision 文档内容，检测 `## 背景`/`## 决策`/`## 后果` 三个标题的非空章节，计算 ADR 覆盖率。`compute_tradeoff_coverage` 纯函数不查数据库，输入为已加载的 Document 列表。

MCP 工具 `tradeoff_coverage`（governance.rs 第 XX 行）已暴露查询能力。该工具加载全量文档，调用 `compute_tradeoff_coverage` 计算覆盖率。

`rule_changes_note` 诚实声明规则增删比率需累积 ProjectSnapshot 历史数据，当前不可计算。偏差可量化为：ADR 覆盖率仅度量"是否记录"而不度量"记录质量"。L2 而非 L1 的原因见操作化规格。
```

- [ ] **Step 5: Update 链10 detail section**

Replace the section starting at line ~162 (`### 链 10：顺势 -> G5 Trend Alignment`).

```
| G5 变更率追踪 | L5 |
| iCT shunshi 检查 | L2 |
```

With:

```
| G5 变更率追踪 | L2 |
| iCT shunshi 检查 | L2 |
```

Add after the iCT description:

```
G5 度量管道已实现：`compute_trend_alignment`（metrics.rs 第 XX 行）从 `ValidationCompleted` 和 `IndexCompleted` 记录计算审查频率-变更频率比值。比值接近 1 表示审查与变更同步，远小于 1 表示审查滞后。

MCP 工具 `trend_alignment`（governance.rs 第 XX 行）已暴露查询能力。该工具查询各 100 条两类记录，调用 `compute_trend_alignment` 计算指标。

偏差可量化为：(1) 仅覆盖时势维度，地势与人势维度未操作化；(2) 未区分"响应性审查"与"例行审查"；(3) 变更统计粒度为文档级索引事件。L2 而非 L1 的原因见操作化规格。iCT 时势维度维持 L2。
```

- [ ] **Step 6: Update file header line**

Update line 8 to reflect the changes. Current:

```
> 指标计算补全后，链 1（L4->L2）、链 5（L4->L2）的 L 级别进一步更新。
```

Replace with:

```
> 指标计算补全后，链 1/5 从 L4 升级到 L2。L5 度量管道补全后，链 6/8/9/10 从 L5 升级到 L2。
```

- [ ] **Step 7: Verify document consistency**

```bash
cd /Users/moc/projects/SiHankor/sihankor && grep -n "^| [0-9]" docs/specs/engineering/Engineering-Mapping.sih.md
```

Expected: All L 级别 values match the target table in the design spec.

- [ ] **Step 8: Commit**

```bash
cd /Users/moc/projects/SiHankor/sihankor
git add docs/specs/engineering/Engineering-Mapping.sih.md
git commit -m "docs: update engineering mapping L-levels for chains 6/8/9/10 (L5x4->L2)

链6 (知止/G1): L5 -> L2, rule_density + MCP tool rule_density
链8 (有度/G3): L5 -> L2, rule_audit + MCP tool rule_audit
链9 (损补/G4): L5 -> L2, tradeoff_coverage + MCP tool tradeoff_coverage
链10 (顺势/G5): L5 -> L2, trend_alignment + MCP tool trend_alignment

Complete L5 pipeline completion (all 4 chains upgraded).
Design spec: 260628-1800-l5-pipeline-design"
```

---

### Self-Review Checklist

After all 4 tasks are complete, verify:

1. `cargo test` — all tests pass
2. `cargo build` — no warnings
3. `git log --oneline -4` — 4 commits corresponding to 4 tasks
4. `grep -n "L5" docs/specs/engineering/Engineering-Mapping.sih.md` — only "L5 / L3" patterns remain for chains not upgraded
