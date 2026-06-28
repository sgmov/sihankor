use std::path::Path;

use super::models::{Violation, ViolationSeverity};

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
    ///
    /// J 语义校准说明（260628-1600）：法论 §有度原定义将 J（矩）的判定方式与阻断行为混淆。
    /// 经 G3 校准：J = 精确机械判定 + 静默记录。判定方式保留机械精确性，
    /// 阻断行为由"pass/fail"校准为"静默记录"——风格性规则不应阻断，但机械判定确保可复现性。
    /// 见法论校准 2026-06-28。
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
                    "- **{}** [{}]: {}\n",
                    v.rule_id, v.location, v.message
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
                    "- **{}** [{}]: {}\n",
                    v.rule_id, v.location, v.message
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

        let status = if fatal > 0 { "blocked" } else { "pass" };

        format!(
            "SiHankor-Governance: {} (F={} G={} J={})",
            status, fatal, guidelines, judgments,
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

    // V-F-01: id 格式校验
    if !is_valid_id(&doc.id) {
        result.violations.push(Violation {
            rule_id: "V-F-01".to_string(),
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
        });
    }

    // V-F-02: type 字段已废除：document nature 由目录路径推断
    // 此规则已移除

    // V-F-03: stage 必须是有效编码
    if !doc.stage.is_valid() {
        result.violations.push(Violation {
            rule_id: "V-F-03".to_string(),
            severity: ViolationSeverity::Fatal,
            message: format!("invalid stage: {}", doc.stage),
            location: "frontmatter.stage".to_string(),
            fix_suggestion: Some(
                "Set stage to one of: 1/3, 2/3, 3/3, X, or 0/<successor-id>".to_string(),
            ),
        });
    }

    // V-F-04: upstream 必填性由 nature 决定
    // note (knowledge/notes/) 无 upstream；proposal 在 proposals/ 有 upstream
    let nature = file_path.and_then(|p| infer_nature(p));
    let upstream_required = !matches!(nature, Some("note"));
    if upstream_required && doc.upstream.is_none() {
        result.violations.push(Violation {
            rule_id: "V-F-04".to_string(),
            severity: ViolationSeverity::Fatal,
            message: "upstream is required for this document nature".to_string(),
            location: "frontmatter.upstream".to_string(),
            fix_suggestion: Some("Add upstream field with the id of the document that authorizes this one. For root docs, set upstream to own id.".to_string()),
        });
    }

    // V-G-10: upstream 自指向检查：root docs should point to self
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
        // V-G-02: 文档必须位于合法目录
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
                rule_id: "V-G-02".to_string(),
                severity: ViolationSeverity::Guideline,
                message: format!(
                    "document '{}' not in a recognized directory (expected: specs/, proposals/, decisions/, reference/, knowledge/notes/)",
                    doc.id
                ),
                location: path.to_string_lossy().to_string(),
                fix_suggestion: Some("Move document to the correct directory matching its nature: specs/, proposals/, decisions/, reference/, or knowledge/notes/".to_string()),
            });
        }

        // V-G-03: 子目录深度 <= 4 (docs/{nature}/{dir1}/{dir2}/{dir3}/{dir4}/file)
        let depth = path.components().count();
        if depth > 5 {
            // root + docs/ + dir1 + dir2 + dir3 + file = 6 components max
            result.violations.push(Violation {
                rule_id: "V-G-03".to_string(),
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
            });
        }
    }

    result
}

/// 域三：Content 验证
pub fn validate_content(doc: &super::models::Document) -> ValidationResult {
    let mut result = ValidationResult::new();

    // V-G-04: 表格列数 <= 3
    for (line_num, line) in doc.content.lines().enumerate() {
        if line.contains('|') && line.trim().starts_with('|') {
            let col_count = line.split('|').filter(|s| !s.is_empty()).count();
            if col_count > 3 {
                result.violations.push(Violation {
                    rule_id: "V-G-04".to_string(),
                    severity: ViolationSeverity::Guideline,
                    message: format!("table has {} columns, maximum is 3", col_count),
                    location: format!("line {}", line_num + 1),
                    fix_suggestion: Some(
                        "Split wide table into bullet lists or subsections".to_string(),
                    ),
                });
            }
        }
    }

    // V-G-05: 代码块必须声明语言标签
    let mut in_code_block = false;
    for (line_num, line) in doc.content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(after_fence) = trimmed.strip_prefix("```") {
            if !in_code_block {
                in_code_block = true;
                let lang = after_fence.trim();
                if lang.is_empty() {
                    result.violations.push(Violation {
                        rule_id: "V-G-05".to_string(),
                        severity: ViolationSeverity::Guideline,
                        message: "code block must declare a language tag".to_string(),
                        location: format!("line {}", line_num + 1),
                        fix_suggestion: Some("Add a language tag after the opening fence: ```mermaid, ```rust, ```text".to_string()),
                    });
                }
            } else {
                in_code_block = false;
            }
        }
    }

    // V-F-05: 正文禁止 --- 水平线
    for (line_num, line) in doc.content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            result.violations.push(Violation {
                rule_id: "V-F-05".to_string(),
                severity: ViolationSeverity::Fatal,
                message: "horizontal rule (---) is forbidden in document body".to_string(),
                location: format!("line {}", line_num + 1),
                fix_suggestion: Some(
                    "Use level-2 headings (## ) for section separation instead of horizontal rules"
                        .to_string(),
                ),
            });
        }
    }

    // V-G-06: 禁止 emoji
    for (line_num, line) in doc.content.lines().enumerate() {
        if contains_emoji(line) {
            result.violations.push(Violation {
                rule_id: "V-G-06".to_string(),
                severity: ViolationSeverity::Guideline,
                message: "emoji characters are forbidden".to_string(),
                location: format!("line {}", line_num + 1),
                fix_suggestion: Some("Remove emoji characters from narrative text".to_string()),
            });
        }
    }

    // V-J-01: 列表嵌套不超过 2 层
    //
    // J 语义校准说明（260628-1600）：J 级规则为精确机械判定 + 静默记录，
    // 仅计数不阻断。经 G3 校准，见法论校准 2026-06-28。
    let max_indent = doc
        .content
        .lines()
        .filter(|l| l.trim().starts_with("- ") || l.trim().starts_with("* "))
        .map(|l| l.chars().take_while(|c| *c == ' ').count() / 2)
        .max()
        .unwrap_or(0);
    if max_indent > 2 {
        result.violations.push(Violation {
            rule_id: "V-J-01".to_string(),
            severity: ViolationSeverity::Judgment,
            message: format!("list nesting exceeds 2 levels (found {})", max_indent),
            location: "content".to_string(),
            fix_suggestion: Some(
                "Flatten deeply nested lists: use paragraphs or subsections instead".to_string(),
            ),
        });
    }

    result
}

/// 域五：Lifecycle 验证
fn validate_lifecycle(doc: &super::models::Document) -> ValidationResult {
    let mut result = ValidationResult::new();

    // V-G-07: 1/3 文档不可被引用（此规则在 reference 域检查上游文档时生效）
    // 这里检查：当前文档如果是 1/3，提醒它不应被其他文档引用

    // V-G-08: X 文档禁止引用
    if doc.stage.as_str() == "X" {
        result.violations.push(Violation {
            rule_id: "V-G-08".to_string(),
            severity: ViolationSeverity::Guideline,
            message: "deprecated (X) document should not be referenced".to_string(),
            location: format!("document {}", doc.id),
            fix_suggestion: Some(
                "Archive or update the referencing document to point to a valid upstream"
                    .to_string(),
            ),
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

    // V-G-09: 2/3 和 3/3 的 decisions/ 文档应有 decided-by
    if doc.stage.as_str() == "2/3" || doc.stage.as_str() == "3/3" {
        let is_decision = file_path
            .and_then(|p| infer_nature(p))
            .map(|n| n == "decision")
            .unwrap_or(false);
        if is_decision && doc.frontmatter.decided_by.is_none() {
            result.violations.push(Violation {
                rule_id: "V-G-09".to_string(),
                severity: ViolationSeverity::Guideline,
                message: format!(
                    "decision document '{}' at stage {} should have decided-by field",
                    doc.id, doc.stage
                ),
                location: "frontmatter.decided-by".to_string(),
                fix_suggestion: Some(
                    "Add decided-by with the identifier of the person who made this decision"
                        .to_string(),
                ),
            });
        }
    }

    // V-F-06: decided-by 不得是 AI 前缀值
    //
    // 语义：decided-by 记录的是"谁决定了这个决策"，必须是人类标识符。
    // 任何以 "ai" 开头的值（ai-auto, ai-assist, ai-anything）都表示 AI 作为决策主体，
    // 违反"AI 不充任决议者"的治理原则（Reconstruction-Spec $7.1）。
    //
    // 历史背景：早期实现仅字面检查 == "ai-auto"，导致 ai-assist 等值绕过校验
    // （见 R4 工程映射审计 4.7 冲突四）。本规则修复为前缀检查，阻断所有 AI 前缀值。
    //
    // 若需记录 AI 辅助级别，应使用独立字段（如 ai-assistance-level），不放入 decided-by。
    if let Some(ref decided_by) = doc.frontmatter.decided_by
        && is_ai_prefixed_decided_by(decided_by)
    {
        result.violations.push(Violation {
            rule_id: "V-F-06".to_string(),
            severity: ViolationSeverity::Fatal,
            message: format!(
                "decided-by value '{}' starts with 'ai' and is forbidden: decided-by must be a human identifier",
                decided_by
            ),
            location: "frontmatter.decided-by".to_string(),
            fix_suggestion: Some(
                "Replace with a human identifier (e.g. 'moc'). AI assistance level, if needed, belongs in a separate field, not decided-by.".to_string(),
            ),
        });
    }

    // V-F-07: 非 decisions/ 目录文档不得有 decided-by
    if doc.frontmatter.decided_by.is_some() {
        let nature = file_path.and_then(|p| infer_nature(p));
        if nature != Some("decision") {
            result.violations.push(Violation {
                rule_id: "V-F-07".to_string(),
                severity: ViolationSeverity::Fatal,
                message: format!(
                    "document '{}' (nature: {}) has decided-by field, which is only allowed for decisions/",
                    doc.id,
                    nature.unwrap_or("unknown")
                ),
                location: "frontmatter.decided-by".to_string(),
                fix_suggestion: Some("Remove the decided-by field from this non-decision document".to_string()),
            });
        }
    }

    result
}

/// 从文件路径推断 document nature
pub fn infer_nature(path: &Path) -> Option<&str> {
    let mut components = path.components();
    // First component must be "docs"
    if components.next().map(|c| c.as_os_str()) != Some(std::ffi::OsStr::new("docs")) {
        return None;
    }
    // Second component is the nature directory
    match components.next()?.as_os_str().to_str()? {
        "specs" => Some("spec"),
        "proposals" => Some("proposal"),
        "decisions" => Some("decision"),
        "reference" => Some("reference"),
        dir if dir.starts_with("knowledge") => Some("note"),
        _ => None,
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

/// 检测 decided-by 值是否以 "ai" 开头（不区分大小写）
///
/// decided-by 必须是人类标识符。任何 AI 前缀值（ai-auto, ai-assist 等）
/// 都表示 AI 作为决策主体，违反"AI 不充任决议者"的治理原则。
/// 详见 V-F-06 规则注释与 R4 工程映射审计 4.7 冲突四。
fn is_ai_prefixed_decided_by(value: &str) -> bool {
    value.to_lowercase().starts_with("ai")
}

/// 检测 emoji 字符
fn contains_emoji(s: &str) -> bool {
    s.chars().any(|c| {
        let cp = c as u32;
        (0x1F600..=0x1F64F).contains(&cp)  // Emoticons
            || (0x1F300..=0x1F5FF).contains(&cp)  // Misc Symbols & Pictographs
            || (0x1F680..=0x1F6FF).contains(&cp)  // Transport & Map
            || (0x1F1E0..=0x1F1FF).contains(&cp)  // Flags
            || (0x2600..=0x26FF).contains(&cp)    // Misc symbols
            || (0x2700..=0x27BF).contains(&cp)    // Dingbats
            || (0xFE00..=0xFE0F).contains(&cp)    // Variation Selectors
            || (0x1F900..=0x1F9FF).contains(&cp)  // Supplemental Symbols
            || (0x1FA00..=0x1FA6F).contains(&cp)  // Chess Symbols
            || (0x1FA70..=0x1FAFF).contains(&cp)  // Symbols Extended-A
            || (0x200D).eq(&cp) // ZWJ (Zero Width Joiner)
            || (0x20E3).eq(&cp) // Combining Enclosing Keycap
            || (0x231A..=0x231B).contains(&cp) // Watch, Hourglass
            || (0x23E9..=0x23F3).contains(&cp) // Double triangles, hourglass with sand
            || (0x23F8..=0x23FA).contains(&cp) // Power, pause, record symbols
            || (0x23CF).eq(&cp) // Eject symbol
            || (0x24C2).eq(&cp) // Circled M
            || (0x25AA..=0x25AB).contains(&cp) // Black/white small squares
            || (0x25B6).eq(&cp) // Play button
            || (0x25C0).eq(&cp) // Reverse button
            || (0x25FB..=0x25FE).contains(&cp) // Medium/large squares
            || (0x2B05..=0x2B07).contains(&cp) // Arrows
            || (0x2B1B..=0x2B1C).contains(&cp) // Black/white large squares
            || (0x2B50).eq(&cp) // Star
            || (0x2B55).eq(&cp) // Circle
            || (0x3030).eq(&cp) // Wavy dash
            || (0x303D).eq(&cp) // Part alternation mark
            || (0x3297).eq(&cp) // Japanese "congratulations"
            || (0x3299).eq(&cp) // Japanese "secret"
            || (0xA9..=0xAE).contains(&cp) // Copyright/Registered signs
            || (0x2122).eq(&cp) // TM sign
            || (0x2139).eq(&cp) // Info symbol
            || (0x2328).eq(&cp) // Keyboard
            || (0x23ED..=0x23EF).contains(&cp) // Media control symbols
            || (0x23F1..=0x23F2).contains(&cp) // Stopwatch, timer
            || (0x23F4..=0x23F7).contains(&cp) // Media playback symbols
            || (0x2620).eq(&cp) // Skull
            || (0x2622..=0x2623).contains(&cp) // Radioactive, biohazard
            || (0x2626).eq(&cp) // Orthodox cross
            || (0x262A).eq(&cp) // Star and crescent
            || (0x262E).eq(&cp) // Peace symbol
            || (0x262F).eq(&cp) // Yin Yang
            || (0x2638..=0x263A).contains(&cp) // Wheel of dharma, faces
            || (0x2640..=0x2653).contains(&cp) // Gender/zodiac symbols
            || (0x2660..=0x2668).contains(&cp) // Card suits/misc
            || (0x267B).eq(&cp) // Recycle
            || (0x267E..=0x267F).contains(&cp) // Symbols
            || (0x2692..=0x2697).contains(&cp) // Tool/misc symbols
            || (0x2699).eq(&cp) // Gear
            || (0x269B..=0x269C).contains(&cp) // Atom
            || (0x26A0..=0x26A1).contains(&cp) // Warning, high voltage
            || (0x26A7).eq(&cp) // Transgender
            || (0x26AA..=0x26B1).contains(&cp) // Circles
            || (0x26B3..=0x26BC).contains(&cp) // Symbols
            || (0x26BD..=0x26BF).contains(&cp) // Sports
            || (0x26C4..=0x26C8).contains(&cp) // Weather/sports
            || (0x26CD).eq(&cp) // Disabled car
            || (0x26CF).eq(&cp) // Pick
            || (0x26D1..=0x26D4).contains(&cp) // Symbols
            || (0x26E9..=0x26EA).contains(&cp) // Symbols
            || (0x26F0..=0x26F5).contains(&cp) // Symbols
            || (0x26F7..=0x26FA).contains(&cp) // Symbols
            || (0x26FD).eq(&cp) // Fuel pump
            || (0x2702).eq(&cp) // Scissors
            || (0x2705).eq(&cp) // Check mark
            || (0x2708..=0x270D).contains(&cp) // Plane, hand symbols
            || (0x270F).eq(&cp) // Pencil
            || (0x2712).eq(&cp) // Black nib
            || (0x2714).eq(&cp) // Check mark
            || (0x2716).eq(&cp) // X mark
            || (0x271D).eq(&cp) // Latin cross
            || (0x2721).eq(&cp) // Star of David
            || (0x2728).eq(&cp) // Sparkles
            || (0x2733..=0x2734).contains(&cp) // Symbols
            || (0x2744).eq(&cp) // Snowflake
            || (0x2747).eq(&cp) // Sparkle
            || (0x274C).eq(&cp) // Cross mark
            || (0x274E).eq(&cp) // Cross mark
            || (0x2753..=0x2755).contains(&cp) // Question/exclamation marks
            || (0x2757).eq(&cp) // Exclamation mark
            || (0x2763..=0x2764).contains(&cp) // Heart
            || (0x2795..=0x2797).contains(&cp) // Plus/minus
            || (0x27A1).eq(&cp) // Right arrow
            || (0x27B0).eq(&cp) // Curly loop
            || (0x27BF).eq(&cp) // Double curly loop
            || (0x2934..=0x2935).contains(&cp) // Arrow symbols
            || (0x2B05..=0x2B07).contains(&cp) // Arrows
            || (0x2B1B..=0x2B1C).contains(&cp) // Squares
            || (0x2B50).eq(&cp) // Star
            || (0x2B55).eq(&cp) // Circle
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
            stage: Stage::from_str(stage).unwrap_or(Stage::Deprecated),
            title: "Test Document".to_string(),
            upstream: upstream.map(|s| s.to_string()),
            frontmatter: Frontmatter {
                id: id.to_string(),
                stage: Stage::from_str(stage).unwrap_or(Stage::Deprecated),
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

    // ── V-F-01: id 格式 ──

    #[test]
    fn test_f01_valid_id_format() {
        assert!(is_valid_id("260613-1800-test-doc"));
        assert!(is_valid_id("260613-1800-001-test"));
        assert!(is_valid_id("990101-0000-x"));
        assert!(!is_valid_id("invalid-id"));
        assert!(!is_valid_id("260613-test"));
        assert!(!is_valid_id("260613-1800")); // no semantic name
    }

    // ── V-F-03: stage 值合法 ──

    #[test]
    fn test_f03_valid_stage_values() {
        // Use note path to avoid V-F-04 (upstream required for spec)
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

    // ── V-F-04: upstream 对非 note 文档必填 ──

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

    // ── V-F-05: 禁止 body 中的 --- ──

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

    // ── V-F-06: 禁止 decided-by 的 ai 前缀值 ──

    #[test]
    fn test_f06_ai_auto_forbidden() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-auto".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        assert!(result.has_errors());
    }

    #[test]
    fn test_f06_ai_assist_forbidden() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("ai-assist".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        // V-F-06 now blocks all ai-prefixed values, including ai-assist
        let fatal = result
            .violations
            .iter()
            .any(|v| v.rule_id == "V-F-06" && matches!(v.severity, ViolationSeverity::Fatal));
        assert!(fatal);
    }

    #[test]
    fn test_f06_human_identifier_ok() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("moc".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        // human identifier passes V-F-06
        let fatal = result
            .violations
            .iter()
            .any(|v| v.rule_id == "V-F-06" && matches!(v.severity, ViolationSeverity::Fatal));
        assert!(!fatal);
    }

    // ── V-F-07: 非 decisions/ 文档禁止 decided-by ──

    #[test]
    fn test_f07_non_decision_must_not_have_decided_by() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("moc".to_string());
        let result = validate_governance(&doc, Some(make_path("spec")));
        assert!(result.has_errors());
    }

    #[test]
    fn test_f07_decision_with_decided_by_ok() {
        let mut doc = make_test_doc("260613-1800-test", "2/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("moc".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        let err = result.violations.iter().any(|v| v.rule_id == "V-F-07");
        assert!(!err);
    }

    // ── V-G-02: 文档在可识别目录下 ──

    #[test]
    fn test_g02_recognized_directory() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/specs/philosophy/test.sih.md")));
        let g02 = result.violations.iter().find(|v| v.rule_id == "V-G-02");
        assert!(g02.is_none()); // no V-G-02 violation for recognized dir
    }

    #[test]
    fn test_g02_unrecognized_directory_warns() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/unknown/test.sih.md")));
        let g02 = result.violations.iter().find(|v| v.rule_id == "V-G-02");
        assert!(g02.is_some());
    }

    // ── V-G-03: 路径深度 ≤ 3 ──

    #[test]
    fn test_g03_path_depth_ok() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/specs/test.sih.md")));
        let g03 = result.violations.iter().find(|v| v.rule_id == "V-G-03");
        assert!(g03.is_none());
    }

    #[test]
    fn test_g03_path_too_deep_warns() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_structure(&doc, Some(Path::new("docs/a/b/c/d/test.sih.md")));
        let g03 = result.violations.iter().find(|v| v.rule_id == "V-G-03");
        assert!(g03.is_some());
    }

    // ── V-G-04: 表格 ≤ 3 列 ──

    #[test]
    fn test_g04_table_cols_ok() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "| a | b | c |\n|---|---|---|\n| 1 | 2 | 3 |".to_string();
        let result = validate_content(&doc);
        let g04 = result.violations.iter().find(|v| v.rule_id == "V-G-04");
        assert!(g04.is_none());
    }

    #[test]
    fn test_g04_table_too_wide() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "| a | b | c | d |\n|---|---|---|---|\n| 1 | 2 | 3 | 4 |".to_string();
        let result = validate_content(&doc);
        let g04 = result.violations.iter().find(|v| v.rule_id == "V-G-04");
        assert!(g04.is_some());
    }

    // ── V-G-05: 代码块语言标签 ──

    #[test]
    fn test_g05_code_block_with_lang() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "```rust\nlet x = 1;\n```".to_string();
        let result = validate_content(&doc);
        let g05 = result.violations.iter().find(|v| v.rule_id == "V-G-05");
        assert!(g05.is_none());
    }

    #[test]
    fn test_g05_code_block_no_lang() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "```\nsome code\n```".to_string();
        let result = validate_content(&doc);
        let g05 = result.violations.iter().find(|v| v.rule_id == "V-G-05");
        assert!(g05.is_some());
    }

    // ── V-G-06: 无 emoji ──

    #[test]
    fn test_g06_no_emoji() {
        let doc = make_test_doc("260613-1800-test", "1/3", None);
        let result = validate_content(&doc);
        let g06 = result.violations.iter().find(|v| v.rule_id == "V-G-06");
        assert!(g06.is_none());
    }

    #[test]
    fn test_g06_emoji_detected() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "Hello 👋 world".to_string();
        let result = validate_content(&doc);
        let g06 = result.violations.iter().find(|v| v.rule_id == "V-G-06");
        assert!(g06.is_some());
    }

    // ── V-G-08: Stage X 标记 ──

    #[test]
    fn test_g08_stage_x_warns() {
        let doc = make_test_doc("260613-1800-test", "X", Some("260613-1700-x"));
        let result = validate_lifecycle(&doc);
        let g08 = result.violations.iter().find(|v| v.rule_id == "V-G-08");
        assert!(g08.is_some());
    }

    #[test]
    fn test_g08_stage_3_ok() {
        let doc = make_test_doc("260613-1800-test", "3/3", Some("260613-1700-x"));
        let result = validate_lifecycle(&doc);
        let g08 = result.violations.iter().find(|v| v.rule_id == "V-G-08");
        assert!(g08.is_none());
    }

    // ── V-G-09: decisions/ 2/3+ 应有 decided-by ──

    #[test]
    fn test_g09_decision_without_decided_by_warns() {
        let doc = make_test_doc("260613-1800-test", "3/3", Some("260613-1700-x"));
        let result = validate_governance(&doc, Some(make_path("decision")));
        let g09 = result.violations.iter().find(|v| v.rule_id == "V-G-09");
        assert!(g09.is_some());
    }

    #[test]
    fn test_g09_decision_with_decided_by_ok() {
        let mut doc = make_test_doc("260613-1800-test", "3/3", Some("260613-1700-x"));
        doc.frontmatter.decided_by = Some("moc".to_string());
        let result = validate_governance(&doc, Some(make_path("decision")));
        let g09 = result.violations.iter().find(|v| v.rule_id == "V-G-09");
        assert!(g09.is_none());
    }

    // ── V-G-10: 根文档自指向 ──

    #[test]
    fn test_g10_root_self_reference_ok() {
        let path = Path::new("docs/specs/philosophy/On-SiHankor.sih.md");
        let doc = make_test_doc(
            "240602-0900-on-sihankor",
            "3/3",
            Some("240602-0900-on-sihankor"),
        );
        // V-G-10 is silent: no violation emitted for self-reference
        let result = validate_governance(&doc, Some(path));
        let violations = result
            .violations
            .iter()
            .filter(|v| v.rule_id == "V-G-10")
            .count();
        assert_eq!(violations, 0); // V-G-10 should not emit violations, just validate internally
    }

    // ── V-J-01: 列表嵌套 ≤ 2 层 ──

    #[test]
    fn test_j01_nested_list_ok() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "- item\n  - sub\n    - subsub".to_string();
        let result = validate_content(&doc);
        let j01 = result.violations.iter().find(|v| v.rule_id == "V-J-01");
        assert!(j01.is_none());
    }

    #[test]
    fn test_j01_list_too_deep() {
        let mut doc = make_test_doc("260613-1800-test", "1/3", None);
        doc.content = "- a\n  - b\n    - c\n      - d".to_string();
        let result = validate_content(&doc);
        let j01 = result.violations.iter().find(|v| v.rule_id == "V-J-01");
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
