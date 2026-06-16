---
id: 260616-1210-post-restructure-doc-cleanup-decision
stage: 3/3
upstream: 260616-1210-post-restructure-doc-cleanup
decided-by: ai-assist
---

# 重构后文档残余引用清理决策

## 背景

本次重构废除了 type 字段、引入了目录即身份、修正了 notes 生命周期。Engineering-Mapping、Engine-Roadmap、Engine-Design-Summary 三份非 3/3 文档中残留旧引用。

## 决策

采纳轻量修订提案 260616-1210-post-restructure-doc-cleanup：

**Engineering-Mapping**：

- 删除 `idea 类型` 引用（第 20、262 行）
- 删除 `tags 不参与逻辑` 引用（第 20、40、56、262 行）
- 删除 F-08（第 38 行）
- 删除 G-06（第 40、56 行）
- 删除"原则一/二/三"行（第 25-27、267-269 行）——公理体系已重构
- G-04 保留——"同格 ratify 文档不超 3 个活跃版本"不依赖 type

**Engine-Roadmap**：

- 成熟度矩阵更新：Compendium 1/3→2/3, Onomastic 2/3→3/3, Arguments 1/3→2/3（已完成推进）
- frontmatter 校验：`id/type/stage` → `id/stage`
- 删除"目录位置与 type 匹配（F-08）"
- 删除 `decided-by 签认存在`

**Engine-Design-Summary**：

- 状态机图 `note: 1/3 → 2/3(自动)` → `note: 1/3 → 2/3(人类确认)`

## 后果

三份文档与本次重构后的规约对齐。均为非 3/3 文档，不需要 Reopen。
