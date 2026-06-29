---
id: 260629-0120-session-summary-trail-format
stage: 3/3
upstream: 260628-2316-session-summary
---

## 一、内容

### 问题

当前 `session_summary` 和 `trail` 两类文档使用 `.sih.md` 后缀，被索引器发现后进入完整治理校验流水线。但它们不参与治理链条：

- 有自定义 frontmatter 字段（`session_id`、`session_role`、`duration_minutes` 等），不走 `upstream`/`decided-by` 标准字段
- 每次 `rebuild_index` 产生 V-G-02 警告（"不在合法目录"），纯噪音
- 校验器（V-F-04）和 governance_check 都专门为它们写豁免逻辑——修一处忘一处

### 方案

| 文档类型 | 当前格式 | 目标格式 | 理由 |
|---|---|---|---|
| session_summary | `.sih.md` (YAML frontmatter + Markdown 正文) | `.yml` 纯 YAML | 数据已经高度结构化，正文是 frontmatter 的自然语言重复，无信息损失 |
| trail | `.sih.md` (YAML frontmatter + Markdown 正文) | `.md` 纯 Markdown | 本质是叙事决策日志，自然语言最合适，frontmatter 只有薄薄三字段 |

### 迁移内容

**session_summary → .yml（6 个文件）**

frontmatter 字段直接映射为 YAML 顶层 key，正文删除（信息已在前端 matter 中）。示例：

```yaml
# 260628-2316-session-summary.yml
id: 260628-2316-session-summary
session_role: root
duration_minutes: 80
goal: 调研 Cache Read 26M token 来源 + 实现 session summary 机制
outcome: 全部完成（3 个 commit）
commits: [14760b6, fc4c61d, c6a98a1]
todos_remaining:
  - "Ratify DSR-4 note ..."
decisions:
  - "V-G-04 阈值从 >3 调到 >4 ..."
risks:
  - "181 turns 历史仍在 cache 中 ..."
data_stats:
  cache_read_total_session: 18881570
  cost_session: 1.52
```

**trail → .md（6 个文件）**

去掉 YAML frontmatter，正文保留。原 frontmatter 中的 `id` 作为标题，`stage` 和 `upstream` 丢弃（trail 不参与治理链）。

### 代码改动

1. **索引器** `src/core/indexer.rs:discover_documents`：只匹配 `.sih.md`，.yml 和 .md 自然跳过（无需改动，确认当前行为即可）
2. **MCP 工具** `src/mcp_server/governance.rs`：
   - `record_trail`：读写路径从 `.sih.md` 改为 `.md`
   - `sihankor_trail_context`：`collect_trails` 路径适配
3. **observe 模块** `src/observe/brief.rs`：
   - `collect_latest_trails`：过滤 `.md` 而非 `.sih.md`
   - `collect_recent_sessions`：读 `.yml` 而非 `.sih.md`
4. **V-G-02 合法目录** `src/core/validator.rs`：可移除 `knowledge/notes/` 之外的豁免逻辑（session_summary 和 trail 不再走校验）
5. **清理**：`infer_nature` 中的 `trail` 和 `session_summary` 分支、V-F-04 豁免、governance_check 豁免可逐步移除

### 索引库影响

`.sih/index.db` 中已有的 session_summary 和 trail 记录会成为孤儿（源文件改名后索引器不再发现）。rebuild_index 会自然清理（已有 stale-prune pass）。

## 不在范围内

1. spec / proposal / decision / note / reference 格式不变
2. 不改变 rebuild_index 的校验规则
3. 不改变 governance_check 的链完整性逻辑
4. 不涉及 kanban / 前端展示
5. 不改变 `sihankor_session_summary` 等 session 工具的输出——读取路径适配是透明的

## @limitations

1. **trail 的纯 .md 缺乏结构化查询能力**：与 .sih.md 相比，失去 `nature`/`stage` 索引。当前 trails 仅有 6 条手动记录，影响可忽略；如果未来 trail 规模化，可能需要在 `collect_trails` 中解析 H2 标题做轻量索引
2. **session_summary .yml 无正文的 AI 语义检索降级**：正文中的自然语言对语义搜索有帮助。YAML 字段中的 `goal`/`outcome`/`decisions` 字段保留了关键语义，短期够用
3. **向后兼容**：MCP server 重启后生效，`cargo build` 即可，无数据迁移脚本

## DEPS

- `260628-2316-session-summary`：首次引入 session_summary nature，记录 governance_check 跳过 upstream 的决策
- `260628-1500-trail-mechanism`：trail 机制规约，定义行迹记录的格式和用法
