#![allow(clippy::print_stdout)]
//! 观测窗 CLI：让司衡看见陌生项目
//!
//! 用法：
//!   sihankor-observe <path> [--format text|json] [--max-depth N]
//!
//! 退出码：
//!   0 — 成功完成扫描
//!   1 — 路径不存在或不可读
//!   2 — 命令行参数错误

use std::path::PathBuf;
use std::process::ExitCode;

use sihankor::observe::{ProjectObservation, RulePredictions, predict, scan_project};

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

struct Cli {
    path: PathBuf,
    format: OutputFormat,
    max_depth: Option<usize>,
}

fn parse_args() -> Result<Cli, String> {
    let mut path: Option<PathBuf> = None;
    let mut format = OutputFormat::Text;
    let mut max_depth: Option<usize> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--format" => {
                let val = args.next().ok_or("--format requires a value")?;
                format = match val.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    other => return Err(format!("invalid --format value: {}", other)),
                };
            }
            "--max-depth" => {
                let val = args.next().ok_or("--max-depth requires a value")?;
                max_depth = Some(val.parse().map_err(|_| "invalid --max-depth value")?);
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other if !other.starts_with("--") => {
                if path.is_some() {
                    return Err(format!("unexpected positional argument: {}", other));
                }
                path = Some(PathBuf::from(other));
            }
            other => return Err(format!("unknown argument: {}", other)),
        }
    }

    let path = path.ok_or("missing required argument: <path>")?;
    Ok(Cli {
        path,
        format,
        max_depth,
    })
}

fn print_help() {
    println!("sihankor-observe — 观测窗入口");
    println!();
    println!("USAGE:");
    println!("    sihankor-observe <path> [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --format <FMT>     输出格式：text（默认）| json");
    println!("    --max-depth <N>    最大目录深度（默认无限制）");
    println!("    -h, --help         显示此帮助");
    println!();
    println!("EXAMPLE:");
    println!("    sihankor-observe /path/to/stranger/project");
    println!("    sihankor-observe . --format json");
}

fn main() -> ExitCode {
    let cli = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("argument error: {}", e);
            eprintln!();
            print_help();
            return ExitCode::from(2);
        }
    };

    if !cli.path.exists() {
        eprintln!("path does not exist: {}", cli.path.display());
        return ExitCode::from(1);
    }

    let obs = match scan_project(&cli.path) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("scan failed: {}", e);
            return ExitCode::from(1);
        }
    };

    let predictions = predict(&obs);

    match cli.format {
        OutputFormat::Text => render_text(&obs, &predictions, cli.max_depth),
        OutputFormat::Json => render_json(&obs, &predictions, cli.max_depth),
    }

    ExitCode::SUCCESS
}

fn render_text(obs: &ProjectObservation, pred: &RulePredictions, max_depth: Option<usize>) {
    println!("Observation Window Report");
    println!("==========================");
    println!("Root: {}", obs.root.display());
    println!("Scanned at: {}", obs.scanned_at);
    println!("Skipped dirs: {}", obs.skipped_dirs.join(", "));
    println!();

    println!("[1] File Stats");
    println!("    total_files:    {}", obs.file_stats.total_files);
    println!("    total_bytes:    {}", obs.file_stats.total_bytes);
    println!("    total_lines:    {}", obs.file_stats.total_lines);
    println!("    avg_lines/file: {:.1}", obs.file_stats.avg_lines);
    println!();

    println!("[2] Directory Depth Distribution");
    let mut depths: Vec<_> = obs.dir_depth_distribution.keys().collect();
    depths.sort();
    if let Some(max) = max_depth {
        depths.retain(|d| **d <= max);
    }
    for d in depths {
        let count = obs.dir_depth_distribution[d];
        println!("    depth {}: {} files", d, count);
    }
    println!();

    println!("[3] Table Column Distribution");
    println!("    total_tables: {}", obs.table_stats.total_tables);
    let mut cols: Vec<_> = obs.table_stats.column_distribution.keys().collect();
    cols.sort();
    for c in cols {
        let count = obs.table_stats.column_distribution[c];
        println!("    {} cols: {} tables", c, count);
    }
    println!();

    println!("[4] Code Block Language Tag Coverage");
    println!("    total_blocks:   {}", obs.code_block_stats.total_blocks);
    println!("    with_lang:      {}", obs.code_block_stats.with_lang);
    println!("    without_lang:   {}", obs.code_block_stats.without_lang);
    let coverage = if obs.code_block_stats.total_blocks > 0 {
        100.0 * obs.code_block_stats.with_lang as f64 / obs.code_block_stats.total_blocks as f64
    } else {
        100.0
    };
    println!("    coverage:       {:.1}%", coverage);
    println!();

    println!("[5] Frontmatter Stats");
    println!(
        "    files_with_frontmatter:    {}",
        obs.frontmatter_stats.files_with_frontmatter
    );
    println!(
        "    files_with_id_field:       {}",
        obs.frontmatter_stats.files_with_id_field
    );
    println!(
        "    files_with_stage_field:    {}",
        obs.frontmatter_stats.files_with_stage_field
    );
    println!(
        "    files_with_sihankor_stage: {}",
        obs.frontmatter_stats.files_with_sihankor_stage
    );
    println!();

    println!("[P] Rule Trigger Predictions (if SiHankor governance introduced)");
    println!("    V-F-01 (id 必填):          {}", pred.v_f01_predicted);
    println!("    V-F-05 (禁止 --- 水平线):  {}", pred.v_f05_predicted);
    println!(
        "    V-G-04 (表格 ≤ 3 列):      {} (files with wide tables)",
        pred.v_g04_predicted
    );
    println!(
        "    V-G-05 (代码块需 lang):    {} (blocks without lang)",
        pred.v_g05_predicted
    );
    println!(
        "    V-G-06 (禁止 emoji):       {} (lines with emoji)",
        pred.v_g06_predicted
    );
    println!();
    println!("    Summary:");
    println!("      F predicted total: {}", pred.f_predicted_total);
    println!("      G predicted total: {}", pred.g_predicted_total);
    println!(
        "      J predicted total: {} (V-J-01 not yet predicted in MVP)",
        pred.j_predicted_total
    );
}

fn render_json(obs: &ProjectObservation, pred: &RulePredictions, max_depth: Option<usize>) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct JsonReport<'a> {
        root: &'a std::path::Path,
        scanned_at: &'a str,
        skipped_dirs: &'a [String],
        file_stats: &'a sihankor::observe::FileStats,
        dir_depth_distribution: Vec<(usize, usize)>,
        table_stats: JsonTableStats<'a>,
        code_block_stats: &'a sihankor::observe::CodeBlockStats,
        frontmatter_stats: JsonFrontmatterStats<'a>,
        horizontal_rule_count: usize,
        emoji_line_count: usize,
        predictions: &'a sihankor::observe::RulePredictions,
        max_depth_filter: Option<usize>,
    }

    #[derive(Serialize)]
    struct JsonTableStats<'a> {
        total_tables: usize,
        column_distribution: Vec<(usize, usize)>,
        files_with_wide_table_count: usize,
        files_with_wide_table: Vec<&'a std::path::Path>,
    }

    #[derive(Serialize)]
    struct JsonFrontmatterStats<'a> {
        files_with_frontmatter: usize,
        files_with_id_field: usize,
        files_with_stage_field: usize,
        files_with_sihankor_stage: usize,
        stage_without_id_count: usize,
        stage_without_id: Vec<&'a std::path::Path>,
    }

    let mut dir_pairs: Vec<_> = obs
        .dir_depth_distribution
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    if let Some(max) = max_depth {
        dir_pairs.retain(|(d, _)| *d <= max);
    }
    dir_pairs.sort_by_key(|(d, _)| *d);

    let mut col_pairs: Vec<_> = obs
        .table_stats
        .column_distribution
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    col_pairs.sort_by_key(|(c, _)| *c);

    let report = JsonReport {
        root: &obs.root,
        scanned_at: &obs.scanned_at,
        skipped_dirs: &obs.skipped_dirs,
        file_stats: &obs.file_stats,
        dir_depth_distribution: dir_pairs,
        table_stats: JsonTableStats {
            total_tables: obs.table_stats.total_tables,
            column_distribution: col_pairs,
            files_with_wide_table_count: obs.table_stats.files_with_wide_table.len(),
            files_with_wide_table: obs
                .table_stats
                .files_with_wide_table
                .iter()
                .map(|p| p.as_path())
                .collect(),
        },
        code_block_stats: &obs.code_block_stats,
        frontmatter_stats: JsonFrontmatterStats {
            files_with_frontmatter: obs.frontmatter_stats.files_with_frontmatter,
            files_with_id_field: obs.frontmatter_stats.files_with_id_field,
            files_with_stage_field: obs.frontmatter_stats.files_with_stage_field,
            files_with_sihankor_stage: obs.frontmatter_stats.files_with_sihankor_stage,
            stage_without_id_count: obs.frontmatter_stats.stage_without_id.len(),
            stage_without_id: obs
                .frontmatter_stats
                .stage_without_id
                .iter()
                .map(|p| p.as_path())
                .collect(),
        },
        horizontal_rule_count: obs.horizontal_rule_count,
        emoji_line_count: obs.emoji_line_count,
        predictions: pred,
        max_depth_filter: max_depth,
    };

    match serde_json::to_string_pretty(&report) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("failed to serialize report: {}", e),
    }
}
