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

impl ValidationResult {
    pub fn new() -> Self {
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
fn validate_frontmatter(doc: &super::models::Document, file_path: Option<&Path>) -> ValidationResult {
    let mut result = ValidationResult::new();

    // F-01: id 格式校验
    if !is_valid_id(&doc.id) {
        result.violations.push(Violation {
            rule_id: "F-01".to_string(),
            severity: ViolationSeverity::Fatal,
            message: format!("id '{}' does not match format YYMMDD-HHMM[-NNN]-slug", doc.id),
            location: "frontmatter.id".to_string(),
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
        });
    }

    // F-04: upstream 必填性由 nature 决定
    // note (knowledge/notes/) 无 upstream；proposal 在 proposals/ 有 upstream
    let nature = file_path.and_then(|p| infer_nature(p));
    let upstream_required = match nature {
        Some("note") => false,
        _ => true,
    };
    if upstream_required && doc.upstream.is_none() {
        result.violations.push(Violation {
            rule_id: "F-04".to_string(),
            severity: ViolationSeverity::Fatal,
            message: "upstream is required for this document nature".to_string(),
            location: "frontmatter.upstream".to_string(),
        });
    }

    // G-01: upstream 全大写域标识检查已移除
    // root docs 应自指向自身 id，不再使用 PHILOSOPHY 等大写域标识

    result
}

/// 域二：Structure 验证
fn validate_structure(doc: &super::models::Document, file_path: Option<&Path>) -> ValidationResult {
    let mut result = ValidationResult::new();

    if let Some(path) = file_path {
        // G-02: 文档必须位于合法目录
        let path_str = path.to_string_lossy();
        let valid_dirs = ["specs/", "proposals/", "decisions/", "reference/", "knowledge/notes/"];
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
            });
        }

        // G-03: 子目录深度 <= 3
        let depth = path.components().count();
        if depth > 5 {
            // root + docs/ + dir1 + dir2 + dir3 + file = 6 components max
            result.violations.push(Violation {
                rule_id: "G-03".to_string(),
                severity: ViolationSeverity::Guideline,
                message: format!("directory depth exceeds 3 levels (found {} components)", depth),
                location: path.to_string_lossy().to_string(),
            });
        }
    }

    result
}

/// 域三：Content 验证
fn validate_content(doc: &super::models::Document) -> ValidationResult {
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
                });
            }
        }
    }

    // G-05: 代码块必须声明语言标签
    let mut in_code_block = false;
    for (line_num, line) in doc.content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                let lang = trimmed[3..].trim();
                if lang.is_empty() {
                    result.violations.push(Violation {
                        rule_id: "G-05".to_string(),
                        severity: ViolationSeverity::Guideline,
                        message: "code block must declare a language tag".to_string(),
                        location: format!("line {}", line_num + 1),
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
        });
    }

    result
}

/// 域六：Governance 验证
fn validate_governance(doc: &super::models::Document, file_path: Option<&Path>) -> ValidationResult {
    let mut result = ValidationResult::new();

    // G-09: 2/3 和 3/3 的 decisions/ 文档应有 decided-by
    if doc.stage.0 == "2/3" || doc.stage.0 == "3/3" {
        let is_decision = file_path
            .and_then(|p| infer_nature(p))
            .map(|n| n == "decision")
            .unwrap_or(false);
        if is_decision {
            if doc.frontmatter.decided_by.is_none() {
                result.violations.push(Violation {
                    rule_id: "G-09".to_string(),
                    severity: ViolationSeverity::Guideline,
                    message: format!(
                        "decision document '{}' at stage {} should have decided-by field",
                        doc.id, doc.stage.0
                    ),
                    location: "frontmatter.decided-by".to_string(),
                });
            }
        }
    }

    // F-06: ai-auto 是违例签认
    if let Some(ref decided_by) = doc.frontmatter.decided_by {
        if decided_by == "ai-auto" {
            result.violations.push(Violation {
                rule_id: "F-06".to_string(),
                severity: ViolationSeverity::Fatal,
                message: "'ai-auto' is a forbidden decided-by value".to_string(),
                location: "frontmatter.decided-by".to_string(),
            });
        }
    }

    result
}

/// 从文件路径推断 document nature
fn infer_nature(path: &Path) -> Option<&str> {
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
    regex_lite::Regex::new(r"^\d{6}-\d{4}(-\d{3})?-.+$").unwrap()
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

    fn make_test_doc(id: &str, stage: &str, upstream: Option<&str>) -> super::super::models::Document {
        super::super::models::Document {
            id: id.to_string(),
            stage: Stage(stage.to_string()),
            title: "Test".to_string(),
            upstream: upstream.map(|s| s.to_string()),
            frontmatter: Frontmatter {
                id: id.to_string(),
                stage: Stage(stage.to_string()),
                upstream: upstream.map(|s| s.to_string()),
                decided_by: None,
                extra: serde_json::Value::Null,
            },
            content: "# Test\nBody text".to_string(),
            status: DocStatus::Ok,
            indexed_at: Utc::now(),
        }
    }

    #[test]
    fn test_valid_id_format() {
        assert!(is_valid_id("260613-1800-test-doc"));
        assert!(is_valid_id("260613-1800-001-test"));
        assert!(!is_valid_id("invalid-id"));
        assert!(!is_valid_id("260613-test"));
    }

    #[test]
    fn test_frontmatter_missing_upstream_for_spec() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        // Without path context, upstream is required by default
        let result = validate_frontmatter(&doc, None);
        assert!(result.has_errors());
    }

    #[test]
    fn test_frontmatter_note_no_upstream_ok() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        // With knowledge/notes/ path, upstream not required
        let path = std::path::Path::new("docs/knowledge/notes/test.sih.md");
        let result = validate_frontmatter(&doc, Some(path));
        assert!(!result.has_errors());
    }

    #[test]
    fn test_content_forbidden_horizontal_rule() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "# Title\n\n---\n\nSome text".to_string();
        let result = validate_content(&doc);
        assert!(result.has_errors());
    }

    #[test]
    fn test_governance_ai_auto_forbidden() {
        // decisions/ document with ai-auto decided-by should flag
        let path = std::path::Path::new("docs/decisions/test.sih.md");
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-upstream"));
        doc.frontmatter.decided_by = Some("ai-auto".to_string());
        let result = validate_governance(&doc, Some(path));
        assert!(result.has_errors());
    }
}
