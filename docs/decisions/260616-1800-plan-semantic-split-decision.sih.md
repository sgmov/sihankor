---
id: 260616-1800-plan-semantic-split-decision
stage: 3/3
upstream: 240602-0900-on-sihankor
decided-by: ai-assist
---

# plan 语义拆分决策

## 背景

当前 Canon $6 将 `proposals/` 定义为同时容纳变更提议（「是否应该做 X」）和 plan 型文档（「路线图、任务拆解」）。两类文档的因果意图不同：

- **变更提议**：针对具体变更点，有决议窗口，决议后关闭（3/3 或 0/）
- **路线图**：描述系统整体方向，持续更新无窗口，永无终了

前一类回答「我们是否应该做 X」，后一类回答「系统将如何成为」：：前者是因果未决的提议，后者是因果已决的方向规约。混放于同一目录造成名实不符：proposal 的「未决」语义被路线图的「已决」语义污染。

Type-Extension 决策曾将 `plan` 定义为独立 type（「策：规划文档」），但 type 字段在 restructure-v2 中废除，plan 作为独立类型的分类路径已不存在。

## 哲学推导

**正名之法**（Onomastic-Philosophy）：一名对应一实。「plan」在传统工程中对应三实：：方案论证（RFC）、方向规约（roadmap）、任务排期（sprint）：：是名实不符。正名要求将其拆分为二：

| 实 | SiHankor 正名 | 归处 |
|----|---------------|------|
| 方案论证 | proposal | `proposals/` |
| 方向规约 | spec | `specs/engineering/` |
| 任务排期 | 文档内部内容，不独立成 nature | ： |

**顺因之法**：spec 的因果意图是「定义系统」，路线图的因果意图是「定义系统将如何成为」：：spec 定义 being，路线图定义 becoming。道一视 being 与 becoming 为一体，因此路线图与 spec 同根同 nature。

**有度之法**：是否存在独立的「查 plan 目录」动机？不存在：：这个动机裂解为「接下来做什么」（看方向规约）和「某 feature 计划状态」（看对应 proposal 的 stage），两者已有满足路径。

**道四之法**：路线图的 stage 3/3 不意为「执行完毕」，而意为「此规划方法的合道性已被验证」：：规约（roadmap）与实现（执行现实）间的间隙永存。

## 决策

1. **路线图级 plan 归入 `specs/engineering/`**，nature 为 spec。Engine-Roadmap 从 `proposals/` 移至 `specs/engineering/`
2. **局部任务级 plan（如 MVP parser 方案）留在 `proposals/`**，nature 为 proposal。此类文档本质是变更提议，只是在 plan 传统中被泛称为 plan
3. **Canon $6 `proposals/` 定义修正**：移除「也存放 plan 型文档」，改为「不放大方向路线图（-> `specs/engineering/`）」
4. **README.md 添加术语映射表**：传统工程 `plan/roadmap` -> SiHankor `specs/engineering/` 的引导

## 后果

- Engine-Roadmap 路径：`docs/proposals/SiHankor-Engine-Roadmap.sih.md` -> `docs/specs/engineering/SiHankor-Engine-Roadmap.sih.md`
- 所有指向 Engine-Roadmap 的 upstream/引用链需同步更新
- Canon $6 第 488 行修正
- README.md 第 73-77 行修正（移除 `plan/` 目录 + 添加术语映射）
- nature 仍为 5 类：spec/proposal/decision/reference/note，不新增
