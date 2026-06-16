---
id: 260616-1500-engine-dev-governance-chain-decision
stage: 3/3
upstream: 260616-1500-engine-dev-governance-chain
decided-by: ai-assist
---

# 引擎开发治理链采纳决策

## 背景

引擎开发需要一条从模糊意图到代码实现的治理链。当前开发流程未被规约化——知识在对话中，不在 docs/ 里。引擎自身也应遵循司衡的治理流程（道四：引擎也不例外于道）。

## 决策

采纳提案 260616-1500-engine-dev-governance-chain。引擎开发遵循五阶段链：

1. 意图孵化（knowledge/drafts/ → proposals/）——grill-me 追问 + 三条转化门槛
2. 提案流转（proposals/ 1/3→2/3→3/3）——方案对比 + 验收标准
3. 决策落地（decisions/ + specs/ 更新）——ADR 记录理由 + 规约修订
4. 代码实现（src/）——按设计文档实现
5. 闭合验证（semantic.yml + 自指验证）——fidelity 检测 + gap 状态变更

引导阶段声明：engine 达到 MVP 基线（frontmatter 解析能力）之前，链的后两步暂不执行，前三步由人自律运行。

变更分级：勘误（直接修改）→ 轻量修订（短路径）→ 设计变更（完整链）→ 法层修正（Canon Reopen）。

产出 `docs/specs/engineering/sihankor-dev-governance.sih.md` 作为引擎开发流程的权威参照。

## 后果

- 后续所有引擎开发以此为流程参照
- gap 实体定义需并行推进（proposal 中的 gap 引用使用占位格式 `[GAP: 简述]`）
- 追问模板和并发冲突处理在后续 proposal 中细化
