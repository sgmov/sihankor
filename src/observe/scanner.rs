//! 纯 Markdown 结构扫描器
//!
//! 输入：任意项目根目录
//! 输出：5 维结构特征 + 5 维规则触达预测
//!
//! 设计原则：与司衡 core 完全解耦（不引用 `crate::core::` 任何 types）。
//! 这样保证观测窗不消费 .sih.md / frontmatter schema / stage 等概念，
//! 纯 markdown 语法特征分析。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

/// 项目观察结果
#[derive(Debug, Clone)]
pub struct ProjectObservation {
    /// 被扫描的根目录
    pub root: PathBuf,
    /// 扫描时间（RFC 3339）
    pub scanned_at: String,
    /// 跳过的目录（如 node_modules, target, .git）
    pub skipped_dirs: Vec<String>,
    /// 5 维结构特征
    pub file_stats: FileStats,
    pub dir_depth_distribution: HashMap<usize, usize>,
    pub table_stats: TableStats,
    pub code_block_stats: CodeBlockStats,
    pub frontmatter_stats: FrontmatterStats,
    pub horizontal_rule_count: usize,
    pub emoji_line_count: usize,
}

/// 文件统计
#[derive(Debug, Clone, Default, Serialize)]
pub struct FileStats {
    pub total_files: usize,
    pub total_bytes: u64,
    pub total_lines: u64,
    pub avg_lines: f64,
}

/// 表格统计
#[derive(Debug, Clone, Default, Serialize)]
pub struct TableStats {
    pub total_tables: usize,
    /// 列数 → 该列数的表数
    pub column_distribution: HashMap<usize, usize>,
    /// 含 ≥ 4 列表格的文件 id 集合
    pub files_with_wide_table: Vec<PathBuf>,
}

/// 代码块统计
#[derive(Debug, Clone, Default, Serialize)]
pub struct CodeBlockStats {
    pub total_blocks: usize,
    pub with_lang: usize,
    pub without_lang: usize,
}

/// frontmatter 统计（不解析司衡 schema）
#[derive(Debug, Clone, Default)]
pub struct FrontmatterStats {
    pub files_with_frontmatter: usize,
    pub files_with_id_field: usize,
    pub files_with_stage_field: usize,
    /// frontmatter 含 `stage` 字段且值为 SiHankor 合法值（1/3, 2/3, 3/3, X, 0/...）
    pub files_with_sihankor_stage: usize,
    /// frontmatter 含 stage 但缺 id 的文件路径（V-F-01 预测目标）
    pub stage_without_id: Vec<PathBuf>,
}

/// 默认跳过的目录（依赖缓存、构建产物、版本控制）
const DEFAULT_SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "__pycache__",
    ".pytest_cache",
    "venv",
    ".venv",
    "dist",
    "build",
    ".next",
    "vendor",
];

/// 扫描一个项目目录
///
/// `root` 可以是任意目录。返回 5 维结构特征。
/// 不修改任何文件，不引用司衡 core 的任何类型。
pub fn scan_project(root: &Path) -> std::io::Result<ProjectObservation> {
    let mut obs = ProjectObservation {
        root: root.to_path_buf(),
        scanned_at: current_rfc3339(),
        skipped_dirs: DEFAULT_SKIP_DIRS.iter().map(|s| s.to_string()).collect(),
        file_stats: FileStats::default(),
        dir_depth_distribution: HashMap::new(),
        table_stats: TableStats::default(),
        code_block_stats: CodeBlockStats::default(),
        frontmatter_stats: FrontmatterStats::default(),
        horizontal_rule_count: 0,
        emoji_line_count: 0,
    };

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !DEFAULT_SKIP_DIRS.contains(&name.as_ref())
        })
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !(name.ends_with(".md") || name.ends_with(".markdown")) {
            continue;
        }

        // 计算目录深度（相对 root）
        let rel = path.strip_prefix(root).unwrap_or(path);
        let depth = rel.components().count().saturating_sub(1);
        *obs.dir_depth_distribution.entry(depth).or_insert(0) += 1;

        // 读文件内容
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // 跳过不可读文件
        };

        obs.file_stats.total_files += 1;
        obs.file_stats.total_bytes += content.len() as u64;

        let lines: Vec<&str> = content.lines().collect();
        obs.file_stats.total_lines += lines.len() as u64;

        analyze_file(&content, &lines, path, &mut obs);
    }

    obs.file_stats.avg_lines = if obs.file_stats.total_files > 0 {
        obs.file_stats.total_lines as f64 / obs.file_stats.total_files as f64
    } else {
        0.0
    };

    Ok(obs)
}

/// 分析单个 markdown 文件
fn analyze_file(_content: &str, lines: &[&str], path: &Path, obs: &mut ProjectObservation) {
    // 1. frontmatter 检测 + 字段统计
    let (frontmatter_end, has_frontmatter) = detect_frontmatter(lines);
    if has_frontmatter {
        obs.frontmatter_stats.files_with_frontmatter += 1;
        let fm = &lines[..frontmatter_end];
        let fm_text = fm.join("\n");
        let has_id = has_field(&fm_text, "id");
        let has_stage = has_field(&fm_text, "stage");
        if has_id {
            obs.frontmatter_stats.files_with_id_field += 1;
        }
        if has_stage {
            obs.frontmatter_stats.files_with_stage_field += 1;
            // 检查 stage 值是否像 SiHankor 合法值
            if let Some(stage_val) = get_field_value(&fm_text, "stage") {
                if is_sihankor_stage(&stage_val) {
                    obs.frontmatter_stats.files_with_sihankor_stage += 1;
                    if !has_id {
                        obs.frontmatter_stats
                            .stage_without_id
                            .push(path.to_path_buf());
                    }
                }
            }
        }
    }

    // 2. body 部分（去掉 frontmatter）
    let body_lines: Vec<&str> = if has_frontmatter {
        lines[frontmatter_end..].to_vec()
    } else {
        lines.to_vec()
    };

    // 3. 表格检测
    let table_count = analyze_tables(&body_lines, path, &mut obs.table_stats);
    if table_count > 0 {
        // 已经在 analyze_tables 中更新
    }

    // 4. 代码块检测
    analyze_code_blocks(&body_lines, &mut obs.code_block_stats);

    // 5. 水平线（上下文感知：排除代码块 / 表格 / 列表项内的 ---）
    obs.horizontal_rule_count = count_horizontal_rules(&body_lines);

    // 6. emoji 行数
    for line in &body_lines {
        if line_contains_emoji(line) {
            obs.emoji_line_count += 1;
        }
    }
}

/// 检测文件是否有 frontmatter（`---\n...\n---\n` 在文件头部）
/// 返回 (frontmatter 结束行号, 是否有 frontmatter)
fn detect_frontmatter(lines: &[&str]) -> (usize, bool) {
    if lines.is_empty() || lines[0].trim() != "---" {
        return (0, false);
    }
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            return (i + 1, true);
        }
    }
    (0, false) // 没找到闭合
}

/// 检查 frontmatter 文本是否含指定字段（粗略正则，足够 MVP）
fn has_field(fm_text: &str, field: &str) -> bool {
    for line in fm_text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(field) {
            // 字段后必须是 `:` 或 `: ` 分隔
            if rest.starts_with(':') {
                return true;
            }
        }
    }
    false
}

/// 获取 frontmatter 字段值
fn get_field_value(fm_text: &str, field: &str) -> Option<String> {
    for line in fm_text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(field) {
            if let Some(val) = rest.strip_prefix(':') {
                return Some(val.trim().to_string());
            }
        }
    }
    None
}

/// 是否是 SiHankor 合法 stage 值
fn is_sihankor_stage(s: &str) -> bool {
    matches!(s, "1/3" | "2/3" | "3/3" | "X") || s.starts_with("0/")
}

/// 分析表格：统计列数分布
fn analyze_tables(lines: &[&str], path: &Path, stats: &mut TableStats) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with('|') && line.ends_with('|') && line.contains('|') {
            // 这是一个表头行
            let col_count = count_columns(line);
            if col_count > 0 {
                count += 1;
                *stats.column_distribution.entry(col_count).or_insert(0) += 1;
                if col_count >= 4 {
                    if !stats.files_with_wide_table.contains(&path.to_path_buf()) {
                        stats.files_with_wide_table.push(path.to_path_buf());
                    }
                }
                // 跳过分隔行（`|---|---|`）和数据行
                i += 1;
                while i < lines.len() {
                    let next = lines[i].trim();
                    if next.starts_with('|') && next.ends_with('|') {
                        i += 1;
                    } else {
                        break;
                    }
                }
                continue;
            }
        }
        i += 1;
    }
    stats.total_tables = count;
    count
}

/// 统计一行 markdown 表格的列数
fn count_columns(line: &str) -> usize {
    // 格式: | a | b | c |
    // 去掉首尾的 |，按 | 分割
    let inner = line.trim().trim_matches('|');
    inner.split('|').count()
}

/// 分析代码块：统计总数、带 lang 的、缺 lang 的
fn analyze_code_blocks(lines: &[&str], stats: &mut CodeBlockStats) {
    let mut in_block = false;

    for line in lines {
        let trimmed = line.trim_start();
        if let Some(after_fence) = trimmed.strip_prefix("```") {
            if !in_block {
                in_block = true;
                // 检查是否有 language tag
                let lang = after_fence.trim();
                let has_lang = !lang.is_empty();
                stats.total_blocks += 1;
                if has_lang {
                    stats.with_lang += 1;
                } else {
                    stats.without_lang += 1;
                }
            } else {
                in_block = false;
            }
        }
    }
}

/// 统计水平线数量：仅计算真正的独立水平线，
/// 排除代码块内、表格内、列表项内的 ---。
///
/// 状态机：
/// - 遇到 ``` 切换 in_code_block
/// - 遇到 | 开头 + | 结尾的行进入 in_table
/// - 遇到 -/* /+/digit. 开头的行进入 list
/// - 只有在以上三种上下文外、且行严格等于 `---`（无 leading whitespace）才计为水平线
fn count_horizontal_rules(lines: &[&str]) -> usize {
    let mut count = 0;
    let mut in_code_block = false;
    let mut in_table = false;

    for line in lines {
        let trimmed = line.trim_start();

        // Code block 边界
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            in_table = false; // 进入/退出 code block 时重置 table 状态
            continue;
        }
        if in_code_block {
            continue;
        }

        // Table 行：以 | 开头并以 | 结尾
        if is_table_line(trimmed) {
            in_table = true;
            continue;
        }
        // 退出 table 状态：空行
        if in_table && trimmed.is_empty() {
            in_table = false;
            continue;
        }
        if in_table {
            continue; // table 内部的非 | 行（虽然少见）跳过
        }

        // List item：-/* /+/digit. 开头
        if is_list_item(trimmed) {
            continue;
        }

        // 真正的水平线：行严格等于 `---`（无 leading whitespace）
        if *line == "---" {
            count += 1;
        }
    }

    count
}

/// 是否是 markdown 表格行
fn is_table_line(trimmed: &str) -> bool {
    trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.contains('|')
}

/// 是否是 markdown 列表项
fn is_list_item(trimmed: &str) -> bool {
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return true;
    }
    // 有序列表：1. 2. 3. ...
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0 && i + 1 < bytes.len() && &bytes[i..i + 2] == b". " {
        return true;
    }
    false
}

/// 检查一行是否含 emoji
fn line_contains_emoji(line: &str) -> bool {
    line.chars().any(|c| {
        let cp = c as u32;
        (0x1F300..=0x1FAFF).contains(&cp) || (0x2600..=0x27BF).contains(&cp) || cp == 0x200D
    })
}

fn current_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // 简单 RFC 3339 格式：YYYY-MM-DDTHH:MM:SSZ
    // 不依赖 chrono：手动算
    let (year, month, day, hour, min, sec) = epoch_to_ymdhms(secs);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    )
}

fn epoch_to_ymdhms(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = (rem / 3600) as u32;
    let min = ((rem % 3600) / 60) as u32;
    let sec = (rem % 60) as u32;
    // 1970-01-01 + days
    let (y, m, d) = days_to_ymd(days as i64);
    (y, m, d, hour, min, sec)
}

fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    // 简化算法：从 1970-01-01 开始
    let mut year = 1970i32;
    let mut remaining = days;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let year_days = if leap { 366 } else { 365 };
        if remaining < year_days {
            break;
        }
        remaining -= year_days;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_days = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        month += 1;
    }
    (year, month as u32, (remaining + 1) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_scan_empty_dir() {
        let dir = make_temp_dir();
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.file_stats.total_files, 0);
    }

    #[test]
    fn test_scan_single_file() {
        let dir = make_temp_dir();
        write_file(dir.path(), "test.md", "# Hello\n\nSome content.");
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.file_stats.total_files, 1);
        assert!(obs.file_stats.avg_lines > 0.0);
    }

    #[test]
    fn test_detect_frontmatter() {
        let lines = vec!["---", "id: test", "stage: 1/3", "---", "# Body"];
        let (end, has) = detect_frontmatter(&lines);
        assert!(has);
        assert_eq!(end, 4);
    }

    #[test]
    fn test_no_frontmatter() {
        let lines = vec!["# Just a title", "Some content"];
        let (end, has) = detect_frontmatter(&lines);
        assert!(!has);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_count_columns() {
        assert_eq!(count_columns("| a | b | c |"), 3);
        assert_eq!(count_columns("| a | b | c | d |"), 4);
        assert_eq!(count_columns("| only |"), 1);
    }

    #[test]
    fn test_sihankor_stage_detection() {
        assert!(is_sihankor_stage("1/3"));
        assert!(is_sihankor_stage("2/3"));
        assert!(is_sihankor_stage("3/3"));
        assert!(is_sihankor_stage("X"));
        assert!(is_sihankor_stage("0/new-id"));
        assert!(!is_sihankor_stage("propose"));
        assert!(!is_sihankor_stage("draft"));
    }

    #[test]
    fn test_horizontal_rule_detection() {
        let dir = make_temp_dir();
        write_file(dir.path(), "rule.md", "# Title\n\n---\n\nMore content");
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.horizontal_rule_count, 1);
    }

    #[test]
    fn test_horizontal_rule_inside_code_block_not_counted() {
        let dir = make_temp_dir();
        let content = "```\n---\nsome code\n---\n```\n";
        write_file(dir.path(), "code.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.horizontal_rule_count, 0);
    }

    #[test]
    fn test_horizontal_rule_inside_list_not_counted() {
        let dir = make_temp_dir();
        let content = "- item 1\n  - sub item\n  ---\n  more text\n- item 2\n";
        write_file(dir.path(), "list.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.horizontal_rule_count, 0);
    }

    #[test]
    fn test_horizontal_rule_inside_table_not_counted() {
        let dir = make_temp_dir();
        let content = "| a | b | c |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n";
        write_file(dir.path(), "table.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.horizontal_rule_count, 0);
    }

    #[test]
    fn test_horizontal_rule_mixed_real_and_false() {
        // 混合：2 个真水平线 + 几个 false positive
        let dir = make_temp_dir();
        let content = "# Title\n\n---\n\n```\n---\n```\n\n- item\n  ---\n  text\n\n---\n\n| a | b |\n|---|---|\n";
        write_file(dir.path(), "mixed.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(
            obs.horizontal_rule_count, 2,
            "expected 2 real rules, got {}",
            obs.horizontal_rule_count
        );
    }

    #[test]
    fn test_indented_dashes_not_counted() {
        // 缩进的 --- 是 setext h2 标记，不是水平线
        let dir = make_temp_dir();
        let content = "Paragraph\n  ---\nMore text\n";
        write_file(dir.path(), "setext.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.horizontal_rule_count, 0);
    }

    #[test]
    fn test_table_analysis() {
        let dir = make_temp_dir();
        let content = "| a | b | c |\n|---|---|---|\n| 1 | 2 | 3 |\n\n| x | y | z | w |\n|---|---|---|---|\n| 1 | 2 | 3 | 4 |\n";
        write_file(dir.path(), "tables.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.table_stats.total_tables, 2);
        assert_eq!(*obs.table_stats.column_distribution.get(&3).unwrap(), 1);
        assert_eq!(*obs.table_stats.column_distribution.get(&4).unwrap(), 1);
        assert_eq!(obs.table_stats.files_with_wide_table.len(), 1);
    }

    #[test]
    fn test_code_block_lang() {
        let dir = make_temp_dir();
        let content = "```rust\nlet x = 1;\n```\n\n```\nsome code\n```\n";
        write_file(dir.path(), "code.md", content);
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.code_block_stats.total_blocks, 2);
        assert_eq!(obs.code_block_stats.with_lang, 1);
        assert_eq!(obs.code_block_stats.without_lang, 1);
    }

    #[test]
    fn test_skip_default_dirs() {
        let dir = make_temp_dir();
        write_file(dir.path(), "real.md", "# real");
        write_file(dir.path(), "node_modules/dep.md", "# dep");
        write_file(dir.path(), "target/build.md", "# build");
        let obs = scan_project(dir.path()).unwrap();
        assert_eq!(obs.file_stats.total_files, 1);
    }
}

// 使用 tempdir crate（已有依赖）— 实际项目里查找
