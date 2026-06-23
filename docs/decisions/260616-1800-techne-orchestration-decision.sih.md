---
id: 260616-1800-techne-orchestration-decision
stage: 2/3
upstream: 260616-1700-techne-orchestration-proposal
decided-by: sihankor
---
# Techne 术层编排工具：设计决策

## 背景

[《Techne 术层编排工具提议》](../proposals/260616-1700-techne-orchestration-proposal.sih.md) 提出将术层编排工具集成到 SiHankor MCP Server，补全道→法→术→鉴四阶段中的执行空白。提议经 iCT 五法检验全 Pass，零发散，晋升至 2/3。

## 方案选择

| 维度       | 决策                                        | 法依据                          |
| ---------- | ------------------------------------------- | ------------------------------- |
| 术的形式   | MCP 工具，挂载在 `SihankorService` 上       | 顺因：道法鉴已在 MCP，术应同级  |
| 命名规则   | `{verb}_{noun}_plan`（无 i- 前缀）          | 知止：术不在几层，不占 i- 前缀  |
| 规格文档   | `docs/specs/techne/`，与 `philosophy/` 平行 | 有度：器域与论域分离            |
| 首批工具   | `generate_document_plan`                    | 损补：文档生成是当前最大空白    |
| 命名英文对 | Techne（希腊语 tekhne：技艺、实践智慧）     | Onomastic-Philosophy $三 ratify |

## 术层工具清单

| 工具名                    | 职责                             | 优先级 |
| ------------------------- | -------------------------------- | ------ |
| `generate_document_plan`  | 编排 iCL + iWW，产出文档生成蓝图 | 首批   |
| `review_progression_plan` | 编排 stage 推进步骤              | 次批   |
| `governance_chain_plan`   | 编排上下游依赖操作顺序           | 次批   |
| `code_audit_plan`         | 编排代码与治理文档一致性检查     | 次批   |

## ADR

### decided-by

ai辅助：iCT 五法检验全 Pass，人类确认方案自洽后晋升。

### DEPS

- [Techne 提议](../proposals/260616-1700-techne-orchestration-proposal.sih.md)
- [命名哲思](../reference/SiHankor-Onomastic-Philosophy.sih.md)
