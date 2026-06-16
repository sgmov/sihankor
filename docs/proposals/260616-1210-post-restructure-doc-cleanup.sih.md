---
id: 2606161210-post-restructure-doc-cleanup
stage: 3/3
upstream: 2406101030-on-sihankor-canon
---

# 重构后文档残余引用清理

> 本次重构（2606151430-docs-restructure-v2）废除了 type 字段、引入了目录即身份、修正了 stage 语义和 notes 生命周期。三份非 3/3 文档中残留旧引用需同步清理。

## 变更清单

### Engineering-Mapping（specs/engineering/，stage 2/3）

| 变更项 | 依据 |
|--------|------|
| 删除公理体系中的"原则一（力度有别）""原则三（三机分权）"等旧术语 | 公理体系已在 Canon 中重构 |
| `F-08（文档必须在 type 对应目录）` → 删除，type 已废除 | Canon $6.2 目录即身份 |
| `G-04（同格 ratify 文档不超 3 个活跃版本）` → 视是否仍有法层依据保留或删除 | 需确认 Canon 当前是否保留了此规则 |
| `G-06（tags 不参与逻辑）` → 删除，tags 概念已废弃 | Conventions 更新中已移除 |

### Engine-Roadmap（proposals/，stage 1/3）

| 变更项 | 依据 |
|--------|------|
| frontmatter 校验：`id/type/stage 必填` → `id/stage 必填` | type 已废除 |
| structure 校验：`目录位置与 type 匹配（F-08）` → `目录位置合法` | F-08 已废除 |
| governance 校验：`decided-by 签认存在` → 删除 | decided-by 仅限 decisions/ frontmatter |
| note stage: `2/3(自动)` → `2/3(人类确认)` | Canon $6.2 修正 |
| 成熟度矩阵中 reference 层：Compendium `1/3`、Onomastic `2/3` 需更新为实际推进后的值 | 本次重构已推进 |

### Engine-Design-Summary（specs/engineering/，stage 2/3）

| 变更项 | 依据 |
|--------|------|
| 状态机图：`note: 1/3 → 2/3(自动) → 3/3(晋升)` → `note: 1/3 → 2/3(人类确认) → 3/3(晋升)` | Canon $6.2 修正 |

## 影响范围

- 三份文档，变更量 ≤ 10 处
- 不改变下游引用权限（均未到 3/3）
- 不需要 Reopen（均未 ratify）
