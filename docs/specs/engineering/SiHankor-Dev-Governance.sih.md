---
id: 260616-1200-sihankor-dev-governance
stage: 1/3
upstream: 240610-1030-on-sihankor-canon
---

# 司衡引擎开发治理

> 司衡引擎的开发遵循司衡自身的治理流程。本文是引擎开发流程的权威参照。

## 一、道四声明：引导悖论

本文描述的完整链包含 engine 自动验证环节。engine 尚未实现——当前处于**引导阶段**。

**引导阶段规则**：

- 链的前三步（drafts → proposal → decision → spec）可立即运行，只需人类的纪律和文档约定
- 链的后两步（自动验证 + semantic.yml 填充）待 engine 达到 MVP 基线后启动

**MVP 基线**：engine 能够解析 .sih.md 的 frontmatter（id, stage, upstream），能读取 docs/ 目录结构并做基本校验。

## 二、治理链

```
knowledge/drafts/  →  proposals/  →  decisions/  →  specs/engineering/  →  src/  →  .sih/semantic.yml

  模糊意图             提案           决议             规约修订              代码        映射验证
  brainstorming        1/3→2/3→3/3    ADR              设计更新              Rust        fidelity检测
  (非.sih.md)          stage 3/3     stage 3/3         stage 2/3→3/3                    gap标记
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
- 方案对比（≥2 种，推荐一种，说明理由）
- 对现有规约的影响
- 验收标准（可验证、可量化）

1/3 → 2/3：提案结构化完成，进入审查。
2/3 → 3/3：经过鉴的检验，决议通过。决议者是人类。

### 第三阶段：决策落地

proposal 3/3 后执行两件事：

1. **决策记录**：在 `docs/decisions/` 中建立 ADR，记录选择方案的理由
2. **规约更新**：若提案涉及设计变更，对应 `docs/specs/engineering/` 文档走 Reopen 流程更新

### 第四阶段：代码实现

规约更新后，按 `docs/specs/engineering/` 中的设计文档进行编码。
代码组织遵循 Rust 工程惯例。

### 第五阶段：闭合验证

engine 达到 MVP 基线后执行：

1. 在 `.sih/semantic.yml` 中注册代码↔规约映射
2. engine 对自身 docs/ 执行验证（自指）
3. fidelity 检测结果记录在 semantic.yml 中
4. gap 状态变更

## 三、变更分级

| 级别     | 流程                      | 判定标准                 | 示例                       |
| -------- | ------------------------- | ------------------------ | -------------------------- |
| 勘误     | 直接修改 + commit message | 不改变下游行为预期       | typo、路径、格式修正       |
| 轻量修订 | proposal → decision       | 改变实现细节，不改变契约 | 修改验证规则、增加工具参数 |
| 设计变更 | 完整五阶段链              | 改变接口契约或架构       | 新增模块、改变流转模型     |
| 法层修正 | Canon Reopen              | 改变法层定义             | 修改 stage 语义、增减法条  |

## 四、gap 引用约定

gap 实体尚未正式定义（待独立 proposal）。过渡期间使用占位格式：

```markdown
[GAP: 引擎核心模块未实现 — 阻塞文档解析和 frontmatter 校验]
```

待 gap 实体定义完成后，批量替换为 `@gap: <id>` 正式格式。

## 五、与司衡治理体系的关系

- Canon（法层）：定义了 stage 语义、生命周期规则、目录即身份——引擎开发链遵循全部法层规则
- Document-Conventions（术层）：定义了 frontmatter 格式、事件记录——引擎开发中 proposal 和 decision 文档遵循 Conventions
- Engineering-Mapping（映射）：定义了哲学→工程的映射——引擎每个模块的哲学溯源参照此文档
