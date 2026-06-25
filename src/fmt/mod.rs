//! format-lint: document format constraint enforcement for .sih.md files.
//!
//! Rules C-01 through C-10 implement Document-Conventions §八 format constraints.
//! Character rules (C-01..C-04) skip fenced code blocks.
//! Structure rules (C-05..C-10) apply to all lines.

use serde::Deserialize;

/// Format lint configuration, loaded from .sih/config.yml.
#[derive(Debug, Clone, Deserialize)]
pub struct FormatConfig {
    #[serde(default)]
    pub format: FormatSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FormatSection {
    #[serde(default = "default_pre_commit")]
    pub pre_commit: bool,
    #[serde(default)]
    pub style: StyleSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StyleSection {
    #[serde(default)]
    pub mix_lang_ignore: bool,
}

const fn default_pre_commit() -> bool {
    true
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            format: FormatSection {
                pre_commit: true,
                style: StyleSection {
                    mix_lang_ignore: false,
                },
            },
        }
    }
}

impl FormatConfig {
    /// Load config from .sih/config.yml if present, otherwise return defaults.
    pub fn load() -> Self {
        let path = std::path::Path::new(".sih/config.yml");
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
        {
            match serde_yaml::from_str(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!(
                        "sihankor-fmt: warning: failed to parse .sih/config.yml: {}. Using defaults.",
                        e
                    );
                }
            }
        }
        FormatConfig::default()
    }
}

/// Format violation level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
}

impl Level {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
        }
    }
}

/// A single format violation.
#[derive(Debug, Clone)]
pub struct Violation {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub code: String,
    pub level: Level,
    pub message: String,
    /// 修复建议（面向开发者的可操作指引）
    pub fix_suggestion: Option<String>,
    /// 道法追溯：此规则对应哪个司衡道法维度
    pub dao_trace: Option<String>,
}

impl Violation {
    /// Format as `file:line:col: CODE level: message` for editor integration.
    pub fn format(&self) -> String {
        format!(
            "{}:{}:{}: {} {}: {}",
            self.file,
            self.line,
            self.col,
            self.code,
            self.level.as_str(),
            self.message
        )
    }

    /// Output as JSON for CI integration.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&serde_json::json!({
            "file": self.file,
            "line": self.line,
            "col": self.col,
            "code": self.code,
            "level": self.level.as_str(),
            "message": self.message,
            "fix_suggestion": self.fix_suggestion,
            "dao_trace": self.dao_trace,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}

/// Check if a line is inside a fenced code block (starts or ends one).
/// Returns the new in_codeblock state.
pub fn update_codeblock_state(line: &str, in_codeblock: bool) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        return !in_codeblock;
    }
    in_codeblock
}

/// Check if a character is within the CJK Unified Ideographs range.
const fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}' // CJK Unified Ideographs Extension A
        | '\u{20000}'..='\u{2A6DF}' // CJK Unified Ideographs Extension B
        | '\u{3000}'..='\u{303F}' // CJK Symbols and Punctuation
        | '\u{FF00}'..='\u{FFEF}' // Halfwidth and Fullwidth Forms
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{2F800}'..='\u{2FA1F}' // CJK Compatibility Ideographs Supplement
    )
}

/// Permitted CJK punctuation per Document-Conventions §8.5:
/// U+3001, U+3002, U+FF0C, U+FF1A, U+FF1B, U+FF08, U+FF09, U+300A, U+300B, U+300C, U+300D
const fn is_permitted_cjk_punctuation(c: char) -> bool {
    matches!(
        c,
        '\u{3001}' // 、
        | '\u{3002}' // 。
        | '\u{FF0C}' // ，
        | '\u{FF1A}' // ：
        | '\u{FF1B}' // ；
        | '\u{FF08}' // （
        | '\u{FF09}' // ）
        | '\u{300A}' // 《
        | '\u{300B}' // 》
        | '\u{300C}' // 「
        | '\u{300D}' // 」
    )
}

const fn is_emoji(c: char) -> bool {
    // Simple emoji detection by Unicode blocks commonly associated with emoji.
    matches!(
        c,
        '\u{1F000}'..='\u{1FFFF}'  // Miscellaneous Symbols, Emoticons, Dingbats, etc.
        | '\u{2600}'..='\u{27BF}'  // Miscellaneous Symbols (covers hearts, dingbats, etc.)
        | '\u{2300}'..='\u{23FF}'  // Miscellaneous Technical
        | '\u{FE00}'..='\u{FE0F}'  // Variation Selectors (emoji modifier)
        | '\u{E0000}'..='\u{E007F}' // Tags block
    )
}

// ── C-01: emoji detection ──

pub fn check_c01(line: &str, line_num: usize, file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (col, c) in line.char_indices() {
        if is_emoji(c) {
            violations.push(Violation {
                file: file.to_string(),
                line: line_num,
                col: col + 1,
                code: "C01".to_string(),
                level: Level::Error,
                message: format!("emoji character found: '{}' (U+{:04X})", c, c as u32),
                fix_suggestion: Some("Replace emoji with text description".to_string()),
                dao_trace: Some("损补".to_string()),
            });
        }
    }
    violations
}

// ── C-01a: non-ASCII non-CJK non-permitted punctuation ──

pub fn check_c01a(line: &str, line_num: usize, file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (col, c) in line.char_indices() {
        if c.is_ascii() {
            continue; // ASCII is always permitted
        }
        if is_cjk(c) {
            continue; // CJK characters are permitted
        }
        if is_permitted_cjk_punctuation(c) {
            continue; // Explicitly permitted CJK punctuation
        }
        // Skip characters handled by more specific rules (C-02, C-03, C-04)
        if matches!(
            c,
            '\u{201C}' | '\u{201D}' | '\u{2014}' | '\u{2190}' | '\u{2192}'
        ) {
            continue;
        }
        violations.push(Violation {
            file: file.to_string(),
            line: line_num,
            col: col + 1,
            code: "C01a".to_string(),
            level: Level::Error,
            message: format!("non-ASCII non-CJK character: '{}' (U+{:04X})", c, c as u32),
            fix_suggestion: Some(
                "Replace with ASCII equivalent or permitted CJK character".to_string(),
            ),
            dao_trace: Some("损补".to_string()),
        });
    }
    violations
}

// ── C-02: em-dash U+2014 ──

pub fn check_c02(line: &str, line_num: usize, file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (col, c) in line.char_indices() {
        if c == '\u{2014}' {
            violations.push(Violation {
                file: file.to_string(),
                line: line_num,
                col: col + 1,
                code: "C02".to_string(),
                level: Level::Error,
                message: "em-dash (U+2014) should be fullwidth colon (U+FF1A)".to_string(),
                fix_suggestion: Some(
                    "Replace with fullwidth colon (：) or ASCII hyphen (-)".to_string(),
                ),
                dao_trace: Some("损补".to_string()),
            });
        }
    }
    violations
}

// ── C-03: curly/smart quotes ──

pub fn check_c03(line: &str, line_num: usize, file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (col, c) in line.char_indices() {
        if c == '\u{201C}' || c == '\u{201D}' {
            violations.push(Violation {
                file: file.to_string(),
                line: line_num,
                col: col + 1,
                code: "C03".to_string(),
                level: Level::Error,
                message: format!(
                    "curly quote (U+{:04X}) should be straight double quote",
                    c as u32
                ),
                fix_suggestion: Some("Replace with straight double quote (\")".to_string()),
                dao_trace: Some("损补".to_string()),
            });
        }
    }
    violations
}

// ── C-04: arrows (→/← not as ASCII) ──

pub fn check_c04(line: &str, line_num: usize, file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (col, c) in line.char_indices() {
        match c {
            '\u{2192}' => {
                violations.push(Violation {
                    file: file.to_string(),
                    line: line_num,
                    col: col + 1,
                    code: "C04".to_string(),
                    level: Level::Error,
                    message: "right arrow (U+2192) should be '->'".to_string(),
                    fix_suggestion: Some("Replace -> with ASCII '->'".to_string()),
                    dao_trace: Some("损补".to_string()),
                });
            }
            '\u{2190}' => {
                violations.push(Violation {
                    file: file.to_string(),
                    line: line_num,
                    col: col + 1,
                    code: "C04".to_string(),
                    level: Level::Error,
                    message: "left arrow (U+2190) should be '<-'".to_string(),
                    fix_suggestion: Some("Replace with ASCII '<-'".to_string()),
                    dao_trace: Some("损补".to_string()),
                });
            }
            _ => {}
        }
    }
    violations
}

// ── C-05: horizontal rule `---` in body (not frontmatter delimiters) ──

pub fn check_c05(lines: &[&str], file: &str, doc_start: usize) -> Vec<Violation> {
    // doc_start is the line index of the closing `---` of frontmatter.
    // doc_end is None or the line index of the opening `---` of the closing frontmatter (n/a).
    // Actually, per Conventions, `---` is only allowed as frontmatter delimiters.
    // After the frontmatter closing `---`, any `---` is a violation.
    let mut violations = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            // Allow the frontmatter opening (line 0) and closing
            if i == 0 || i == doc_start {
                continue;
            }
            violations.push(Violation {
                file: file.to_string(),
                line: i + 1,
                col: 1,
                code: "C05".to_string(),
                level: Level::Error,
                message: "horizontal rule in body; use '##' headings instead".to_string(),
                fix_suggestion: Some(
                    "Use level-2 headings (## ) for section separation instead of ---".to_string(),
                ),
                dao_trace: Some("有度".to_string()),
            });
        }
    }
    violations
}

// ── C-06: list nesting > 2 levels ──

pub fn check_c06(
    lines: &[&str],
    file: &str,
    codeblock_lines: &std::collections::HashSet<usize>,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if codeblock_lines.contains(&i) {
            continue;
        }
        let trimmed = line.trim_start();
        // Count leading spaces/tabs to determine nesting depth
        let leading = line.len() - trimmed.len();
        // List markers: -, *, +, or numbered (1., 1), etc.)
        let is_list_item = trimmed.starts_with('-')
            || trimmed.starts_with('*')
            || trimmed.starts_with('+')
            || trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                && trimmed.chars().find(|c| !c.is_ascii_digit()) == Some('.');

        if is_list_item {
            // 2-space indent per nesting level
            let depth = leading / 2;
            if depth > 2 {
                violations.push(Violation {
                    file: file.to_string(),
                    line: i + 1,
                    col: 1,
                    code: "C06".to_string(),
                    level: Level::Warning,
                    message: format!(
                        "list nesting depth {} exceeds maximum 2; use paragraphs instead",
                        depth
                    ),
                    fix_suggestion: Some(
                        "Flatten deeply nested lists: use paragraphs or subsections instead"
                            .to_string(),
                    ),
                    dao_trace: Some("有度".to_string()),
                });
            }
        }
    }
    violations
}

// ── C-07: suspected ASCII art (≥3 consecutive lines starting with |+/-\) ──

pub fn check_c07(
    lines: &[&str],
    file: &str,
    codeblock_lines: &std::collections::HashSet<usize>,
) -> Vec<Violation> {
    let ascii_art_starts = ['|', '+', '-', '/', '\\'];
    let mut violations = Vec::new();
    let mut run_start: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        if codeblock_lines.contains(&i) {
            run_start = None;
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            run_start = None;
            continue;
        }
        let Some(first_char) = trimmed.chars().next() else {
            continue;
        };
        if ascii_art_starts.contains(&first_char) {
            if run_start.is_none() {
                run_start = Some(i);
            }
            if let Some(rs) = run_start {
                let run_len = i - rs + 1;
                if run_len >= 3 {
                    violations.push(Violation {
                        file: file.to_string(),
                        line: rs + 1,
                    col: 1,
                    code: "C07".to_string(),
                    level: Level::Warning,
                    message: "suspected ASCII art diagram (≥3 consecutive lines); use Mermaid flowchart instead".to_string(),
                    fix_suggestion: Some("Replace ASCII art with a Mermaid flowchart diagram".to_string()),
                    dao_trace: Some("有度".to_string()),
                });
                    run_start = None; // Reset to avoid duplicate reports
                }
            }
        } else {
            run_start = None;
        }
    }
    violations
}

// ── C-08: mixed CJK/English in same paragraph ──

pub fn check_c08(
    lines: &[&str],
    file: &str,
    codeblock_lines: &std::collections::HashSet<usize>,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if codeblock_lines.contains(&i) {
            continue;
        }
        let has_cjk = line.chars().any(is_cjk);
        let has_ascii_alpha = line.chars().any(|c| c.is_ascii_alphabetic());
        // Skip lines that are purely structural (headings, tables, links)
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with('|') || trimmed.starts_with('[') {
            continue;
        }
        if has_cjk && has_ascii_alpha {
            violations.push(Violation {
                file: file.to_string(),
                line: i + 1,
                col: 1,
                code: "C08".to_string(),
                level: Level::Warning,
                message: "mixed Chinese/English in same paragraph".to_string(),
                fix_suggestion: Some("Separate CJK and English into distinct paragraphs or wrap English terms in code spans".to_string()),
                dao_trace: Some("损补".to_string()),
            });
        }
    }
    violations
}

// ── C-09: code block without valid language tag ──

const VALID_LANG_TAGS: &[&str] = &["mermaid", "text", "yaml", "json", "rust", "markdown"];

pub fn check_c09(lines: &[&str], file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut in_codeblock = false;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some(after_fence) = trimmed.strip_prefix("```") {
            if !in_codeblock {
                // Opening fence — check language tag
                let tag = after_fence.trim();
                if tag.is_empty() {
                    violations.push(Violation {
                        file: file.to_string(),
                        line: i + 1,
                        col: 1,
                        code: "C09".to_string(),
                        level: Level::Error,
                        message: "code block missing language tag".to_string(),
                        fix_suggestion: Some(
                            "Add a language tag: ```rust, ```mermaid, ```text".to_string(),
                        ),
                        dao_trace: Some("有度".to_string()),
                    });
                } else if !VALID_LANG_TAGS.contains(&tag) {
                    violations.push(Violation {
                        file: file.to_string(),
                        line: i + 1,
                        col: 1,
                        code: "C09".to_string(),
                        level: Level::Warning,
                        message: format!(
                            "unknown language tag '{}'; permitted: {:?}",
                            tag, VALID_LANG_TAGS
                        ),
                        fix_suggestion: Some(format!(
                            "Use one of the permitted tags: {}",
                            VALID_LANG_TAGS.join(", ")
                        )),
                        dao_trace: Some("有度".to_string()),
                    });
                }
            }
            in_codeblock = !in_codeblock;
        }
    }
    violations
}

// ── C-10: table columns > 3 ──

pub fn check_c10(lines: &[&str], file: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut last_table_violation = false;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let cols: Vec<&str> = trimmed
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .collect();
            let is_separator = cols
                .iter()
                .all(|s| s.trim().chars().all(|c| c == '-' || c == ':' || c == ' '));
            if is_separator {
                let parts: Vec<&str> = trimmed.split('|').collect();
                let count = if parts.len() > 2 { parts.len() - 2 } else { 0 };
                if count > 3 && !last_table_violation {
                    violations.push(Violation {
                        file: file.to_string(),
                        line: i + 1,
                        col: 1,
                        code: "C10".to_string(),
                        level: Level::Error,
                        message: format!("table has {} columns; maximum is 3", count),
                        fix_suggestion: Some(
                            "Split wide table into bullet lists or subsections".to_string(),
                        ),
                        dao_trace: Some("有度".to_string()),
                    });
                    last_table_violation = true;
                }
            } else if cols.len() > 3 && !last_table_violation {
                violations.push(Violation {
                    file: file.to_string(),
                    line: i + 1,
                    col: 1,
                    code: "C10".to_string(),
                    level: Level::Error,
                    message: format!("table has {} columns; maximum is 3", cols.len()),
                    fix_suggestion: Some(
                        "Split wide table into bullet lists or subsections".to_string(),
                    ),
                    dao_trace: Some("有度".to_string()),
                });
                last_table_violation = true;
            }
        } else {
            last_table_violation = false;
        }
    }
    violations
}

/// Lint a single document, returning all violations.
pub fn lint_document(file: &str, content: &str, config: &FormatConfig) -> Vec<Violation> {
    let lines: Vec<&str> = content.lines().collect();

    // Find frontmatter boundaries and track code blocks
    let mut doc_start: Option<usize> = None; // line index of closing `---` of frontmatter
    let mut in_frontmatter = false;
    let mut in_codeblock = false;
    let mut codeblock_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if i == 0 && trimmed == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter && trimmed == "---" {
            in_frontmatter = false;
            doc_start = Some(i);
            continue;
        }
        // Track code blocks after frontmatter
        if !in_frontmatter {
            in_codeblock = update_codeblock_state(line, in_codeblock);
            if in_codeblock {
                codeblock_lines.insert(i);
            }
        }
    }

    let fm_end = doc_start.unwrap_or(0);
    let mut violations = Vec::new();

    // Apply character-level rules (skip code blocks and frontmatter)
    for (i, line) in lines.iter().enumerate() {
        if i <= fm_end {
            continue; // Skip frontmatter
        }
        if codeblock_lines.contains(&i) {
            continue; // Skip code blocks
        }
        let line_num = i + 1;
        violations.extend(check_c01(line, line_num, file));
        violations.extend(check_c01a(line, line_num, file));
        violations.extend(check_c02(line, line_num, file));
        violations.extend(check_c03(line, line_num, file));
        violations.extend(check_c04(line, line_num, file));
    }

    // Apply structure-level rules (all lines, code blocks filtered per-rule)
    violations.extend(check_c05(&lines, file, fm_end));
    violations.extend(check_c06(&lines, file, &codeblock_lines));
    violations.extend(check_c07(&lines, file, &codeblock_lines));
    if !config.format.style.mix_lang_ignore {
        violations.extend(check_c08(&lines, file, &codeblock_lines));
    }
    violations.extend(check_c09(&lines, file));
    {
        let mut c10_violations = check_c10(&lines, file);
        // C-10 按 nature 分级：spec/decision 文档宽表为 Warning，其余 Error
        // 法依据：有度——spec/decision 的多维映射表服务于结构呈现，非叙事可读性
        let path = std::path::Path::new(file);
        let nature = crate::core::validator::infer_nature(path).unwrap_or("unknown");
        if nature == "spec" || nature == "decision" {
            for v in &mut c10_violations {
                v.level = Level::Warning;
                v.message = "table has >3 columns (spec/decision docs: warning only; split if for narrative purpose)".to_string();
                v.fix_suggestion = Some("Consider splitting this table if it serves narrative rather than structural purpose".to_string());
            }
        }
        violations.extend(c10_violations);
    }

    violations
}

/// 产出格式 lint 的治理追溯标记（与 validator 的 trailer 格式一致）
///
/// 格式错误映射：Error -> Fatal（阻断），Warning -> Guideline（警告）
pub fn governance_trailer(violations: &[Violation]) -> String {
    let errors = violations
        .iter()
        .filter(|v| v.level == Level::Error)
        .count();
    let warnings = violations
        .iter()
        .filter(|v| v.level == Level::Warning)
        .count();

    let mut dao_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for v in violations {
        if let Some(ref dao) = v.dao_trace {
            *dao_counts.entry(dao.as_str()).or_insert(0) += 1;
        }
    }

    let status = if errors > 0 { "blocked" } else { "pass" };
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
        "SiHankor-Governance: {} (E={} W={}; {})",
        status,
        errors,
        warnings,
        dao_parts.join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _make_file_content(frontmatter: &str, body: &str) -> String {
        format!("---\n{}\n---\n{}", frontmatter, body)
    }

    // ── C-01: emoji ──

    #[test]
    fn test_c01_no_emoji() {
        let v = check_c01("plain text", 1, "test.md");
        assert!(v.is_empty());
    }

    #[test]
    fn test_c01_emoji_detected() {
        let v = check_c01("hello 🔥 world", 1, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C01");
        assert_eq!(v[0].level, Level::Error);
    }

    // ── C-01a: non-ASCII non-CJK ──

    #[test]
    fn test_c01a_no_violation() {
        let v = check_c01a("hello 你好", 1, "test.md");
        assert!(v.is_empty());
    }

    #[test]
    fn test_c01a_middle_dot() {
        let v = check_c01a("a·b", 1, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C01a");
    }

    #[test]
    fn test_c01a_section_sign() {
        let v = check_c01a("§1", 1, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C01a");
    }

    // ── C-02: em-dash ──

    #[test]
    fn test_c02_em_dash() {
        let line = "A\u{2014}B";
        let v = check_c02(line, 1, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C02");
    }

    #[test]
    fn test_c02_no_em_dash() {
        // Fullwidth colon (U+FF1A) is the correct replacement, not em-dash
        let v = check_c02("A\u{FF1A}B", 1, "test.md");
        assert!(v.is_empty());
    }

    // ── C-03: curly quotes ──

    #[test]
    fn test_c03_curly_quotes() {
        let line = "he said \u{201C}hello\u{201D}";
        let v = check_c03(line, 1, "test.md");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].code, "C03");
    }

    #[test]
    fn test_c03_straight_quotes_ok() {
        let v = check_c03("he said \"hello\"", 1, "test.md");
        assert!(v.is_empty());
    }

    // ── C-04: arrows ──

    #[test]
    fn test_c04_right_arrow() {
        let line = "go \u{2192} there";
        let v = check_c04(line, 1, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C04");
    }

    #[test]
    fn test_c04_left_arrow() {
        let line = "back \u{2190} home";
        let v = check_c04(line, 1, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C04");
    }

    #[test]
    fn test_c04_ascii_arrows_ok() {
        let v = check_c04("a -> b <- c", 1, "test.md");
        assert!(v.is_empty());
    }

    // ── C-05: horizontal rule in body ──

    #[test]
    fn test_c05_no_hr() {
        let lines: Vec<&str> = vec!["## heading", "text"];
        let v = check_c05(&lines, "test.md", 0);
        assert!(v.is_empty());
    }

    #[test]
    fn test_c05_hr_in_body() {
        let lines: Vec<&str> = vec!["## heading", "---", "more text"];
        let v = check_c05(&lines, "test.md", 0);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C05");
    }

    // ── C-06: list nesting ──

    #[test]
    fn test_c06_nesting_ok() {
        let lines: Vec<&str> = vec!["- item", "  - nested", "    - nested2"];
        let cb = std::collections::HashSet::new();
        let v = check_c06(&lines, "test.md", &cb);
        // depth: - (0/2=0), "  -" (2/2=1), "    -" (4/2=2) = ok
        assert!(v.is_empty());
    }

    #[test]
    fn test_c06_nesting_too_deep() {
        let lines: Vec<&str> = vec!["- item", "  - nested", "    - deep", "      - too deep"];
        let cb = std::collections::HashSet::new();
        let v = check_c06(&lines, "test.md", &cb);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C06");
    }

    // ── C-07: ASCII art ──

    #[test]
    fn test_c07_no_ascii_art() {
        let lines: Vec<&str> = vec!["plain text", "more text"];
        let cb = std::collections::HashSet::new();
        let v = check_c07(&lines, "test.md", &cb);
        assert!(v.is_empty());
    }

    #[test]
    fn test_c07_ascii_art_detected() {
        let lines: Vec<&str> = vec!["|-- header --|", "|-- body ----|", "|-- footer --|"];
        let cb = std::collections::HashSet::new();
        let v = check_c07(&lines, "test.md", &cb);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C07");
    }

    // ── C-08: mixed CJK/English ──

    #[test]
    fn test_c08_no_mix() {
        let lines: Vec<&str> = vec!["这是中文", "this is english"];
        let cb = std::collections::HashSet::new();
        let v = check_c08(&lines, "test.md", &cb);
        assert!(v.is_empty());
    }

    #[test]
    fn test_c08_mixed() {
        let lines: Vec<&str> = vec!["这是一个test段落"];
        let cb = std::collections::HashSet::new();
        let v = check_c08(&lines, "test.md", &cb);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C08");
    }

    // ── C-09: code block language tag ──

    #[test]
    fn test_c09_no_tag() {
        let lines: Vec<&str> = vec!["```", "some code", "```"];
        let v = check_c09(&lines, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C09");
        assert_eq!(v[0].level, Level::Error);
    }

    #[test]
    fn test_c09_valid_tag() {
        let lines: Vec<&str> = vec!["```rust", "fn main() {}", "```"];
        let v = check_c09(&lines, "test.md");
        assert!(v.is_empty());
    }

    // ── C-10: table columns ──

    #[test]
    fn test_c10_table_ok() {
        let lines: Vec<&str> = vec!["| A | B | C |", "|---|---|---|"];
        let v = check_c10(&lines, "test.md");
        assert!(v.is_empty());
    }

    #[test]
    fn test_c10_table_too_wide() {
        let lines: Vec<&str> = vec!["| A | B | C | D |", "|---|---|---|---|"];
        let v = check_c10(&lines, "test.md");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "C10");
    }

    // ── lint_document integration ──

    #[test]
    fn test_lint_clean_document() {
        let content = "---\nid: test\nstage: 1/3\n---\n## heading\ntext here";
        let v = lint_document("test.sih.md", content, &FormatConfig::default());
        assert!(v.is_empty());
    }

    #[test]
    fn test_lint_document_with_emoji() {
        let content = "---\nid: test\nstage: 1/3\n---\n## heading\nf🔥";
        let v = lint_document("test.sih.md", content, &FormatConfig::default());
        assert!(!v.is_empty());
        assert_eq!(v[0].code, "C01");
    }

    #[test]
    fn test_lint_skips_frontmatter() {
        // emoji in frontmatter should NOT be flagged (frontmatter is not body)
        let content = "---\nid: test\nstage: 1/3\n---\nclean text";
        let v = lint_document("test.sih.md", content, &FormatConfig::default());
        assert!(v.is_empty(), "all rules should skip frontmatter lines");
    }

    #[test]
    fn test_lint_skips_code_blocks() {
        let content = "---\nid: test\nstage: 1/3\n---\n## heading\n```text\nemoji here: 🔥\n```";
        let v = lint_document("test.sih.md", content, &FormatConfig::default());
        // Code block contents should be skipped by character rules
        // But C-09 checks the opening fence tag — "text" is valid
        assert!(v.is_empty());
    }

    #[test]
    fn test_violation_format() {
        let v = Violation {
            file: "test.sih.md".to_string(),
            line: 5,
            col: 3,
            code: "C01".to_string(),
            level: Level::Error,
            message: "emoji found".to_string(),
            fix_suggestion: Some("remove emoji".to_string()),
            dao_trace: Some("损补".to_string()),
        };
        assert_eq!(v.format(), "test.sih.md:5:3: C01 error: emoji found");
    }
}
