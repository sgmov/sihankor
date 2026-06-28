---
id: 260628-2300-session-summary
stage: 1/3
upstream: 260628-2300-project-brief-mcp-tool
---
# 会话摘要提案

## 一、问题

### 1.1 上下文压力

司衡累计 2375 次对话轮次，Cache Read 达 206M tokens。每次新 session 启动时需要大量上下文来恢复状态，这些上下文目前靠人工复述（维护者每次解释项目当前阶段）。

### 1.2 现有机制未覆盖会话终结

司衡已有：
- **行迹（Trail）**：记录意图形成过程中的方向转折点
- **项目简报（sihankor_project_brief）**：聚合 git 状态 + 行迹摘要 + 文档统计

但缺少：**会话终结产物**——每次 session 结束时，agent 做了什么、产出了什么、还有什么没做完。这些信息目前只存在于对话历史中，下一个 session 读全量历史成本极高。

### 1.3 新 session 的冷启动问题

每次新 session 启动，司衡需要：
1. 读取项目当前状态（sihankor_project_brief）
2. 读取历史对话（目前只能读全量或靠人工复述）
3. 理解当前治理阶段和待办

Session summary 填补第 2 项的空白。

## 二、方案

### 2.1 新增文档 nature：session_summary

在 `docs/knowledge/session-summaries/` 目录下存储，命名格式：

```
YYYYMMDD-HHMMSS-summary.sih.md
```

 Frontmatter 字段：

 ```yaml
 # id 格式：YYYYMMDD-HHMMSS-session-summary
 id: 260628-HHMMSS-session-summary
 session_id: <mvs_xxx>
 session_role: root|branch
 duration_minutes: <N>
 goal: <本次会话目标>
 outcome: <产出摘要，或"无产出">
 commits: <commit hash 列表，留空表示无>
 todos_remaining:
   - <待办1> (blocked by: <阻塞原因>)
   - <待办2>
 decisions:
   - <治理决策>: <依据>
 risks: <风险描述，可空>
 ```
```

若本次会话无实际产出，frontmatter 中添加：

```yaml
outcome: 无产出
```

### 2.2 新增 G 级规则：V-G-SS01

**规则名称**：会话终结前必须产出 session_summary

**检查逻辑**：
- 扫描 `docs/knowledge/session-summaries/` 目录
- 获取最近一条 session summary 文件的修改时间
- 若最近一条文件的 mtime 早于当前时间 24 小时以上，且当前 session 有过实际工作（commits 数量 > 0），则触发 V-G-SS01 warn
- 若 `outcome: 无产出`，则不触发

**Severity**：Guideline（warn），不阻断

### 2.3 sihankor_project_brief 自动引用

更新 `sihankor_project_brief` 输出，新增 "Recent Sessions" 段：

```
## Recent Sessions
- [260628-1430] goal: DSR-4 ripgrep review, outcome: ripgrep 22 .md / 0 .sih.md confirmed, commits: 66d1cfe
- [260628-1500] goal: Trail mechanism implementation, outcome: 2 tools + proposal ratified, commits: d7c973a
- [260628-2200] goal: V-G-04 calibration, outcome: 483->70 G04 violations cleared, commits: fc4c61d
```

引用最新 3 条 session summary。

### 2.4 无需新增 MCP 工具

Session summary 通过文件写入实现，不需要独立 MCP 工具。Agent 在会话结束时自行创建文件，sihankor_project_brief 读取已有文件。

## 三、实现计划

### 3.1 代码改动范围

| 文件 | 改动 |
|---|---|
| `src/core/validator.rs` | infer_nature 支持 session-summaries/ 目录；V-G-SS01 规则实现 |
| `src/observe/brief.rs` | 新增 Recent Sessions 段，读取 session-summaries/ 目录 |
| `src/core/kanban.rs` | nature_order 加入 session_summary |
| `src/core/governance_check.rs` | session_summary 跳过 upstream 必填检查 |
| `src/fmt/mod.rs` | spec/decision warning 逻辑扩展至 session_summary |

### 3.2 验收标准

- `cargo build && cargo test --all-targets` 全过
- `cargo run --bin rebuild_index -- --strict` 无新增 F 违规
- `sihankor_project_brief` 输出包含 "Recent Sessions" 段
- 新建 `docs/knowledge/session-summaries/` 目录存在，格式符合模板
