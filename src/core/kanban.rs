use serde::Serialize;

use crate::core::database::SihDatabase;
use crate::core::models::DocStatus;

// ---------------------------------------------------------------------------
// 看板数据模型
// ---------------------------------------------------------------------------

/// 看板顶层结构
#[derive(Debug, Clone, Serialize)]
pub struct Kanban {
    /// 生成时间 (UTC)
    pub generated_at: String,
    /// 治理链阶段列
    pub columns: Vec<KanbanColumn>,
    /// 道四：已知盲区
    pub limitations: Vec<String>,
    /// 摘要统计
    pub summary: KanbanSummary,
}

/// 看板列：对应治理链的一个阶段
#[derive(Debug, Clone, Serialize)]
pub struct KanbanColumn {
    /// 列标识：draft | propose | decide | spec | code | verify
    pub phase: String,
    /// 中文标签
    pub label: String,
    /// 准入条件描述
    pub entry_condition: String,
    /// 在制品限制（0 表示无限制）
    pub wip_limit: u32,
    /// 当前卡片数
    pub card_count: usize,
    /// 卡片列表
    pub cards: Vec<KanbanCard>,
    /// verify 列的分组视图（仅 verify 列使用，按 nature 聚合，仅展开有问题项）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<VerifyGroup>>,
}

/// verify 列分组：按 nature 聚合 3/3 文档，仅展开 Warning/Error 详情
#[derive(Debug, Clone, Serialize)]
pub struct VerifyGroup {
    /// nature 分类
    pub nature: String,
    /// 该分类下正常（Ok）的卡片数
    pub clean_count: usize,
    /// 有问题需关注的卡片（Warning/Error）：展开完整详情
    pub attention: Vec<KanbanCard>,
}

/// 看板卡片：文档或代码任务的统一表示
#[derive(Debug, Clone, Serialize)]
pub struct KanbanCard {
    /// 唯一标识（文档 ID 或代码模块路径）
    pub id: String,
    /// 卡片标题
    pub title: String,
    /// 实体类型：document | code_task
    pub entity_type: String,
    /// 当前 stage 编码
    pub stage: String,
    /// 状态：Ok | Warning | Error（文档）/ pending | in_progress | done（代码）
    pub status: String,
    /// 文档 nature（仅文档卡片）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nature: Option<String>,
    /// 一句话概述：它是什么、当前处在什么阶段、为什么重要
    pub summary: String,
    /// 阻塞项列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<String>,
    /// 依赖卡片 ID
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

/// 看板摘要统计
#[derive(Debug, Clone, Serialize)]
pub struct KanbanSummary {
    pub total_cards: usize,
    pub by_phase: Vec<(String, usize)>,
    pub active_cards: usize,
    pub blocked_cards: usize,
    pub done_cards: usize,
}

// ---------------------------------------------------------------------------
// 治理链阶段定义（来源：Dev-Governance 五阶段链 + Canon stage 状态机）
// ---------------------------------------------------------------------------

const COLUMNS_DEF: &[(&str, &str, &str, u32)] = &[
    (
        "draft",
        "草稿",
        "想法碎片，尚未形成 proposal。对应 knowledge/drafts/ 或 note 1/3",
        0,
    ),
    (
        "propose",
        "提案",
        "proposal stage 1/3：方案待讨论。准入条件：grill-me 三条件满足",
        0,
    ),
    (
        "decide",
        "决议",
        "proposal/decision stage 2/3：方案已评审，待定稿。准入条件：decision 文档存在",
        0,
    ),
    (
        "spec",
        "规约",
        "spec stage 2/3→3/3：设计规约。准入条件：decision 3/3 已就位",
        0,
    ),
    (
        "code",
        "代码",
        "src/ 实现中。准入条件：spec 2/3+ 且 cargo check 通过",
        0,
    ),
    (
        "verify",
        "验证",
        "spec↔code fidelity 闭合。准入条件：test pass + semantic.yml 映射完整",
        0,
    ),
];

// ---------------------------------------------------------------------------
// 卡片分类逻辑
// ---------------------------------------------------------------------------

/// nature 的中文角色说明
fn nature_to_role(nature: &str) -> &'static str {
    match nature {
        "spec" => "技术规约（定义系统是什么）",
        "proposal" => "变更提案（提议要改什么）",
        "decision" => "架构决策（为什么这样选）",
        "reference" => "参照文档（术语和概念定义）",
        "note" => "实践笔记（从工程中提炼的经验）",
        _ => "文档",
    }
}

/// stage 的含义说明（根据 nature 不同，stage 的语义不同）
fn stage_to_description(stage: &str, nature: &str) -> &'static str {
    match (stage, nature) {
        ("3/3", _) => "已定稿：治理权威确立，可作为下游依据",
        ("2/3", "proposal") => "方案已评审通过，等待转化为 decision",
        ("2/3", "spec") => "设计已收敛，等待代码实现验证后定稿（ratify）",
        ("2/3", _) => "内容已收敛，等待最终确认后定稿",
        ("1/3", "proposal") => "方案草案：待评审讨论",
        ("1/3", "spec") => "设计草案：待收敛为稳定规约",
        ("1/3", _) => "草稿阶段：内容未收敛",
        _ => "",
    }
}

/// 从文档的 nature + stage 判定它属于哪个看板列
fn classify_document(nature: &str, stage: &str, _status: &DocStatus) -> &'static str {
    // 3/3 文档：已完成，进入 verify 列
    if stage == "3/3" {
        return "verify";
    }

    // 终止态：不显示在看板上
    if stage == "X" || stage.starts_with("0/") {
        return "archive";
    }

    // 非 3/3 文档：按治理链阶段分类
    match (nature, stage) {
        ("proposal", "1/3") => "propose",
        ("proposal", "2/3") => "decide",
        ("decision", "1/3") | ("decision", "2/3") => "decide",
        ("spec", "1/3") | ("spec", "2/3") => "spec",
        ("reference", "1/3") | ("reference", "2/3") => "spec",
        ("note", "1/3") | ("note", "2/3") => "draft",
        _ => "draft",
    }
}

/// 生成看板
pub async fn generate_kanban(db: &dyn SihDatabase) -> Kanban {
    let mut columns: Vec<KanbanColumn> = COLUMNS_DEF
        .iter()
        .map(|(phase, label, entry_condition, wip_limit)| KanbanColumn {
            phase: phase.to_string(),
            label: label.to_string(),
            entry_condition: entry_condition.to_string(),
            wip_limit: *wip_limit,
            card_count: 0,
            cards: Vec::new(),
            groups: None,
        })
        .collect();

    let mut limitations: Vec<String> = Vec::new();
    let mut total = 0usize;
    let mut blocked = 0usize;
    let mut done_count = 0usize;

    // 从数据库获取所有文档
    let all_docs = db.get_all_documents().await.unwrap_or_default();

    for doc in &all_docs {
        let phase = classify_document(&doc.nature, doc.stage.as_str(), &doc.status);
        if phase == "archive" {
            continue; // 终止态文档不进入看板
        }
        total += 1;

        // 构建卡片
        let mut blockers: Vec<String> = Vec::new();
        let mut depends_on: Vec<String> = Vec::new();

        // 阻塞检测：Error 状态
        if doc.status == DocStatus::Error {
            blockers.push("validation error: document has fatal validation issues".to_string());
            blocked += 1;
        }

        // 上游依赖：非 root 文档且有 upstream
        if let Some(ref upstream) = doc.upstream
            && upstream != &doc.id
        {
            depends_on.push(upstream.clone());
        }

        // 2/3 文档没有 upstream 视为阻塞
        if doc.stage.as_str() == "2/3" && doc.upstream.is_none() {
            blockers.push("stage 2/3 but no upstream defined: governance chain broken".to_string());
            blocked += 1;
        }

        let role = nature_to_role(&doc.nature);
        let stage_desc = stage_to_description(doc.stage.as_str(), &doc.nature);
        let status_note = match doc.status {
            DocStatus::Ok => String::new(),
            DocStatus::Warning => "（有格式建议，不影响功能）".to_string(),
            DocStatus::Error => "（有 validation 致命错误需修复）".to_string(),
        };

        let summary = format!(
            "{}：{} | stage {}{}",
            role, stage_desc, doc.stage, status_note
        );

        let card = KanbanCard {
            id: doc.id.clone(),
            title: doc.title.clone(),
            entity_type: "document".to_string(),
            stage: doc.stage.to_display(),
            status: doc.status.as_str().to_string(),
            nature: Some(doc.nature.clone()),
            summary,
            blockers,
            depends_on,
        };

        if doc.stage.as_str() == "3/3" {
            done_count += 1;
        }

        // 放入对应列
        if let Some(col) = columns.iter_mut().find(|c| c.phase == phase) {
            col.cards.push(card);
            col.card_count = col.cards.len();
        }
    }

    // 添加代码任务卡片（从 Roadmap §8 手动维护，带 [CODE] 标记的数据来源）
    let code_tasks = get_code_tasks();
    for task in &code_tasks {
        total += 1;
        if !task.blockers.is_empty() {
            blocked += 1;
        }
        if task.stage == "done" {
            done_count += 1;
        }
        if let Some(col) = columns.iter_mut().find(|c| c.phase == task.phase()) {
            col.cards.push(task.clone());
            col.card_count = col.cards.len();
        }
    }

    // 对 verify 列按 nature 分组：Ok 项折叠计数，Warning/Error 展开详情
    if let Some(verify_col) = columns.iter_mut().find(|c| c.phase == "verify") {
        let mut groups: Vec<VerifyGroup> = Vec::new();
        let nature_order = ["spec", "proposal", "decision", "reference", "note", "session_summary"];
        let nature_labels = ["技术规约", "变更提案", "架构决策", "参照文档", "实践笔记", "会话摘要"];

        for (i, nat) in nature_order.iter().enumerate() {
            let nature_cards: Vec<&KanbanCard> = verify_col
                .cards
                .iter()
                .filter(|c| c.nature.as_deref() == Some(nat))
                .collect();

            if nature_cards.is_empty() {
                continue;
            }

            let attention: Vec<KanbanCard> = nature_cards
                .iter()
                .filter(|c| c.status != "Ok")
                .map(|c| (*c).clone())
                .collect();

            let clean_count = nature_cards.len().saturating_sub(attention.len());

            groups.push(VerifyGroup {
                nature: format!("{}（{}）", nature_labels[i], nat),
                clean_count,
                attention,
            });
        }

        verify_col.groups = Some(groups);
    }

    // 道四盲区
    limitations.push("代码任务卡片来源于 Roadmap §8 手动维护，并非从 src/ 自动推导。".to_string());
    limitations
        .push("文档卡片的 blockers 仅基于 validation status 和 upstream 缺失检测，".to_string());
    limitations.push("看板不反映跨文档语义冲突（需 Mind 阶段的 iCL 关系图谱）。".to_string());

    let active = total.saturating_sub(done_count);

    let by_phase: Vec<(String, usize)> = columns
        .iter()
        .map(|c| (c.phase.clone(), c.card_count))
        .collect();

    let summary = KanbanSummary {
        total_cards: total,
        by_phase,
        active_cards: active,
        blocked_cards: blocked,
        done_cards: done_count,
    };

    let generated_at = chrono::Utc::now().to_rfc3339();

    Kanban {
        generated_at,
        columns,
        limitations,
        summary,
    }
}

/// 代码任务：从 Roadmap §8 提取的待办项（手动维护）
fn get_code_tasks() -> Vec<KanbanCard> {
    vec![
        KanbanCard {
            id: "ci-cd-enhance".to_string(),
            title: "CI/CD 增强".to_string(),
            entity_type: "code_task".to_string(),
            stage: "in_progress".to_string(),
            status: "in_progress".to_string(),
            nature: None,
            summary: "当前 CI 只有 cargo test + clippy + fmt，缺乏 release build 验证。完成后可确保每次 push 不破坏编译，解除 crates.io 发布阻塞。".to_string(),
            blockers: Vec::new(),
            depends_on: Vec::new(),
        },
        KanbanCard {
            id: "semantic-yml-fill".to_string(),
            title: "semantic.yml：规约↔代码 fidelity 映射".to_string(),
            entity_type: "code_task".to_string(),
            stage: "pending".to_string(),
            status: "pending".to_string(),
            nature: None,
            summary: ".sih/semantic.yml 是代码到规约的可追溯映射表（703 bytes 占位）。填充后，engine 可以自动验证 '代码是否忠实实现了规约'。当前可手动建立基础映射，Mind 成熟后自动化。".to_string(),
            blockers: vec!["待 Mind 成熟后自动化，当前可手动建立基础映射".to_string()],
            depends_on: Vec::new(),
        },
        KanbanCard {
            id: "crates-io-publish".to_string(),
            title: "外部发布：crates.io + 文档站".to_string(),
            entity_type: "code_task".to_string(),
            stage: "pending".to_string(),
            status: "pending".to_string(),
            nature: None,
            summary: "将 sihankor crate 发布到 crates.io，使外部项目可通过 Cargo 依赖司衡引擎。需 CI/CD 增强完成后才能安全发布（确保 release build 不破损）。".to_string(),
            blockers: vec!["CI/CD 增强完成后才能安全发布".to_string()],
            depends_on: vec!["ci-cd-enhance".to_string()],
        },
        KanbanCard {
            id: "code-lint-impl".to_string(),
            title: "code-lint：Rust 代码质量约束引擎化".to_string(),
            entity_type: "code_task".to_string(),
            stage: "pending".to_string(),
            status: "pending".to_string(),
            nature: None,
            summary: "将《司衡鉴论》的代码审查标准实现为自动化 lint 工具。对应 proposal 260616-2100（stage 2/3），需先完成 decision 定稿（3/3）后才能开工。".to_string(),
            blockers: vec!["proposal 260616-2100 需从 2/3 推进至 decision 3/3".to_string()],
            depends_on: vec!["260616-2100-code-lint-proposal".to_string()],
        },
    ]
}

/// 生成自包含 HTML 可视化看板
pub fn render_html(kanban: &Kanban) -> String {
    let data_json = serde_json::to_string(kanban).unwrap_or_else(|_| "{}".to_string());
    format!(
        r#"<!DOCTYPE html>
<html lang="zh">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>司衡工程看板</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:#f0f2f5;color:#1a1a2e;min-height:100vh}}
.header{{background:linear-gradient(135deg,#1a1a2e,#16213e);color:#e8e8e8;padding:16px 24px;display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:8px}}
.header h1{{font-size:1.3em;font-weight:600}}
.summary{{display:flex;gap:16px;font-size:0.85em;flex-wrap:wrap}}
.summary span{{background:rgba(255,255,255,0.1);padding:4px 12px;border-radius:12px}}
.summary .active{{background:#ffd166;color:#1a1a2e}}
.summary .blocked{{background:#ef476f;color:#fff}}
.summary .done{{background:#06d6a0;color:#1a1a2e}}
.board{{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px;padding:16px;align-items:start}}
.col{{background:#fff;border-radius:8px;box-shadow:0 1px 4px rgba(0,0,0,0.08);overflow:hidden}}
.col-header{{padding:10px 14px;font-weight:600;font-size:0.9em;display:flex;justify-content:space-between;align-items:center}}
.col-header .count{{background:#e8e8e8;padding:2px 8px;border-radius:10px;font-size:0.8em}}
.col-body{{padding:8px;max-height:70vh;overflow-y:auto}}
.card{{margin:6px;padding:10px 12px;border-radius:6px;font-size:0.82em;border-left:4px solid #ccc;background:#fafafa;transition:box-shadow 0.15s}}
.card:hover{{box-shadow:0 2px 8px rgba(0,0,0,0.12)}}
.card .title{{font-weight:600;margin-bottom:2px;word-break:break-word}}
.card .meta{{display:flex;gap:6px;flex-wrap:wrap;margin-top:4px;font-size:0.8em;color:#666}}
.card .tag{{padding:1px 6px;border-radius:8px;font-size:0.78em;white-space:nowrap}}
.tag-Ok{{background:#d4edda;color:#155724}}
.tag-Warning{{background:#fff3cd;color:#856404}}
.tag-Error,.tag-blocked{{background:#f8d7da;color:#721c24}}
.tag-pending{{background:#e2e3f1;color:#3a3d6b}}
.tag-in_progress{{background:#cce5ff;color:#004085}}
.tag-done{{background:#d4edda;color:#155724}}
.card.document{{border-left-color:#118ab2}}
.card.code_task{{border-left-color:#073b4c}}
.card.blocked{{border-left-color:#ef476f;background:#fff5f5}}
.blocker-list{{margin-top:4px;font-size:0.78em;color:#ef476f}}
.blocker-list li{{margin-left:14px}}
.depends{{font-size:0.76em;color:#888;margin-top:2px}}
.col-phase-draft .col-header{{background:#f8f9fa;color:#6c757d}}
.col-phase-propose .col-header{{background:#fff3cd;color:#856404}}
.col-phase-decide .col-header{{background:#cce5ff;color:#004085}}
.col-phase-spec .col-header{{background:#e2e3f1;color:#3a3d6b}}
.col-phase-code .col-header{{background:#d4edda;color:#155724}}
.col-phase-verify .col-header{{background:#d1ecf1;color:#0c5460}}
.limitations{{padding:12px 24px;font-size:0.78em;color:#888;border-top:1px solid #e8e8e8;margin-top:16px}}
.limitations p{{margin:2px 0}}
.footer{{padding:8px 24px 16px;font-size:0.75em;color:#aaa;text-align:right}}
</style>
</head>
<body>
<div class="header">
  <h1>司衡工程看板</h1>
  <div class="summary">
    <span>总计 {total}</span>
    <span class="active">活跃 {active}</span>
    <span class="blocked">阻塞 {blocked}</span>
    <span class="done">完成 {done}</span>
  </div>
</div>
<div class="board" id="board"></div>
<div class="limitations" id="limitations"></div>
<div class="footer" id="footer"></div>
<script>
const DATA = {data_json};

function statusTag(s) {{
  const cls = s === 'Ok' ? 'Ok' : s === 'Warning' ? 'Warning' : s === 'Error' ? 'Error' : s === 'in_progress' ? 'in_progress' : s === 'pending' ? 'pending' : 'done';
  return '<span class="tag tag-'+cls+'">'+s+'</span>';
}}

function natureBadge(n) {{
  const map = {{spec:'规',proposal:'案',decision:'决',note:'记',reference:'参'}};
  return n ? '<span class="tag" style="background:#e8e8e8">'+ (map[n]||n) +'</span>' : '';
}}

function render() {{
  const board = document.getElementById('board');
  const limitations = document.getElementById('limitations');
  const footer = document.getElementById('footer');

  DATA.columns.forEach(col => {{
    const div = document.createElement('div');
    div.className = 'col col-phase-'+col.phase;
    div.innerHTML = '<div class="col-header"><span>'+col.label+' ('+col.card_count+')</span><span class="count">'+col.entry_condition.split('：')[0]+'</span></div>'+
      '<div class="col-body">'+col.cards.map(card => {{
        let cls = 'card '+card.entity_type;
        if (card.blockers && card.blockers.length) cls += ' blocked';
        let html = '<div class="'+cls+'">';
        html += '<div class="title">'+natureBadge(card.nature)+' '+card.title+'</div>';
        html += '<div class="meta">'+statusTag(card.status);
        if (card.stage) html += '<span class="tag" style="background:#e8e8e8">'+card.stage+'</span>';
        if (card.entity_type === 'code_task') html += '<span class="tag" style="background:#d1ecf1">code</span>';
        if (card.depends_on && card.depends_on.length) html += '<span class="tag" style="background:#f8f9fa">'+card.depends_on.length+' dep</span>';
        html += '</div>';
        if (card.blockers && card.blockers.length) {{
          html += '<ul class="blocker-list">';
          card.blockers.forEach(b => {{ html += '<li>'+b+'</li>'; }});
          html += '</ul>';
        }}
        if (card.depends_on && card.depends_on.length) {{
          html += '<div class="depends">← '+card.depends_on.slice(0,2).join(', ')+(card.depends_on.length>2?' ...':'')+'</div>';
        }}
        html += '</div>';
        return html;
      }}).join('')+'</div>';
    board.appendChild(div);
  }});

  limitations.innerHTML = DATA.limitations.map(l => '<p>⚠ '+l+'</p>').join('');
  footer.innerHTML = '生成时间：'+DATA.generated_at;
}}

render();
</script>
</body>
</html>"#,
        data_json = data_json,
        total = kanban.summary.total_cards,
        active = kanban.summary.active_cards,
        blocked = kanban.summary.blocked_cards,
        done = kanban.summary.done_cards,
    )
}

impl KanbanCard {
    fn phase(&self) -> &'static str {
        match self.stage.as_str() {
            "pending" => "spec",
            "in_progress" => "code",
            "done" => "verify",
            _ => "draft",
        }
    }
}
