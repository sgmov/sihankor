---
id: 260616-1200-sihankor-dev-governance
stage: 3/3
upstream: 240610-1030-on-sihankor-canon
---

# 司衡引擎开发治理

> 司衡引擎的开发遵循司衡自身的治理流程。本文是引擎开发流程的权威参照。

## 一、道四声明：引导悖论（更新于 2026-06-16）

本文描述的完整链包含 engine 自动验证环节。engine **第一阶段已完成**（parser + validator(13规则) + indexer + 6 MCP 工具 + SQLite，~1800 LOC），MVP 基线已超越。

**当前状态**：

- 链的前三步（drafts -> proposal -> decision -> spec）已运行多轮，产出 6 个 3/3 decision 和 9 个 proposal
- 链的后两步（自动验证 + semantic.yml 填充）已可启用：：engine 可对自己的 docs/ 执行 `validate_sihmd`
- semantic.yml 填充待 Mind Phase (iCL) 实现后自动化

**MVP 基线**（已达成）：engine 能够解析 .sih.md 的 frontmatter（id, stage, upstream），能读取 docs/ 目录结构并做基本校验。

## 二、治理链

```text
knowledge/drafts/  →  proposals/  →  decisions/  →  specs/engineering/  →  src/  →  .sih/semantic.yml

  模糊意图             提案           决议             规约修订              代码        映射验证
  brainstorming        1/3→2/3→3/3    ADR              设计更新              Rust        fidelity检测
  (非.sih.md)          stage 3/3     stage 3/3         stage 1/3→2/3→3/3                 gap标记
```

### 第一阶段：意图孵化

**起点**：任何关于引擎的模糊意图。

**加工方式**：grill-me 追问。追问源于鉴层的反推九段式，核心问题：

1. 这个功能解决哪个道层问题？
2. 如果不做，哪个 gap 会持续存在？
3. 最小可行方案是什么？（有度）
4. 有没有现成工具可以直接用？（知止）
5. 实现后如何验证它确实缩小了 gap？（可证伪）

**转化门槛**：满足以下三条后，从 `knowledge/drafts/` 转化为 `proposals/`：

1. 能说清要解决的 gap
2. 能列出至少 2 种方案并指出关键差异
3. 能定义 1 条可验证的验收标准

### 第二阶段：提案流转

proposal 放入 `docs/proposals/`，stage 1/3。

proposal 标准结构：

- 要解决的 gap（引用相关文档）
- 方案对比（>=2 种，推荐一种，说明理由）
- 对现有规约的影响
- 验收标准（可验证、可量化）

1/3 -> 2/3：提案结构化完成，进入审查。
2/3 -> 3/3：经过鉴的检验，决议通过。**决议者必须是与提案作者不同的人**；在单人项目中，AI 可充任审查者（识别 gap、对比方案），但不充任决议者：：最终采用/否决的决策仍由人做出。

proposal 3/3 后，对应 decision 文档的 frontmatter 须声明 `decided-by`，记录决策者身份。decision 文档 2/3+ stage 须含 `decided-by`（参见 G-09）；非 decisions/ 目录下的文档不得含 `decided-by`（参见 F-07）。

### 第三阶段：决策落地

proposal 3/3 后执行两件事：

1. **决策记录**：在 `docs/decisions/` 中建立 ADR，记录选择方案的理由，声明 `decided-by`
2. **规约更新**：若提案涉及设计变更，对应 `docs/specs/engineering/` 文档走 Reopen 流程更新（Canon $3.4 定义：触发判据->退回 2/3->ADR 修正->3/3）。变更分级见 $三

### 第四阶段：代码实现

规约更新后，按 `docs/specs/engineering/` 中的设计文档进行编码。
代码组织遵循 Rust 工程惯例。

### 第五阶段：闭合验证

在 `.sih/semantic.yml` 中注册代码<->规约映射。engine 已可执行自指验证（`validate_sihmd` 对自身 docs/ 运行），fidelity 检测结果记录在 semantic.yml 中。

待 Mind Phase (iCL) 实现后，gap 状态变更自动化。

## 三、变更分级

| 级别 | 流程与判定 | 示例 |
|------|-----------|------|
| **勘误** | 直接修改 + commit message。判定：不改变下游行为预期 | typo、路径、事实同步 |
| **轻量修订** | proposal -> decision。判定：改变实现细节，不改变契约 | 修改验证规则、增加工具参数 |
| **设计变更** | 完整五阶段链。判定：改变接口契约或架构 | 新增模块、改变流转模型 |
| **法层修正** | Canon Reopen。判定：改变法层定义 | 修改 stage 语义、增减法条 |

## 四、gap 引用约定

gap 实体尚未正式定义（待独立 proposal）。过渡期间使用占位格式：

```markdown
[GAP: 此处简述缺口 — 阻塞了什么]
```

实际示例（已关闭）：

```markdown
[GAP: 引擎核心模块未实现 — 阻塞文档解析和 frontmatter 校验]  —— 已于 2026-06-16 关闭
```

待 gap 实体定义完成后，批量替换为 `@gap: <id>` 正式格式。

## 五、与司衡治理体系的关系

| 文档 | 层 | 与本文的关系 |
|------|-----|-------------|
| Canon | 法 | 定义 stage 语义、生命周期规则、目录即身份：：引擎开发链遵循全部法层规则 |
| Document-Conventions | 术 | 定义 frontmatter 格式、文档风格约束：：proposal 和 decision 文档遵循 Conventions |
| Engineering-Mapping | 术 | 定义哲学->工程的映射：：引擎每个模块的哲学溯源参照此文档 |
| Engine-Design-Summary | 术 | 定义引擎架构设计：：代码实现以此为准 |
| Mind-Design | 术 | 定义三机流转和 MCP 工具：：第二阶段实现参照 |
| Engine-Roadmap | 术 | 定义引擎工程路线图：：本文开发链的宏观方向参照 |

## 六、decided-by 治理约束

治理链中 `decided-by` 的规则由 validator 的 F-07 和 G-09 执行：

- **F-07（戒）**：非 `decisions/` 目录下的文档，frontmatter 不得含 `decided-by`。违反 -> Fatal。
- **G-09（规）**：`decisions/` 下 stage 为 2/3 或 3/3 的文档，frontmatter 应含 `decided-by`。缺失 -> Guideline。
- **决议者约束**（本文 $二.2）：proposal->decision 的决议者须为作者之外的人。此约束目前无法被 engine 机械验证，标记为 `[human-review]`。
