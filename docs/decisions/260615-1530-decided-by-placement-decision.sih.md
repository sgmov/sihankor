---
id: 260615-1530-decided-by-placement-decision
stage: 3/3
upstream: 260615-1530-decided-by-placement
decided-by: ai-assist
---

# decided-by 字段定位修正决策

## 背景

`decided-by` 是状态变迁属性，不属于文档状态本身。On-SiHankor 和 Arche 在 frontmatter 中残留 `decided-by: ai-assist`，违反此原则。需在规约中显式化 `decided-by` 的正确位置。

## 决策

采纳提案 260615-1530-decided-by-placement。

1. decisions/ 下的文档：`decided-by` 在 frontmatter 中，F 级
2. stage 变更（人为触发）：`decided-by` 在 `.sih/events/` 对应事件记录中，F 级
3. Reopen 声明：`decided-by` 在 Reopen 块内，F 级
4. specs/、proposals/、reference/、knowledge/notes/ 下的文档：frontmatter 不声明 `decided-by`

## 后果

- On-SiHankor 和 Arche frontmatter 清理 `decided-by` 行
- Document-Conventions $4.4 补充排除规则
- Document-Conventions $4.5 事件记录 schema 增加 `decided-by` 字段
- Canon 不变：：法层已覆盖"人为触发 stage 变更需 ADR"，此决策是术层执行细节
