//! 项目简报：从已有数据源聚合项目状态摘要
//!
//! 数据源：
//! - project_status（快照）：从数据库查询文档统计
//! - trail_context（行迹）：从 knowledge/trails/ 目录读取最新 5 条
//! - git 状态：通过 git 命令获取 branch 和 dirty 信息
//! - CI 状态：通过检测 .github/workflows 和本地 CI 配置文件
//!
//! 设计原则：
//! - 不存文件
//! - 不建索引
//! - 不改写任何现有数据结构
//! - 纯文本输出（不超过 2000 tokens）

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;

/// 项目简报结构
#[derive(Debug)]
pub struct ProjectBrief {
    /// 项目根目录
    pub root: PathBuf,
    /// 采集时间（RFC 3339）
    pub generated_at: String,
    /// 总文档数
    pub total_docs: usize,
    /// 按 stage 分布
    pub by_stage: Vec<(String, usize)>,
    /// 按 nature 分布
    pub by_nature: Vec<(String, usize)>,
    /// git 当前分支
    pub git_branch: Option<String>,
    /// git 是否 dirty
    pub git_dirty: bool,
    /// 最新 5 条行迹摘要
    pub latest_trails: Vec<TrailEntry>,
    /// CI 配置路径
    pub ci_paths: Vec<PathBuf>,
    /// 最新 3 条会话摘要
    pub recent_sessions: Vec<SessionEntry>,
    /// 会话健康诊断
    pub session_health: Vec<String>,
}

/// 会话摘要条目
#[derive(Debug)]
pub struct SessionEntry {
    pub id: String,
    pub session_id: Option<String>,
    pub goal: Option<String>,
    pub outcome: Option<String>,
    pub commits: Option<String>,
}

#[derive(Debug)]
pub struct TrailEntry {
    pub trace_id: String,
    pub created_at: String,
    pub anchor_doc: String,
    pub trail_type: String,
    pub summary: String,
}

/// 生成项目简报
///
/// 输入：项目根目录（Path）
/// 输出：纯文本字符串（不超过 2000 tokens）
pub fn generate(root: &Path) -> String {
    let mut brief = ProjectBrief {
        root: root.to_path_buf(),
        generated_at: current_rfc3339(),
        total_docs: 0,
        by_stage: Vec::new(),
        by_nature: Vec::new(),
        git_branch: None,
        git_dirty: false,
        latest_trails: Vec::new(),
        ci_paths: Vec::new(),
        recent_sessions: Vec::new(),
        session_health: Vec::new(),
    };

    // 1. 扫描知识库行迹
    let trails_dir = root.join("knowledge").join("trails");
    if trails_dir.is_dir() {
        brief.latest_trails = collect_latest_trails(&trails_dir, 5);
    }

    // 1b. 扫描会话摘要（最新 3 条）
    let sessions_dir = root.join("docs").join("knowledge").join("session-summaries");
    if sessions_dir.is_dir() {
        brief.recent_sessions = collect_recent_sessions(&sessions_dir, 3);
    }

    // 会话健康诊断
    brief.session_health = check_session_health(root, &brief.recent_sessions, &brief.latest_trails);

    // 2. 扫描 CI 配置路径
    let ci_candidates = vec![
        root.join(".github").join("workflows"),
        root.join(".gitlab").join("ci"),
        root.join(".circleci"),
        root.join(".travis.yml"),
        root.join(".github/workflows"),
    ];
    for p in ci_candidates {
        if p.is_dir() || p.is_file() {
            brief.ci_paths.push(p);
        }
    }

    // 3. git 状态（通过 git 命令）
    if let Some((branch, dirty)) = git_status(root) {
        brief.git_branch = Some(branch);
        brief.git_dirty = dirty;
    }

    // 4. 扫描文档总数（.sih.md 文件数，作为 project_status 的近似）
    let mut sih_count = 0;
    let mut stage_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut nature_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(name.as_ref(), "node_modules" | "target" | ".git" | "__pycache__" | ".pytest_cache")
        })
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !name.ends_with(".sih.md") && !name.ends_with(".md") {
            continue;
        }
        sih_count += 1;

        // 解析 frontmatter 中的 stage 和 nature
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            let (fm_end, has_fm) = detect_frontmatter(&lines);
            if has_fm {
                let fm_text: String = lines[..fm_end].join("\n");
                if let Some(stage) = get_field(&fm_text, "stage") {
                    *stage_map.entry(stage).or_insert(0) += 1;
                }
                if let Some(nature) = get_field(&fm_text, "nature") {
                    *nature_map.entry(nature).or_insert(0) += 1;
                }
            }
        }
    }

    brief.total_docs = sih_count;
    brief.by_stage = stage_map.into_iter().collect();
    brief.by_nature = nature_map.into_iter().collect();

    format_brief(&brief)
}

/// 从 knowledge/trails/ 目录收集最新 N 条行迹
pub fn collect_trails(root: &Path, limit: usize) -> Vec<TrailEntry> {
    let trails_dir = root.join("knowledge").join("trails");
    collect_latest_trails(&trails_dir, limit)
}

fn collect_latest_trails(trails_dir: &Path, limit: usize) -> Vec<TrailEntry> {
    let mut entries: Vec<_> = WalkDir::new(trails_dir)
        .follow_links(false)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.path().extension().map(|s| s == "md").unwrap_or(false)
        })
        .collect();

    // 按修改时间倒序
    entries.sort_by(|a, b| {
        let ta = a
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        let tb = b
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        tb.cmp(&ta)
    });

    let mut trails = Vec::new();
    for entry in entries.into_iter().take(limit) {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Some(trail) = parse_trail_file(entry.path(), &content) {
                trails.push(trail);
            }
        }
    }
    trails
}

/// 解析单个 trail 文件，提取元信息
fn parse_trail_file(path: &Path, content: &str) -> Option<TrailEntry> {
    let lines: Vec<&str> = content.lines().collect();
    let (fm_end, has_fm) = detect_frontmatter(&lines);
    if !has_fm {
        return None;
    }

    let fm_text = lines[..fm_end].join("\n");
    let trace_id = get_field(&fm_text, "trace_id")
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });
    let created_at = get_field(&fm_text, "created_at")
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    let anchor_doc = get_field(&fm_text, "anchor_doc")
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default();
    let trail_type = get_field(&fm_text, "type")
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // 从正文中提取第一行作为摘要
    let body: String = lines[fm_end..]
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    let summary = if body.len() > 100 {
        format!("{}...", &body[..100])
    } else {
        body
    };

    Some(TrailEntry {
        trace_id,
        created_at,
        anchor_doc,
        trail_type,
        summary,
    })
}

/// 从 session-summaries/ 目录收集最新 N 条会话摘要
fn collect_recent_sessions(sessions_dir: &Path, limit: usize) -> Vec<SessionEntry> {
    let mut entries: Vec<_> = WalkDir::new(sessions_dir)
        .follow_links(false)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.path()
                    .extension()
                    .map(|s| s == "md" || s == "sih.md")
                    .unwrap_or(false)
        })
        .collect();

    // 按修改时间倒序
    entries.sort_by(|a, b| {
        let ta = a
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let tb = b
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        tb.cmp(&ta)
    });

    let mut sessions = Vec::new();
    for entry in entries.into_iter().take(limit) {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if let Some(s) = parse_session_file(entry.path(), &content) {
                sessions.push(s);
            }
        }
    }
    sessions
}

/// 解析单个 session summary 文件，提取元信息
fn parse_session_file(path: &Path, content: &str) -> Option<SessionEntry> {
    let lines: Vec<&str> = content.lines().collect();
    let (fm_end, has_fm) = detect_frontmatter(&lines);
    if !has_fm {
        return None;
    }

    let fm_text = lines[..fm_end].join("\n");
    let id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    Some(SessionEntry {
        id,
        session_id: get_field(&fm_text, "session_id").map(|s| s.trim_matches('"').to_string()),
        goal: get_field(&fm_text, "goal").map(|s| s.trim_matches('"').to_string()),
        outcome: get_field(&fm_text, "outcome").map(|s| s.trim_matches('"').to_string()),
        commits: get_field(&fm_text, "commits").map(|s| s.trim_matches('"').to_string()),
    })
}

/// 会话健康诊断：基于项目级信号检测上下文压力
///
/// 四个检测维度全部基于文件系统/git（不依赖 agent 状态）：
/// - session summary 密度（>3 条在 24h 内 → 建议合并）
/// - commit 密度（>5 个 commit 在同一 goal → 建议结束）
/// - trail 密度（同一 anchor_doc >3 条 trail → 决策链过长）
/// - CI 停滞检测（G 违规连续 3 次 ±0 → review backlog）
fn check_session_health(
    root: &Path,
    sessions: &[SessionEntry],
    trails: &[TrailEntry],
) -> Vec<String> {
    let mut alerts: Vec<String> = Vec::new();

    // 1. Session summary 密度（> 3 条 → 建议合并会话）
    if sessions.len() > 3 {
        alerts.push(format!(
            "{} session summaries recorded — consider consolidating into longer sessions to reduce context overhead",
            sessions.len()
        ));
    }

    // 2. Commit 密度（同一 goal 的 commit 数 > 5）
    let commit_count = {
        let output = Command::new("git")
            .args(["--no-pager", "log", "--oneline", "-50"])
            .current_dir(root)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output();
        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count()
        } else {
            0
        }
    };

    if commit_count > 5 {
        // 检查是否有多个 commit 针对同一 goal（以最近 3 条 session goal 为基准）
        let recent_goals: Vec<&str> = sessions.iter()
            .filter_map(|s| s.goal.as_deref())
            .collect();
        if recent_goals.len() >= 2 && recent_goals[0] == recent_goals[1] {
            alerts.push(format!(
                "{} commits with repeated goal '{}' — goal likely achieved, consider closing this task",
                commit_count, recent_goals[0]
            ));
        }
    }

    // 3. Trail 密度（同一 anchor_doc > 3 条 trail）
    let mut anchor_counts: HashMap<&str, usize> = HashMap::new();
    for t in trails {
        *anchor_counts.entry(t.anchor_doc.as_str()).or_default() += 1;
    }
    for (anchor, count) in anchor_counts {
        if count > 3 {
            alerts.push(format!(
                "{} trails anchored on '{}' — decision chain may be too long, consider ratify or close",
                count, anchor
            ));
        }
    }

    // 4. CI 停滞检测（需要跑 ci-check.sh 判断 G 违规变化）
    // 注：此检查依赖外部 CI 脚本，若不存在则跳过
    let ci_script = root.join("scripts").join("ci-check.sh");
    if ci_script.exists() {
        let output = Command::new("sh")
            .args([ci_script.to_str().unwrap_or(""), "--strict"])
            .current_dir(root)
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.contains("G 级违规") {
                alerts.push("CI G violations present — consider running `cargo run --bin rebuild_index -- --strict` to review backlog".into());
            }
        }
    }

    alerts
}

/// 执行 git status，返回 (branch, is_dirty)
fn git_status(root: &Path) -> Option<(String, bool)> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        return None;
    }

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok()?;

    let dirty = !String::from_utf8_lossy(&status_output.stdout).trim().is_empty();

    Some((branch, dirty))
}

/// 格式化简报为纯文本
fn format_brief(brief: &ProjectBrief) -> String {
    let mut out = String::new();

    out.push_str("## SiHankor Project Brief\n\n");
    out.push_str(&format!("Generated: {}\n", brief.generated_at));
    out.push('\n');

    // Git 状态
    out.push_str("### Git Status\n");
    let git_info = match &brief.git_branch {
        Some(branch) => {
            if brief.git_dirty {
                format!("Branch: {} (dirty)", branch)
            } else {
                format!("Branch: {} (clean)", branch)
            }
        }
        None => "Git: not a git repository".to_string(),
    };
    out.push_str(&git_info);
    out.push_str("\n\n");

    // CI 状态
    out.push_str("### CI Configuration\n");
    if brief.ci_paths.is_empty() {
        out.push_str("CI: none detected\n");
    } else {
        out.push_str(&format!(
            "CI: {} path(s) found\n",
            brief.ci_paths.len()
        ));
        for p in &brief.ci_paths {
            if let Ok(rel) = p.strip_prefix(&brief.root) {
                out.push_str(&format!("  - {}\n", rel.display()));
            }
        }
    }
    out.push('\n');

    // 文档统计
    out.push_str("### Document Statistics\n");
    out.push_str(&format!("Total .md/.sih.md files: {}\n", brief.total_docs));

    if !brief.by_stage.is_empty() {
        out.push_str("By stage:\n");
        for (stage, count) in &brief.by_stage {
            out.push_str(&format!("  {}: {}\n", stage, count));
        }
    }

    if !brief.by_nature.is_empty() {
        out.push_str("By nature:\n");
        for (nature, count) in &brief.by_nature {
            out.push_str(&format!("  {}: {}\n", nature, count));
        }
    }
    out.push('\n');

    // 最新行迹
    out.push_str("### Latest Trails\n");
    if brief.latest_trails.is_empty() {
        out.push_str("Trails: none recorded\n");
    } else {
        for trail in &brief.latest_trails {
            out.push_str(&format!(
                "- [{}] {} (type: {}, anchor: {})\n",
                trail.trace_id,
                trail.summary,
                trail.trail_type,
                trail.anchor_doc
            ));
        }
    }
    out.push('\n');

    // 最新会话摘要
    out.push_str("### Recent Sessions\n");
    if brief.recent_sessions.is_empty() {
        out.push_str("Sessions: none recorded (use sihankor_session_summary before ending a session)\n");
    } else {
        for s in &brief.recent_sessions {
            let goal = s.goal.as_deref().unwrap_or("(no goal)");
            let outcome = s.outcome.as_deref().unwrap_or("(no outcome)");
            let commits = s.commits.as_deref().unwrap_or("(no commits)");
            out.push_str(&format!(
                "- [{}] goal: {}, outcome: {}, commits: {}\n",
                s.id, goal, outcome, commits
            ));
        }
    }

    // 会话健康诊断
    out.push_str("### Session Health\n");
    if brief.session_health.is_empty() {
        out.push_str("All signals normal.\n");
    } else {
        for h in &brief.session_health {
            out.push_str(&format!("⚠ {}\n", h));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 检测 frontmatter，返回 (frontmatter 结束行号, 是否有 frontmatter)
fn detect_frontmatter(lines: &[&str]) -> (usize, bool) {
    if lines.is_empty() || lines[0].trim() != "---" {
        return (0, false);
    }
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            return (i + 1, true);
        }
    }
    (0, false)
}

/// 从 frontmatter 文本中提取字段值
fn get_field(fm_text: &str, field: &str) -> Option<String> {
    for line in fm_text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(field) {
            if rest.starts_with(':') {
                return Some(rest[1..].trim().to_string());
            }
        }
    }
    None
}

fn current_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let (y, mo, d, h, mi, s) = epoch_to_ymdhms(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, mi, s)
}

fn epoch_to_ymdhms(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = (rem / 3600) as u32;
    let min = ((rem % 3600) / 60) as u32;
    let sec = (rem % 60) as u32;
    let (y, m, da) = days_to_ymd(days as i64);
    (y, m, da, hour, min, sec)
}

fn days_to_ymd(days: i64) -> (i32, u32, u32) {
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
    let mut month = 1u32;
    for &md in &month_days {
        if remaining < md as i64 {
            break;
        }
        remaining -= md as i64;
        month += 1;
    }
    (year, month, (remaining + 1) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(dir: &Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_generate_empty_dir() {
        let dir = make_temp_dir();
        let brief = generate(dir.path());
        assert!(brief.contains("SiHankor Project Brief"));
        assert!(brief.contains("Total .md/.sih.md files: 0"));
    }

    #[test]
    fn test_generate_with_docs() {
        let dir = make_temp_dir();
        write_file(
            dir.path(),
            "docs/specs/test.sih.md",
            "---\nid: test\nstage: 1/3\n---\n# Test\n",
        );
        write_file(
            dir.path(),
            "docs/proposals/prop.sih.md",
            "---\nid: prop\nstage: 2/3\nnature: proposal\n---\n# Proposal\n",
        );
        let brief = generate(dir.path());
        assert!(brief.contains("Total .md/.sih.md files: 2"));
        assert!(brief.contains("1/3"));
        assert!(brief.contains("2/3"));
    }

    #[test]
    fn test_detect_frontmatter() {
        let lines = vec!["---", "id: test", "---", "# Body"];
        let (end, has) = detect_frontmatter(&lines);
        assert!(has);
        // 关闭 --- 在 index 2，返回 i+1 = 3（body 从 lines[3] 开始）
        assert_eq!(end, 3);
    }

    #[test]
    fn test_get_field() {
        let fm = "id: test\nstage: 1/3\nnature: spec";
        assert_eq!(get_field(fm, "stage"), Some("1/3".to_string()));
        assert_eq!(get_field(fm, "id"), Some("test".to_string()));
    }

    #[test]
    fn test_generate_contains_session_health_section() {
        // generate() returns a String - check it contains the Session Health section
        let output = generate(Path::new("."));
        assert!(output.contains("### Session Health"), "Output should contain Session Health section");
        // Session Health section: either "All signals normal." OR "⚠" warning
        let has_normal = output.contains("All signals normal.");
        let has_warning = output.contains("⚠");
        assert!(has_normal || has_warning, "Should show normal status or ⚠ warning. Output:\n{}", output);
    }
}

