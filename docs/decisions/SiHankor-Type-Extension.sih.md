---
id: 260614-0000-sihankor-type-extension
stage: 0/260615-1500-docs-restructure-v2-decision
upstream: 240610-1500-sihankor-document-conventions
decided-by: ai-assist
---

# 司衡文档类型扩展决策

> 将司衡文档 type 从 4 种扩展为 7 种，补入 plan/decision/proposal 三种类型。

## 背景

司衡引擎代码实现（`src/core/models.rs`）中定义了 7 种 document type：

- treatise
- compendium
- mapping
- note
- plan
- decision
- proposal

但 `Document-Conventions.sih.md $4.3` 文档类型表只列出了 4 种（treatise/compendium/mapping/note），文档还声称"五类不可增删"。这造成了规约与实现之间的间隙（道四），且规约自身内部也不一致。

与此同时，项目实际使用了 5 个目录（specs/proposals/decisions/reference/notes/plan），其中 proposals/ 和 decisions/ 没有对应的 type 定义。

## 决策

将 `Document-Conventions.sih.md $4.3` 文档类型表从 4 种扩展为 7 种，补入以下三种 type：

| type     | 中文对 | 英文对   | 定义                                   |
| -------- | ------ | -------- | -------------------------------------- |
| plan     | 策     | Plan     | 规划文档：目标分解、任务拆解、路线图   |
| decision | 决     | Decision | 架构决策：权衡利弊、选定方案、记录理由 |
| proposal | 议     | Proposal | 方向提案：论证提案、评估影响、请求确认 |

同时将"五类不可增删"修正为"七类不可增删"。

## 后果

- 代码与文档对齐，G-02 type-目录验证规则不再产生误报
- 新增的 3 种 type 需在司衡法论（Canon.sih.md）中有对应法层定义
- 已在 `decisions/` 目录下的文档（External-Validation、Legacy-Migration-Governance）应将 type 从 treatise 修正为 decision

## 签认

| 字段 | 值                                                           |
| ---- | ------------------------------------------------------------ |
| 日期 | 2026-06-14                                                   |
| 依据 | 司衡顺因之法：规范先行于实现，代码暴露的间隙需在规范层面修正 |
