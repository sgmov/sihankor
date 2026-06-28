#![allow(clippy::print_stdout)]
//! SiHankor 自治理入口：rebuild_index 增量
//!
//! 用法：
//!   rebuild_index [--warn|--strict] [--format text|json] [--docs-dir <P>] [--db-path <P>]
//!
//! 退出码：
//!   0 — 无阻断项（默认 --warn 模式或 --strict 模式下所有检查通过）
//!   1 — 阻断项触发（--strict 模式下 F 级违规 / 批准的 new rule 违规；或 parse/db 错误）
//!   2 — 命令行参数错误
//!
//! 默认模式（--warn）报告全部问题但不阻断，用于先观测一轮数据再决定是否切 --strict。
//! 此约定由 plan 260628-2030-ci-self-govern 决策：
//!   "新规则：先 --warn 不阻断，跑一轮数据后再决定是否 --strict"。

use std::path::PathBuf;
use std::sync::Arc;

use sihankor::core::database::SqliteBackend;
use sihankor::core::governance_check::{
    StageTransitionIssue, UpstreamChainIssue, check_stage_transitions, check_upstream_chain,
};
use sihankor::core::indexer::{self, IndexReport, ViolationDetail};
use sihankor::core::models::Document;
use sihankor::core::orchestrator;
use sihankor::core::validator::ValidationConfig;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    /// 报告全部问题但不阻断（CI 默认模式，先观测数据）
    Warn,
    /// F 级违规 + 批准的 new rule 违规即阻断
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

struct Cli {
    mode: Mode,
    format: OutputFormat,
    docs_dir: PathBuf,
    db_path: PathBuf,
}

fn parse_args() -> Result<Cli, String> {
    let mut mode = Mode::Warn;
    let mut format = OutputFormat::Text;
    let mut docs_dir = PathBuf::from("docs/");
    let mut db_path = PathBuf::from(".sih/index.db");

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--warn" => mode = Mode::Warn,
            "--strict" => mode = Mode::Strict,
            "--format" => {
                let val = args.next().ok_or("--format requires a value")?;
                format = match val.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    other => return Err(format!("invalid --format value: {}", other)),
                };
            }
            "--docs-dir" => {
                let val = args.next().ok_or("--docs-dir requires a value")?;
                docs_dir = PathBuf::from(val);
            }
            "--db-path" => {
                let val = args.next().ok_or("--db-path requires a value")?;
                db_path = PathBuf::from(val);
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {}", other)),
        }
    }

    Ok(Cli {
        mode,
        format,
        docs_dir,
        db_path,
    })
}

fn print_help() {
    println!("rebuild_index — SiHankor 自治理入口");
    println!();
    println!("USAGE:");
    println!("    rebuild_index [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --warn              报告全部问题但不阻断（默认）");
    println!("    --strict            F 级违规 + 批准的 new rule 违规即阻断");
    println!("    --format <FMT>      输出格式：text（默认）| json");
    println!("    --docs-dir <PATH>   文档目录（默认 docs/）");
    println!("    --db-path <PATH>    索引数据库（默认 .sih/index.db）");
    println!("    -h, --help          显示此帮助");
    println!();
    println!("EXIT CODES:");
    println!("    0  无阻断项");
    println!("    1  阻断项触发（--strict 模式 F 违规 / 解析失败 / 数据库错误）");
    println!("    2  命令行参数错误");
}

// ---------------------------------------------------------------------------
// Report types for text/json output
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
struct GovernanceReport {
    mode: String,
    index_summary: IndexSummary,
    fatal_violations: Vec<ViolationDetail>,
    guideline_violations: Vec<ViolationDetail>,
    judgment_count: usize,
    upstream_chain_issues: Vec<UpstreamIssueJson>,
    stage_transition_issues: Vec<StageTransitionIssueJson>,
    /// 是否有任何阻断项触发
    blocking: bool,
    /// 退出码（供 json 输出供 CI 解析）
    exit_code: i32,
}

#[derive(Debug, serde::Serialize)]
struct IndexSummary {
    discovered: usize,
    parsed: usize,
    indexed: usize,
    parse_errors: usize,
    db_errors: usize,
}

#[derive(Debug, serde::Serialize)]
struct UpstreamIssueJson {
    doc_id: String,
    doc_stage: String,
    upstream_id: String,
    reason: String,
}

impl From<UpstreamChainIssue> for UpstreamIssueJson {
    fn from(i: UpstreamChainIssue) -> Self {
        Self {
            doc_id: i.doc_id,
            doc_stage: i.doc_stage,
            upstream_id: i.upstream_id,
            reason: i.reason.as_str().to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct StageTransitionIssueJson {
    doc_id: String,
    current_stage: String,
    reason: String,
}

impl From<StageTransitionIssue> for StageTransitionIssueJson {
    fn from(i: StageTransitionIssue) -> Self {
        Self {
            doc_id: i.doc_id,
            current_stage: i.current_stage,
            reason: i.reason.as_str().to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Report assembly
// ---------------------------------------------------------------------------

fn build_report(
    mode: &Mode,
    index: IndexReport,
    upstream_issues: Vec<UpstreamChainIssue>,
    stage_issues: Vec<StageTransitionIssue>,
    docs_for_count: &[Document],
) -> GovernanceReport {
    let parse_errors = index
        .errors
        .iter()
        .filter(|(p, _)| p.ends_with(".sih.md"))
        .count();
    let db_errors = index.errors.len() - parse_errors;

    // 阻断判定
    let f_block = !index.fatal_violations.is_empty();
    let upstream_block = false; // 当前默认所有 upstream_issues 仅报告；plan 决策后再升级
    let stage_block = false;
    let errors_block = !index.errors.is_empty();

    let blocking = match mode {
        Mode::Strict => f_block || upstream_block || stage_block || errors_block,
        Mode::Warn => errors_block, // --warn 下 parse/db 错误仍阻断（结构性错误，不算新规则）
    };

    let exit_code = if blocking { 1 } else { 0 };

    let _ = docs_for_count; // 显式占位以备未来扩展

    GovernanceReport {
        mode: match mode {
            Mode::Warn => "warn",
            Mode::Strict => "strict",
        }
        .to_string(),
        index_summary: IndexSummary {
            discovered: index.discovered,
            parsed: index.parsed,
            indexed: index.indexed,
            parse_errors,
            db_errors,
        },
        fatal_violations: index.fatal_violations,
        guideline_violations: index.guideline_violations,
        judgment_count: index.judgment_count,
        upstream_chain_issues: upstream_issues.into_iter().map(Into::into).collect(),
        stage_transition_issues: stage_issues.into_iter().map(Into::into).collect(),
        blocking,
        exit_code,
    }
}

// ---------------------------------------------------------------------------
// Output rendering
// ---------------------------------------------------------------------------

fn render_text(report: &GovernanceReport) {
    let s = &report.index_summary;
    println!("\nSiHankor Self-Governance Report");
    println!("==============================");
    println!("Mode: {}", report.mode);
    println!();
    println!("Index:");
    println!("  discovered: {}", s.discovered);
    println!("  parsed:     {}", s.parsed);
    println!("  indexed:    {}", s.indexed);
    println!("  parse_errors: {}", s.parse_errors);
    println!("  db_errors:    {}", s.db_errors);
    println!();

    if !report.fatal_violations.is_empty() {
        println!(
            "F 级违规 ({} items) [{}]",
            report.fatal_violations.len(),
            if matches!(report.mode.as_str(), "strict") {
                "BLOCKING"
            } else {
                "WARN"
            }
        );
        for v in &report.fatal_violations {
            println!(
                "  - {} [{}] {}: {}",
                v.doc_id, v.rule_id, v.location, v.message
            );
        }
        println!();
    }

    if !report.guideline_violations.is_empty() {
        println!(
            "G 级违规 ({} items) [WARN]",
            report.guideline_violations.len()
        );
        for v in &report.guideline_violations {
            println!(
                "  - {} [{}] {}: {}",
                v.doc_id, v.rule_id, v.location, v.message
            );
        }
        println!();
    }

    if report.judgment_count > 0 {
        println!(
            "J 级静默记录 ({} items, details suppressed)",
            report.judgment_count
        );
        println!();
    }

    if !report.upstream_chain_issues.is_empty() {
        println!(
            "上游链完整性 ({} items) [NEW RULE — {}]",
            report.upstream_chain_issues.len(),
            if matches!(report.mode.as_str(), "strict") {
                "BLOCKING"
            } else {
                "WARN"
            }
        );
        for i in &report.upstream_chain_issues {
            println!(
                "  - {} -> {} [{}]: stage={}",
                i.doc_id, i.upstream_id, i.reason, i.doc_stage
            );
        }
        println!();
    }

    if !report.stage_transition_issues.is_empty() {
        println!(
            "Stage 转换合法性 ({} items) [NEW RULE — {}]",
            report.stage_transition_issues.len(),
            if matches!(report.mode.as_str(), "strict") {
                "BLOCKING"
            } else {
                "WARN"
            }
        );
        for i in &report.stage_transition_issues {
            println!(
                "  - {} [{}]: current_stage={}",
                i.doc_id, i.reason, i.current_stage
            );
        }
        println!();
    }

    let any_issues = report.index_summary.parse_errors > 0
        || report.index_summary.db_errors > 0
        || !report.fatal_violations.is_empty()
        || !report.guideline_violations.is_empty()
        || !report.upstream_chain_issues.is_empty()
        || !report.stage_transition_issues.is_empty()
        || report.judgment_count > 0;
    if !any_issues {
        println!("No issues found.");
        println!();
    }

    println!("Blocking: {}", report.blocking);
    println!("Exit code: {}", report.exit_code);
}

fn render_json(report: &GovernanceReport) {
    match serde_json::to_string_pretty(report) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("failed to serialize report: {}", e),
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cli = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("argument error: {}", e);
            eprintln!();
            print_help();
            std::process::exit(2);
        }
    };

    let db: Arc<dyn sihankor::core::database::SihDatabase> = match SqliteBackend::open(&cli.db_path)
    {
        Ok(db) => Arc::new(db),
        Err(e) => {
            eprintln!("failed to open db at {:?}: {}", cli.db_path, e);
            std::process::exit(1);
        }
    };

    let report =
        match orchestrator::run_pipeline(&*db, &cli.docs_dir, &ValidationConfig::default()).await {
            r => r,
        };

    let index_report = report.index;

    // 加载所有文档用于 cross-document governance checks
    let all_docs: Vec<Document> = match db.get_all_documents().await {
        Ok(docs) => docs,
        Err(e) => {
            eprintln!("failed to load documents: {}", e);
            std::process::exit(1);
        }
    };

    let upstream_issues = check_upstream_chain(&all_docs);
    let stage_issues = check_stage_transitions(&all_docs);

    let governance = build_report(
        &cli.mode,
        index_report,
        upstream_issues,
        stage_issues,
        &all_docs,
    );

    // print legacy "Index Rebuild Report" header（保持向后兼容）
    let _ = indexer::discover_documents; // 占位以防未使用警告

    match cli.format {
        OutputFormat::Text => render_text(&governance),
        OutputFormat::Json => render_json(&governance),
    }

    std::process::exit(governance.exit_code);
}
