---
id: 260616-1700-techne-orchestration-proposal
stage: 2/3
upstream: 260613-1728-sihankor-onomastic-philosophy
---
# Techne 术层编排工具：MCP 集成提议

## 一、正名：Techne 是什么

Techne（术）是司衡四阶段中的执行层：：道(Tao)观之，法(Canon)定之，**术(Techne)行之**，鉴(Assay)照之。

术不做推理，不做决策，不做验证。术把道(iCL)的认知结果和法(iWW)的决策方向串联为可执行的编排流程，产出结构化蓝图，由 Agent + LLM 完成实际内容生成，最后由鉴(iCT)检验。

术的形式是 MCP 工具——和已有的 `analyze_document`（道）、`propose_decision`（法）、`full_analysis`（鉴）同级，挂在 SiHankor MCP Server 上。术不是机（无 i- 前缀），是编排器。

## 二、顺因：术在治理链中的定位

道法鉴三机已在 MCP 中完备，术是从法到鉴之间缺失的执行环节。

```mermaid
flowchart LR
    Tao["道 Tao：iCL 认知"] --> Canon["法 Canon：iWW 决策"]
    Canon --> Techne["术 Techne：编排执行"]
    Techne --> Assay["鉴 Assay：iCT 检验"]
```

术填补了「法做出决策后、鉴检验之前」的执行空白。因果链完整：道认知上下文 → 法决定策略 → **术编排执行** → 鉴验证产出。

## 三、有度：范围

### 纳入

- 文档生成编排（`generate_document_plan`）：输入目标文档类型和上下文，输出 GenerationPlan 蓝图
- 文档审阅编排（`review_progression_plan`）：输入待审文档，输出 stage 推进步骤
- 治理链编排（`governance_chain_plan`）：输入目标文档，输出上下游依赖操作顺序
- 代码治约一致性编排（`code_audit_plan`）：对齐代码实现与治理文档

### 不纳入

- 实际文档内容生成（由 Agent + LLM 完成）
- 格式校验（已有 `validate_sihmd`）
- 三机推理逻辑（iCL/iWW/iCT 独立运作，术只编排调用顺序）
- Agent 行为控制（AGENTS.md 等配置文件不在治理域）

## 四、知止：术器分离

术有两个实体，不可混淆：

| 实体         | 位置                           | 性质                  |
| ------------ | ------------------------------ | --------------------- |
| 术层编排工具 | `src/mcp_server/governance.rs` | MCP 工具（Rust 代码） |
| 术层工具规格 | `docs/specs/techne/`           | .sih.md 规格文档      |

编排逻辑在代码里，规格文档描述接口和行为。术不做哲学推导（那是论域的事），只声明每个工具的输入、输出、调用链。

`docs/specs/techne/` 和 `docs/specs/philosophy/` 平行但分离：philosophy 是论域（道论、法论、鉴论），techne 是器域（工具规格）。

## 五、损补

当前体系中的空白：

- `full_analysis` 能分析已有文档，但不能指导新文档的生成
- Agent 生成文档时缺少结构化蓝图，只能凭经验猜测 frontmatter 字段和引用关系
- 治理链推进（stage 1/3 → 2/3 → 3/3）没有编排工具

术补全这些空白。已有工具不受影响。

## 六、顺势：实施计划

### 第一阶段：基础设施

1. 在 `src/mcp_server/governance.rs` 中新增术层工具方法
2. 创建 `docs/specs/techne/Techne.sih.md`（索引入口）
3. 实现 `generate_document_plan` 作为首个术层工具

### 第二阶段：扩展

1. 实现 `review_progression_plan`
2. 实现 `governance_chain_plan`
3. 实现 `code_audit_plan`
4. 补全各工具对应的规格文档

### 术层工具命名规则

`mcp__sihankor__{verb}_{noun}_plan`

不遵循 i- 前缀规则（术不在几层），使用描述性动名结构表达编排意图。

## DEPS

- 260613-1728-sihankor-onomastic-philosophy
  - 命名哲思：六层脉络中术 = Techne 的定义，层前缀规则

## SEE-ALSO

- 240610-1030-on-sihankor-canon
  - 法论：F/G/J 规则体系
- 260613-1650-sihankor-mind-design
  - Mind 设计规范：iCL/iWW/iCT 三机接口
