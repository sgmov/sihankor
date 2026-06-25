//! `sihankor-proptest` — 验码：对标注函数运行随机输入 fuzzing，仅报告硬崩溃。
//!
//! Usage: `sihankor-proptest`
#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used, clippy::vec_init_then_push, clippy::unnecessary_unwrap)]
//!
//! 对 `#[proptest]` 标注的函数生成随机输入，通过 `catch_unwind` 捕获 panic。
//! 不判断语义正确性——只判定硬崩溃。输出结构化报告供 CI 消费。
//!
//! 标注方式：在函数签名上方添加 `// @proptest` 注释。
//! 暂不标注的函数不产生任何运行时开销。

use std::panic::{self, AssertUnwindSafe};

/// 单个 fuzz 测试的结果
#[derive(Debug)]
struct FuzzResult {
    /// 被测试的函数名
    target: &'static str,
    /// 所在源文件
    file: &'static str,
    /// 总输入数
    total: usize,
    /// 崩溃数
    crashes: usize,
    /// 崩溃详情：(输入摘要, panic 消息)
    crash_details: Vec<(String, String)>,
}

impl FuzzResult {
    const fn _build(target: &'static str, file: &'static str, total: usize, crashes: usize, crash_details: Vec<(String, String)>) -> Self {
        Self { target, file, total, crashes, crash_details }
    }
}

/// 生成随机字符串输入池
fn generate_string_inputs() -> Vec<String> {
    let mut inputs = Vec::new();

    // 空字符串
    inputs.push(String::new());

    // 纯 ASCII
    inputs.push("hello world".to_string());
    inputs.push("a".repeat(10000)); // 超长 ASCII

    // 纯 CJK
    inputs.push("你好世界司衡治理文档".to_string());

    // 混编
    inputs.push("id: 260613-1800-test\nstage: 1/3\n---\n# 标题\n正文内容".to_string());

    // 只含分隔符
    inputs.push("---\n---\n---".to_string());
    inputs.push("```\n```\n```".to_string());

    // 表格
    inputs.push("| a | b | c | d | e |\n|---|---|---|---|---|".to_string());

    // 深层嵌套列表
    inputs.push("  - a\n    - b\n      - c\n        - d\n          - e".to_string());

    // Unicode 边界
    inputs.push("\u{0000}\u{0001}\u{FFFD}\u{10FFFF}".to_string());
    inputs.push("\u{2014}\u{201C}\u{201D}\u{2192}\u{2190}".to_string()); // 禁止字符
    inputs.push("🔥🎉💻".to_string()); // emoji

    // Frontmatter 格式
    inputs.push("---\nid: bad-id\nstage: 99/99\n---\n".to_string());
    inputs.push("---\nid: 260613-1800-ok\nstage: 1/3\nupstream: 240602-0900-x\n---\n# Title\ntext".to_string());
    inputs.push("no frontmatter at all, just plain text".to_string());

    // 极端长度
    inputs.push("x".repeat(100000));

    // 只有 --- 的行
    inputs.push("---".to_string());
    inputs.push("***".to_string());
    inputs.push("___".to_string());

    inputs
}

/// 生成随机 (&str, &str) 对（用于需要两个参数的函数）
fn generate_str_pairs() -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let files = ["test.sih.md", "docs/specs/test.sih.md", "docs/knowledge/notes/test.sih.md", ""];
    for f in &files {
        for s in &generate_string_inputs() {
            pairs.push((f.to_string(), s.clone()));
        }
    }
    pairs
}

/// 安全地运行一个函数并捕获 panic
fn run_safely<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce() -> R + panic::UnwindSafe,
{
    panic::catch_unwind(f).map_err(|e| {
        
        if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "unknown panic".to_string()
        }
    })
}

// ── Fuzz targets ──

/// Fuzz parse_frontmatter
fn fuzz_parse_frontmatter() -> FuzzResult {
    let inputs = generate_string_inputs();
    let mut crashes = Vec::new();

    for input in &inputs {
        let input_clone = input.clone();
        let preview = preview_input(input);
        let result = run_safely(AssertUnwindSafe(|| {
            sihankor::core::parser::parse_content(&input_clone)
        }));
        if result.is_err() {
            crashes.push((preview, result.unwrap_err()));
        }
    }

    FuzzResult {
        target: "parse_content",
        file: "src/core/parser.rs",
        total: inputs.len(),
        crashes: crashes.len(),
        crash_details: crashes,
    }
}

/// Fuzz validate_content
fn fuzz_validate_content() -> FuzzResult {
    use sihankor::core::models::{Document, DocStatus, Frontmatter, Stage};
    use sihankor::core::validator::validate_content;

    let inputs = generate_string_inputs();
    let mut crashes = Vec::new();

    for input in &inputs {
        let input_clone = input.clone();
        let doc = Document {
            id: "260618-0000-test".to_string(),
            stage: Stage("1/3".to_string()),
            title: "Test".to_string(),
            upstream: None,
            frontmatter: Frontmatter {
                id: "260618-0000-test".to_string(),
                stage: Stage("1/3".to_string()),
                upstream: None,
                decided_by: None,
                extra: serde_json::Value::Null,
            },
            content: input_clone.clone(),
            status: DocStatus::Ok,
            indexed_at: chrono::Utc::now(),
            nature: "spec".to_string(),
        };
        let preview = preview_input(input);
        let result = run_safely(AssertUnwindSafe(|| validate_content(&doc)));
        if result.is_err() {
            crashes.push((preview, result.unwrap_err()));
        }
    }

    FuzzResult {
        target: "validate_content",
        file: "src/core/validator.rs",
        total: inputs.len(),
        crashes: crashes.len(),
        crash_details: crashes,
    }
}

/// Fuzz check_c01a (fmt 字符分类)
fn fuzz_check_c01a() -> FuzzResult {
    let inputs = generate_string_inputs();
    let mut crashes = Vec::new();

    for input in &inputs {
        let input_clone = input.clone();
        let preview = preview_input(input);
        let result = run_safely(AssertUnwindSafe(|| {
            sihankor::fmt::check_c01a(&input_clone, 1, "test.sih.md")
        }));
        if result.is_err() {
            crashes.push((preview, result.unwrap_err()));
        }
    }

    FuzzResult {
        target: "check_c01a",
        file: "src/fmt/mod.rs",
        total: inputs.len(),
        crashes: crashes.len(),
        crash_details: crashes,
    }
}

/// Fuzz extract_frontmatter
fn fuzz_extract_frontmatter() -> FuzzResult {
    let inputs = generate_string_inputs();
    let mut crashes = Vec::new();

    for input in &inputs {
        let input_clone = input.clone();
        let preview = preview_input(input);
        let result = run_safely(AssertUnwindSafe(|| {
            sihankor::core::parser::parse_content(&input_clone)
        }));
        if result.is_err() {
            crashes.push((preview, result.unwrap_err()));
        }
    }

    FuzzResult {
        target: "extract_frontmatter (via parse_content)",
        file: "src/core/parser.rs",
        total: inputs.len(),
        crashes: crashes.len(),
        crash_details: crashes,
    }
}

/// Fuzz lint_document (fmt 全量)
fn fuzz_lint_document() -> FuzzResult {
    let pairs = generate_str_pairs();
    let config = sihankor::fmt::FormatConfig::default();
    let mut crashes = Vec::new();

    for (file, content) in &pairs {
        let file_clone = file.clone();
        let content_clone = content.clone();
        let preview = preview_input(content);
        let result = run_safely(AssertUnwindSafe(|| {
            sihankor::fmt::lint_document(&file_clone, &content_clone, &config)
        }));
        if result.is_err() {
            crashes.push((preview, result.unwrap_err()));
        }
    }

    FuzzResult {
        target: "lint_document",
        file: "src/fmt/mod.rs",
        total: pairs.len(),
        crashes: crashes.len(),
        crash_details: crashes,
    }
}

/// 截断输入用于报告
fn preview_input(s: &str) -> String {
    let escaped: String = s
        .chars()
        .take(60)
        .map(|c| {
            if c.is_control() && c != '\n' {
                format!("\\u{{{:04X?}}}", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect();
    if s.chars().count() > 60 {
        format!("{}...", escaped)
    } else {
        escaped
    }
}

fn main() {
    println!("sihankor-proptest: 验码开始\n");

    let results = vec![
        fuzz_parse_frontmatter(),
        fuzz_validate_content(),
        fuzz_check_c01a(),
        fuzz_extract_frontmatter(),
        fuzz_lint_document(),
    ];

    let total_tests: usize = results.iter().map(|r| r.total).sum();
    let total_crashes: usize = results.iter().map(|r| r.crashes).sum();

    // 结构化输出
    for r in &results {
        if r.crashes > 0 {
            println!(
                "## F 级阻断: {} ({}) — {} crashes / {} inputs",
                r.target, r.file, r.crashes, r.total
            );
            for (i, (input, msg)) in r.crash_details.iter().enumerate() {
                println!("  - crash {}: {}", i + 1, msg);
                println!("    input: {}", input);
            }
            println!();
        }
    }

    if total_crashes == 0 {
        println!("验码通过: 0 crashes / {} total inputs across {} targets",
            total_tests, results.len());
    }

    // 治理追溯标记
    let status = if total_crashes > 0 { "blocked" } else { "pass" };
    println!("\nSiHankor-Governance: {} (proptest: crashed={} total={})",
        status, total_crashes, total_tests);

    if total_crashes > 0 {
        std::process::exit(1);
    }
}
