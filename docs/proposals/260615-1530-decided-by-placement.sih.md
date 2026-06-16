---
id: 2606151530-decided-by-placement
stage: 3/3
upstream: 2406101030-on-sihankor-canon
---

# decided-by 字段定位修正

## 背景

当前体系中 `decided-by` 的使用存在三个不一致：

1. **位置不一致**：On-SiHankor 和 Arche 在 frontmatter 中声明 `decided-by: ai-assist`，但按 Canon 和 Conventions 的既有规则，`decided-by` 是 ADR 签认字段——treatise 型文档（specs/ 下）不应在 frontmatter 中携带此字段
2. **语义混淆**：文档经历多个 stage 变更时，每个变更可能由不同人决定。frontmatter 只能存一个值——存最新的丢失历史，存列表则 frontmatter 膨胀
3. **decisions/ 的例外未显式化**：decision 文档的写作意图就是记录决策，`decided-by` 是其内容的一部分，应在 frontmatter 中保留。这一例外在 Conventions 中隐含（"独立 ADR 文档（decisions/）：frontmatter 字段"）但其余文档的排斥规则未说明

## 提案

### 一、decided-by 的正确位置

| 场景                   | 位置                                      | 力度             |
| ---------------------- | ----------------------------------------- | ---------------- |
| decisions/ 下的文档    | frontmatter                               | F 级（缺则非法） |
| stage 变更（人为触发） | `.sih/events/{doc-id}.yml` 中对应事件记录 | F 级             |
| Reopen 声明            | Reopen 块内                               | F 级             |
| 文档创建               | `.sih/events/` 中建议记录                 | G 级             |

### 二、frontmatter 中不应有 decided-by（decisions/ 除外）

`decided-by` 回答"谁决定了这个状态变更"——属于状态变迁事件，不属于文档状态本身。specs/、proposals/、reference/、knowledge/notes/ 下的文档，frontmatter 不声明 `decided-by`。

### 三、执行清理

| 文档                             | 动作                                          |
| -------------------------------- | --------------------------------------------- |
| On-SiHankor.sih.md               | 移除 frontmatter 中的 `decided-by: ai-assist` |
| Arche-The-One-Above-Being.sih.md | 移除 frontmatter 中的 `decided-by: ai-assist` |

### 四、规约同步

- Document-Conventions $4.4 补充：frontmatter 中 `decided-by` 仅限 decisions/ 目录下的文档
- Document-Conventions $4.5 事件记录：stage-change 事件增加 `decided-by` 必填字段

## 影响范围

- 2 份文档 frontmatter 清理
- 1 份术层文档（Document-Conventions）补充规则
- 法层（Canon）不变——Canon 定义的是"人为触发的 stage 变更必须有 ADR"，`decided-by` 的位置是术层约定
