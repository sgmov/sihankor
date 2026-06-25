use std::path::Path;

use super::models::{Violation, ViolationSeverity};

/// 验证域
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationDomain {
    Frontmatter,
    Structure,
    Content,
    Reference,
    Lifecycle,
    Governance,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub violations: Vec<Violation>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub const fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.violations
            .iter()
            .all(|v| v.severity != ViolationSeverity::Fatal)
    }

    pub fn has_warnings(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Guideline)
    }

    pub fn has_errors(&self) -> bool {
        self.violations
            .iter()
            .any(|v| v.severity == ViolationSeverity::Fatal)
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.violations.extend(other.violations);
    }

    /// 按 F > G > J 分级分组，产出 CI 守护结构化报告
    ///
    /// F 级：阻断项，无 F 级违规时报告为空
    /// G 级：警告项，有 G 级违规时报告列出
    /// J 级：静默项，仅计数，不列明细
    pub fn to_structured_report(&self) -> String {
        let fatal: Vec<_> = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Fatal)
            .collect();
        let guidelines: Vec<_> = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Guideline)
            .collect();
        let judgments: Vec<_> = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Judgment)
            .collect();

        let mut report = String::new();

        if !fatal.is_empty() {
            report.push_str(&format!("## F 级阻断 ({} items)\n\n", fatal.len()));
            for v in &fatal {
                report.push_str(&format!(
                    "- **{}** ({}) [{}]: {}\n",
                    v.rule_id,
                    v.dao_trace.as_deref().unwrap_or("-"),
                    v.location,
                    v.message
                ));
                if let Some(ref fix) = v.fix_suggestion {
                    report.push_str(&format!("  -> Fix: {}\n", fix));
                }
            }
            report.push('\n');
        }

        if !guidelines.is_empty() {
            report.push_str(&format!("## G 级警告 ({} items)\n\n", guidelines.len()));
            for v in &guidelines {
                report.push_str(&format!(
                    "- **{}** ({}) [{}]: {}\n",
                    v.rule_id,
                    v.dao_trace.as_deref().unwrap_or("-"),
                    v.location,
                    v.message
                ));
                if let Some(ref fix) = v.fix_suggestion {
                    report.push_str(&format!("  -> Fix: {}\n", fix));
                }
            }
            report.push('\n');
        }

        if !judgments.is_empty() {
            report.push_str(&format!(
                "## J 级静默记录 ({} items, details suppressed)\n",
                judgments.len()
            ));
        }

        if report.is_empty() {
            report.push_str("No violations found.\n");
        }

        report
    }

    /// 输出为 JSON（用于 CI 集成/机器解析）
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.violations).unwrap_or_else(|_| "[]".to_string())
    }

    /// 输出治理追溯标记（git trailer 格式），CI 可追加到 commit message
    ///
    /// 产出格式: `SiHankor-Governance: pass|blocked (F=N G=N J=N; 知止=N 顺因=N 有度=N 损补=N)`
    /// 可通过 `git log --grep="SiHankor-Governance:"` 检索
    pub fn to_governance_trailer(&self) -> String {
        let fatal = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Fatal)
            .count();
        let guidelines = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Guideline)
            .count();
        let judgments = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Judgment)
            .count();

        let mut dao_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for v in &self.violations {
            if let Some(ref dao) = v.dao_trace {
                *dao_counts.entry(dao.as_str()).or_insert(0) += 1;
            }
        }

        let status = if fatal > 0 { "blocked" } else { "pass" };
        let dao_parts: Vec<String> = ["知止", "顺因", "有度", "损补", "顺势"]
            .iter()
            .filter_map(|d| {
                let count = dao_counts.get(d).copied().unwrap_or(0);
                if count > 0 {
                    Some(format!("{}={}", d, count))
                } else {
                    None
                }
            })
            .collect();

        format!(
            "SiHankor-Governance: {} (F={} G={} J={}; {})",
            status,
            fatal,
            guidelines,
            judgments,
            dao_parts.join(" ")
        )
    }
}

/// 验证配置：控制哪些域启用
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub frontmatter: bool,
    pub structure: bool,
    pub content: bool,
    pub reference: bool,
    pub lifecycle: bool,
    pub governance: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            frontmatter: true,
            structure: true,
            content: true,
            reference: true,
            lifecycle: true,
            governance: true,
        }
    }
}

/// 验证单篇文档
pub fn validate_document(
    doc: &super::models::Document,
    file_path: Option<&Path>,
    config: &ValidationConfig,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    if config.frontmatter {
        result.merge(validate_frontmatter(doc, file_path));
    }
    if config.structure {
        result.merge(validate_structure(doc, file_path));
    }
    if config.content {
        result.merge(validate_content(doc));
    }
    if config.lifecycle {
        result.merge(validate_lifecycle(doc));
    }
    if config.governance {
        result.merge(validate_governance(doc, file_path));
    }
    // reference 域需要数据库查询，在索引阶段单独执行

    result
}

/// 域一：Frontmatter 验证
fn validate_frontmatter(
    doc: &super::models::Document,
    file_path: Option<&Path>,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // F-01: id 格式校验
    if !is_valid_id(&doc.id) {
        result.violations.push(Violation {
            rule_id: "F-01".to_string(),
            severity: ViolationSeverity::Fatal,
            message: format!(
                "id '{}' does not match format YYMMDD-HHMM[-NNN]-slug",
                doc.id
            ),
            location: "frontmatter.id".to_string(),
            fix_suggestion: Some(
                "Rename id to match YYMMDD-HHMM[-NNN]-semantic-name, e.g. 260613-1800-my-doc"
                    .to_string(),
            ),
            dao_trace: Some("知止".to_string()),
        });
    }

    // F-02: type 字段已废除——document nature 由目录路径推断
    // 此规则已移除

    // F-03: stage 必须是有效编码
    if !doc.stage.is_valid() {
        result.violations.push(Violation {
            rule_id: "F-03".to_string(),
            severity: ViolationSeverity::Fatal,
            message: format!("invalid stage: {}", doc.stage.0),
            location: "frontmatter.stage".to_string(),
            fix_suggestion: Some(
                "Set stage to one of: 1/3, 2/3, 3/3, X, or 0/<successor-id>".to_string(),
            ),
            dao_trace: Some("知止".to_string()),
        });
    }

    // F-04: upstream 必填性由 nature 决定
    // note (knowledge/notes/) 无 upstream；proposal 在 proposals/ 有 upstream
    let nature = file_path.and_then(|p| infer_nature(p));
    let upstream_required = !matches!(nature, Some("note"));
    if upstream_required && doc.upstream.is_none() {
        result.violations.push(Violation {
            rule_id: "F-04".to_string(),
            severity: ViolationSeverity::Fatal,
            message: "upstream is required for this document nature".to_string(),
            location: "frontmatter.upstream".to_string(),
            fix_suggestion: Some("Add upstream field with the id of the document that authorizes this one. For root docs, set upstream to own id.".to_string()),
            dao_trace: Some("顺因".to_string()),
        });
    }

    // G-10: upstream 自指向检查 — root docs should point to self
    // root docs 应自指向自身 id，不再使用 PHILOSOPHY 等大写域标识
    if doc.frontmatter.upstream.as_deref() == Some(&doc.id) {
        // Self-referencing: valid for root documents
    }

    result
}

/// 域二：Structure 验证
fn validate_structure(doc: &super::models::Document, file_path: Option<&Path>) -> ValidationResult {
    let mut result = ValidationResult::new();

    if let Some(path) = file_path {
        // G-02: 文档必须位于合法目录
        let path_str = path.to_string_lossy();
        let valid_dirs = [
            "specs/",
            "proposals/",
            "decisions/",
            "reference/",
            "knowledge/notes/",
        ];
        let in_valid_dir = valid_dirs.iter().any(|dir| path_str.contains(dir));

        if !in_valid_dir {
            result.violations.push(Violation {
                rule_id: "G-02".to_string(),
                severity: ViolationSeverity::Guideline,
                message: format!(
                    "document '{}' not in a recognized directory (expected: specs/, proposals/, decisions/, reference/, knowledge/notes/)",
                    doc.id
                ),
                location: path.to_string_lossy().to_string(),
                fix_suggestion: Some("Move document to the correct directory matching its nature: specs/, proposals/, decisions/, reference/, or knowledge/notes/".to_string()),
                dao_trace: Some("有度".to_string()),
            });
        }

        // G-03: 子目录深度 <= 3
        let depth = path.components().count();
        if depth > 5 {
            // root + docs/ + dir1 + dir2 + dir3 + file = 6 components max
            result.violations.push(Violation {
                rule_id: "G-03".to_string(),
                severity: ViolationSeverity::Guideline,
                message: format!(
                    "directory depth exceeds 3 levels (found {} components)",
                    depth
                ),
                location: path.to_string_lossy().to_string(),
                fix_suggestion: Some(
                    "Flatten directory structure to max 3 subdirectory levels under docs/"
                        .to_string(),
                ),
                dao_trace: Some("有度".to_string()),
            });
        }
    }

    result
}

/// 域三：Content 验证
pub fn validate_content(doc: &super::models::Document) -> ValidationResult {
    let mut result = ValidationResult::new();

    // G-04: 表格列数 <= 3
    for (line_num, line) in doc.content.lines().enumerate() {
        if line.contains('|') && line.trim().starts_with('|') {
            let col_count = line.split('|').filter(|s| !s.is_empty()).count();
            if col_count > 3 {
                result.violations.push(Violation {
                    rule_id: "G-04".to_string(),
                    severity: ViolationSeverity::Guideline,
                    message: format!("table has {} columns, maximum is 3", col_count),
                    location: format!("line {}", line_num + 1),
                    fix_suggestion: Some(
                        "Split wide table into bullet lists or subsections".to_string(),
                    ),
                    dao_trace: Some("有度".to_string()),
                });
            }
        }
    }

    // G-05: 代码块必须声明语言标签
    let mut in_code_block = false;
    for (line_num, line) in doc.content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(after_fence) = trimmed.strip_prefix("```") {
            if !in_code_block {
                in_code_block = true;
                let lang = after_fence.trim();
                if lang.is_empty() {
                    result.violations.push(Violation {
                        rule_id: "G-05".to_string(),
                        severity: ViolationSeverity::Guideline,
                        message: "code block must declare a language tag".to_string(),
                        location: format!("line {}", line_num + 1),
                        fix_suggestion: Some("Add a language tag after the opening fence: ```mermaid, ```rust, ```text".to_string()),
                        dao_trace: Some("有度".to_string()),
                    });
                }
            } else {
                in_code_block = false;
            }
        }
    }

    // F-05: 正文禁止 --- 水平线
    for (line_num, line) in doc.content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            result.violations.push(Violation {
                rule_id: "F-05".to_string(),
                severity: ViolationSeverity::Fatal,
                message: "horizontal rule (---) is forbidden in document body".to_string(),
                location: format!("line {}", line_num + 1),
                fix_suggestion: Some(
                    "Use level-2 headings (## ) for section separation instead of horizontal rules"
                        .to_string(),
                ),
                dao_trace: Some("有度".to_string()),
            });
        }
    }

    // G-06: 禁止 emoji
    for (line_num, line) in doc.content.lines().enumerate() {
        if contains_emoji(line) {
            result.violations.push(Violation {
                rule_id: "G-06".to_string(),
                severity: ViolationSeverity::Guideline,
                message: "emoji characters are forbidden".to_string(),
                location: format!("line {}", line_num + 1),
                fix_suggestion: Some("Remove emoji characters from narrative text".to_string()),
                dao_trace: Some("损补".to_string()),
            });
        }
    }

    // J-01: 列表嵌套不超过 2 层
    let max_indent = doc
        .content
        .lines()
        .filter(|l| l.trim().starts_with("- ") || l.trim().starts_with("* "))
        .map(|l| l.chars().take_while(|c| *c == ' ').count() / 2)
        .max()
        .unwrap_or(0);
    if max_indent > 2 {
        result.violations.push(Violation {
            rule_id: "J-01".to_string(),
            severity: ViolationSeverity::Judgment,
            message: format!("list nesting exceeds 2 levels (found {})", max_indent),
            location: "content".to_string(),
            fix_suggestion: Some(
                "Flatten deeply nested lists: use paragraphs or subsections instead".to_string(),
            ),
            dao_trace: Some("有度".to_string()),
        });
    }

    result
}

/// 域五：Lifecycle 验证
fn validate_lifecycle(doc: &super::models::Document) -> ValidationResult {
    let mut result = ValidationResult::new();

    // G-07: 1/3 文档不可被引用（此规则在 reference 域检查上游文档时生效）
    // 这里检查：当前文档如果是 1/3，提醒它不应被其他文档引用

    // G-08: X 文档禁止引用
    if doc.stage.0 == "X" {
        result.violations.push(Violation {
            rule_id: "G-08".to_string(),
            severity: ViolationSeverity::Guideline,
            message: "deprecated (X) document should not be referenced".to_string(),
            location: format!("document {}", doc.id),
            fix_suggestion: Some(
                "Archive or update the referencing document to point to a valid upstream"
                    .to_string(),
            ),
            dao_trace: Some("知止".to_string()),
        });
    }

    result
}

/// 域六：Governance 验证
fn validate_governance(
    doc: &super::models::Document,
    file_path: Option<&Path>,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // G-09: 2/3 和 3/3 的 decisions/ 文档应有 decided-by
    if doc.stage.0 == "2/3" || doc.stage.0 == "3/3" {
        let is_decision = file_path
            .and_then(|p| infer_nature(p))
            .map(|n| n == "decision")
            .unwrap_or(false);
        if is_decision && doc.frontmatter.decided_by.is_none() {
            result.violations.push(Violation {
                rule_id: "G-09".to_string(),
                severity: ViolationSeverity::Guideline,
                message: format!(
                    "decision document '{}' at stage {} should have decided-by field",
                    doc.id, doc.stage.0
                ),
                location: "frontmatter.decided-by".to_string(),
                fix_suggestion: Some(
                    "Add decided-by with the identifier of the person who made this decision"
                        .to_string(),
                ),
                dao_trace: Some("顺因".to_string()),
            });
        }
    }

    // F-06: ai-auto 是违例签认
    if let Some(ref decided_by) = doc.frontmatter.decided_by
        && decided_by == "ai-auto"
    {
        result.violations.push(Violation {
            rule_id: "F-06".to_string(),
            severity: ViolationSeverity::Fatal,
            message: "'ai-auto' is a forbidden decided-by value".to_string(),
            location: "frontmatter.decided-by".to_string(),
            fix_suggestion: Some(
                "Replace 'ai-auto' with a human identifier (e.g. 'moc')".to_string(),
            ),
            dao_trace: Some("知止".to_string()),
        });
    }

    // F-07: 非 decisions/ 目录文档不得有 decided-by
    if doc.frontmatter.decided_by.is_some() {
        let nature = file_path.and_then(|p| infer_nature(p));
        if nature != Some("decision") {
            result.violations.push(Violation {
                rule_id: "F-07".to_string(),
                severity: ViolationSeverity::Fatal,
                message: format!(
                    "document '{}' (nature: {}) has decided-by field, which is only allowed for decisions/",
                    doc.id,
                    nature.unwrap_or("unknown")
                ),
                location: "frontmatter.decided-by".to_string(),
                fix_suggestion: Some("Remove the decided-by field from this non-decision document".to_string()),
                dao_trace: Some("知止".to_string()),
            });
        }
    }

    result
}

/// 从文件路径推断 document nature
pub fn infer_nature(path: &Path) -> Option<&str> {
    let path_str = path.to_string_lossy();
    if path_str.contains("specs/") {
        Some("spec")
    } else if path_str.contains("proposals/") {
        Some("proposal")
    } else if path_str.contains("decisions/") {
        Some("decision")
    } else if path_str.contains("reference/") {
        Some("reference")
    } else if path_str.contains("knowledge/notes/") {
        Some("note")
    } else {
        None
    }
}

/// id 格式校验：YYMMDD-HHMM[-NNN]-语义短名
fn is_valid_id(id: &str) -> bool {
    let re = regex_pattern_for_id();
    re.is_match(id)
}

fn regex_pattern_for_id() -> regex_lite::Regex {
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    {
        regex_lite::Regex::new(r"^\d{6}-\d{4}(-\d{3})?-.+$").expect("invalid id format regex")
    }
}

/// 检测 emoji 字符
fn contains_emoji(s: &str) -> bool {
    s.chars().any(|c| {
        let cp = c as u32;
        // 常见 emoji 范围
        (0x1F600..=0x1F64F).contains(&cp)
            || (0x1F300..=0x1F5FF).contains(&cp)
            || (0x1F680..=0x1F6FF).contains(&cp)
            || (0x1F1E0..=0x1F1FF).contains(&cp)
            || (0x2600..=0x26FF).contains(&cp)
            || (0x2700..=0x27BF).contains(&cp)
            || (0xFE00..=0xFE0F).contains(&cp)
            || (0x1F900..=0x1F9FF).contains(&cp)
            || (0x1FA00..=0x1FA6F).contains(&cp)
            || (0x1FA70..=0x1FAFF).contains(&cp)
            || (0x200D).eq(&cp) // ZWJ
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{DocStatus, Frontmatter, Stage};
    use chrono::Utc;

    fn make_test_doc(
        id: &str,
        stage: &str,
        upstream: Option<&str>,
    ) -> super::super::models::Document {
        super::super::models::Document {
            id: id.to_string(),
            stage: Stage(stage.to_string()),
            title: "Test Document".to_string(),
            upstream: upstream.map(|s| s.to_string()),
            frontmatter: Frontmatter {
                id: id.to_string(),
                stage: Stage(stage.to_string()),
                upstream: upstream.map(|s| s.to_string()),
                decided_by: None,
                extra: serde_json::Value::Null,
            },
            content: "# Test\n\nBody text.".to_string(),
            status: DocStatus::Ok,
            indexed_at: Utc::now(),
            nature: String::new(),
        }
    }

    fn make_path(nature: &str) -> &Path {
        match nature {
            "spec" => Path::new("docs/specs/engineering/test.sih.md"),
            "proposal" => Path::new("docs/proposals/test.sih.md"),
            "decision" => Path::new("docs/decisions/test.sih.md"),
            "reference" => Path::new("docs/reference/test.sih.md"),
            "note" => Path::new("docs/knowledge/notes/test.sih.md"),
            _ => Path::new("docs/test.sih.md"),
        }
    }

    // ── F-01: id 格式 ──

    #[test]
    fn test_f01_valid_id_format() {
        assert!(is_valid_id("260613-1800-test-doc"));
        assert!(is_valid_id("260613-1800-001-test"));
        assert!(is_valid_id("990101-0000-x"));
        assert!(!is_valid_id("invalid-id"));
        assert!(!is_valid_id("260613-test"));
        assert!(!is_valid_id("260613-1800")); // no semantic name
    }

    // ── F-03: stage 值合法 ──

    #[test]
    fn test_f03_valid_stage_values() {
        // Use note path to avoid F-04 (upstream required for spec)
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_frontmatter(&doc, Some(make_path("note")));
        assert!(!result.has_errors()); // "1/3" is valid

        let doc = make_test_doc("260613-1800-test-x", "X", None);
        let result = validate_frontmatter(&doc, Some(make_path("note")));
        assert!(!result.has_errors()); // "X" is valid
    }

    #[test]
    fn test_f03_invalid_stage_rejected() {
        let doc = make_test_doc("260613-1800-test", "4/3", None);
        let result = validate_frontmatter(&doc, Some(make_path("spec")));
        assert!(result.has_errors());
    }

    // ── F-04: upstream 对非 note 文档必填 ──

    #[test]
    fn test_f04_spec_requires_upstream() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_frontmatter(&doc, Some(make_path("spec")));
        assert!(result.has_errors());
    }

    #[test]
    fn test_f04_note_without_upstream_ok() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_frontmatter(&doc, Some(make_path("note")));
        assert!(!result.has_errors());
    }

    // ── F-05: 禁止 body 中的 --- ──

    #[test]
    fn test_f05_no_horizontal_rule_in_body() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "# Title\n\n---\n\nSome text".to_string();
        let result = validate_content(&doc);
        assert!(result.has_errors());
    }

    #[test]
    fn test_f05_body_without_hr_ok() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_content(&doc);
        assert!(!result.has_errors());
    }

    // ── F-06: 禁止 decided-by: ai-auto ──

    #[test]
    fn test_f06_ai_auto_forbidden() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-auto".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        assert!(result.has_errors());
    }

    #[test]
    fn test_f06_ai_assist_ok() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-assist".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        // ai-assist is allowed (F-06 only blocks ai-auto)
        let fatal = result
            .violations
            .iter()
            .any(|v| v.rule_id == "F-06" && matches!(v.severity, ViolationSeverity::Fatal));
        assert!(!fatal);
    }

    // ── F-07: 非 decisions/ 文档禁止 decided-by ──

    #[test]
    fn test_f07_non_decision_must_not_have_decided_by() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-assist".to_string());
        let result = validate_governance(&doc, Some(make_path("spec")));
        assert!(result.has_errors());
    }

    #[test]
    fn test_f07_decision_with_decided_by_ok() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-assist".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        let err = result.violations.iter().any(|v| v.rule_id == "F-07");
        assert!(!err);
    }

    // ── G-02: 文档在可识别目录下 ──

    #[test]
    fn test_g02_recognized_directory() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/specs/philosophy/test.sih.md")));
        let g02 = result.violations.iter().find(|v| v.rule_id == "G-02");
        assert!(g02.is_none()); // no G-02 violation for recognized dir
    }

    #[test]
    fn test_g02_unrecognized_directory_warns() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/unknown/test.sih.md")));
        let g02 = result.violations.iter().find(|v| v.rule_id == "G-02");
        assert!(g02.is_some());
    }

    // ── G-03: 路径深度 ≤ 3 ──

    #[test]
    fn test_g03_path_depth_ok() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/specs/test.sih.md")));
        let g03 = result.violations.iter().find(|v| v.rule_id == "G-03");
        assert!(g03.is_none());
    }

    #[test]
    fn test_g03_path_too_deep_warns() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/a/b/c/d/test.sih.md")));
        let g03 = result.violations.iter().find(|v| v.rule_id == "G-03");
        assert!(g03.is_some());
    }

    // ── G-04: 表格 ≤ 3 列 ──

    #[test]
    fn test_g04_table_cols_ok() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "| a | b | c |\n|---|---|---|\n| 1 | 2 | 3 |".to_string();
        let result = validate_content(&doc);
        let g04 = result.violations.iter().find(|v| v.rule_id == "G-04");
        assert!(g04.is_none());
    }

    #[test]
    fn test_g04_table_too_wide() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "| a | b | c | d |\n|---|---|---|---|\n| 1 | 2 | 3 | 4 |".to_string();
        let result = validate_content(&doc);
        let g04 = result.violations.iter().find(|v| v.rule_id == "G-04");
        assert!(g04.is_some());
    }

    // ── G-05: 代码块语言标签 ──

    #[test]
    fn test_g05_code_block_with_lang() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "```rust\nlet x = 1;\n```".to_string();
        let result = validate_content(&doc);
        let g05 = result.violations.iter().find(|v| v.rule_id == "G-05");
        assert!(g05.is_none());
    }

    #[test]
    fn test_g05_code_block_no_lang() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "```\nsome code\n```".to_string();
        let result = validate_content(&doc);
        let g05 = result.violations.iter().find(|v| v.rule_id == "G-05");
        assert!(g05.is_some());
    }

    // ── G-06: 无 emoji ──

    #[test]
    fn test_g06_no_emoji() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_content(&doc);
        let g06 = result.violations.iter().find(|v| v.rule_id == "G-06");
        assert!(g06.is_none());
    }

    #[test]
    fn test_g06_emoji_detected() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "Hello 👋 world".to_string();
        let result = validate_content(&doc);
        let g06 = result.violations.iter().find(|v| v.rule_id == "G-06");
        assert!(g06.is_some());
    }

    // ── G-08: Stage X 标记 ──

    #[test]
    fn test_g08_stage_x_warns() {
        let doc = make_test_doc("260613-1800-test", "X", Some("260613-1700-x"));
        let result = validate_lifecycle(&doc);
        let g08 = result.violations.iter().find(|v| v.rule_id == "G-08");
        assert!(g08.is_some());
    }

    #[test]
    fn test_g08_stage_3_ok() {
        let doc = make_test_doc("260613-1800-test", "3/3", Some("260613-1700-x"));
        let result = validate_lifecycle(&doc);
        let g08 = result.violations.iter().find(|v| v.rule_id == "G-08");
        assert!(g08.is_none());
    }

    // ── G-09: decisions/ 2/3+ 应有 decided-by ──

    #[test]
    fn test_g09_decision_without_decided_by_warns() {
        let doc = make_test_doc("260613-1800-test", "3/3", Some("260613-1700-x"));
        let result = validate_governance(&doc, Some(make_path("decision")));
        let g09 = result.violations.iter().find(|v| v.rule_id == "G-09");
        assert!(g09.is_some());
    }

    #[test]
    fn test_g09_decision_with_decided_by_ok() {
        let mut doc = make_test_doc("260613-1800-test", "3/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-assist".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        let g09 = result.violations.iter().find(|v| v.rule_id == "G-09");
        assert!(g09.is_none());
    }

    // ── G-10: 根文档自指向 ──

    #[test]
    fn test_g10_root_self_reference_ok() {
        let path = Path::new("docs/specs/philosophy/On-SiHankor.sih.md");
        let doc = make_test_doc(
            "240602-0900-on-sihankor",
            "3/3",
            Some("240602-0900-on-sihankor"),
        );
        // G-10 is silent: no violation emitted for self-reference
        let result = validate_governance(&doc, Some(path));
        let violations = result
            .violations
            .iter()
            .filter(|v| v.rule_id == "G-10")
            .count();
        assert_eq!(violations, 0); // G-10 should not emit violations, just validate internally
    }

    // ── J-01: 列表嵌套 ≤ 2 层 ──

    #[test]
    fn test_j01_nested_list_ok() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "- item\n  - sub\n    - subsub".to_string();
        let result = validate_content(&doc);
        let j01 = result.violations.iter().find(|v| v.rule_id == "J-01");
        assert!(j01.is_none());
    }

    #[test]
    fn test_j01_list_too_deep() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "- a\n  - b\n    - c\n      - d".to_string();
        let result = validate_content(&doc);
        let j01 = result.violations.iter().find(|v| v.rule_id == "J-01");
        assert!(j01.is_some());
    }

    // ── Integration: full validate_document ──

    #[test]
    fn test_full_validation_clean_doc() {
        let doc = make_test_doc("260613-1800-test-doc", "2/3", Some("260613-1700-upstream"));
        let result = validate_document(&doc, Some(make_path("spec")), &ValidationConfig::default());
        // A clean, well-formed document should have no Fatal errors and may have guidelines
        assert!(!result.has_errors());
    }

    #[test]
    fn test_full_validation_broken_doc() {
        let mut doc = make_test_doc("bad-id", "bad-stage", None);
        doc.content = "---\nbroken".to_string();
        let result = validate_document(&doc, Some(make_path("spec")), &ValidationConfig::default());
        assert!(result.has_errors());
    }
}
