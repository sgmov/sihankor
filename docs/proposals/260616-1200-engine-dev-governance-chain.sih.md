---
id: 2606161200-engine-dev-governance-chain
stage: 3/3
upstream:
  - 2606110000-sihankor-engine-design-summary
  - 2406101030-on-sihankor-canon
---

# 司衡引擎开发治理链

> 将司衡自身的治理流程应用于司衡引擎开发。本文描述了从模糊意图到代码实现再到自指验证的完整链路。

## 一、道四声明：引导悖论

本提案描述的完整链包含 engine 自动验证环节（语义映射、gap 检测、自指合规校验）。engine 尚未实现这些能力。

这不是链的设计缺陷，是道四的引导悖论：规约（本提案）与实现（engine）之间存在初始间隙。正确的做法不是推迟链的定义直到 engine 完成，而是**显式声明引导阶段**：

**引导阶段**（engine 达到 MVP 基线之前）：

- 链的前三步（drafts → proposal → decision）可立即运行——它们只需要人类的纪律和文档约定
- 链的第四步（代码实现）由人自律执行
- 链的第五步（自动验证 + semantic.yml 填充）暂不执行，待 engine 具备 parser 能力后启动

**MVP 基线**：engine 能够解析 .sih.md 的 frontmatter（id, stage, upstream），能读取 docs/ 目录结构并做基本校验。达到此基线后，链的后两步开始自转。

## 二、治理链总览

```text
knowledge/drafts/     proposals/          decisions/         specs/engineering/    src/             .sih/semantic.yml

模糊意图 ──→ brainstorming ──→ 提案 ──→ 决议 ──→ 规约修订 ──→ 代码实现 ──→ 映射验证
             追问+方案对比       流程流转   ADR记录理由    设计文档更新        Rust实现        fidelity检测
             (非.sih.md)        1/3→2/3→3/3  stage 3/3      stage 2/3→3/3                       gap标记
```

### 第一阶段：意图孵化（knowledge/drafts/ → proposals/）

起点：任何关于引擎的模糊意图。

加工：grill-me 追问。追问方向源于鉴层的反推九段式——不是"这个想法对不对"，而是：

1. 这个功能解决哪个道层问题？（顺因：上游溯源）
2. 如果不做，哪个 gap 会持续存在？（必要性）
3. 做它的最小可行方案是什么？（有度）
4. 有没有现成的 crate 或工具可以直接用？（知止）
5. 实现后如何验证它确实缩小了 gap？（可证伪）

转化门槛。drafts 中的 brainstorming 满足以下三条后转化为 proposal：

1. 能说清要解决的 gap（引用相关文档中的缺口描述）
2. 能列出至少 2 种方案并指出关键差异
3. 能定义 1 条可验证的验收标准

### 第二阶段：提案流转（proposals/ 内）

proposal 结构：

- 要解决的 gap
- 方案对比（≥2 种，推荐一种并说明理由）
- 对现有规约的影响（是否导致 spec/ 中某文档需修改）
- 验收标准（可验证、可量化）

1/3→2/3：提案结构化完成，进入审查
2/3→3/3：经过鉴的检验，决议通过。决议者是人类

### 第三阶段：决策落地（decisions/ + specs/ 更新）

proposal 3/3 的产出：

1. **decision 文档**（decisions/）：记录选择方案的理由
2. **规约更新**：如果提案涉及设计变更，对应的 specs/engineering/ 文档走 Reopen 更新

### 第四阶段：代码实现（src/）

规约更新后开始编码。代码实现遵循 specs/engineering/ 中的设计文档。

### 第五阶段：闭合验证（自指）

engine 实现后：

1. `.sih/semantic.yml` 注册代码↔规约映射
2. engine 对自身 docs/ 执行验证（自指）
3. gap 状态变更：open → closed（经 engine 验证通过）

## 三、变更分级

并非所有变更都需要走完整五阶段链。按有度之法分级：

| 级别     | 流程                                      | 示例                                    |
| -------- | ----------------------------------------- | --------------------------------------- |
| 勘误     | 直接修改 + commit message 引用理由        | typo 修正、路径更新、格式修正           |
| 轻量修订 | proposal → decision（跳过 brainstorming） | 修改一条验证规则、增加一个 MCP 工具参数 |
| 设计变更 | 完整链                                    | 新增一个治理模块、改变三机流转模型      |
| 法层修正 | Canon Reopen 流程                         | 修改 stage 语义、新增或废除一条法       |

判定标准：是否改变下游的行为预期。勘误不改变任何预期。轻量修订改变实现细节但不改变接口契约。设计变更改变接口契约。

## 四、自举路径

本提案本身是链的第一个实例。以"引导阶段"模式运行：

1. 本对话 = knowledge/drafts/ 阶段（已在进行）
2. 本文档 = proposals/ 阶段
3. 人类确认推进 = decision 阶段（模拟）
4. 产出 `docs/specs/engineering/sihankor-dev-governance.sih.md` = spec 阶段

本提案不要求 grill-me 追问（追问模板本身尚未定义），不要求 engine 验证（engine 不存在）。这是第一遍——条件不完备是合理的，引导阶段的显式声明覆盖了此间隙。

## 五、待决议事项

### 5.1 gap 实体定义（阻塞项）

proposal 中引用 gap 时，当前使用 roadmap 中的 G1-G6 编号作为占位。gap 不是司衡体系的内建概念——没有法层定义，没有术层格式，没有生命周期。

需要一份独立的 proposal 定义 gap 实体。在此之前，本提案及相关 proposal 中的 gap 引用使用占位格式 `[GAP: <简述>]`，待 gap 实体定义完成后批量替换为正式格式。

### 5.2 并发提案冲突

多人协作场景下 proposal 之间的依赖冲突管理。当前项目为单人，暂不处理（知止）。

### 5.3 追问模板

grill-me 追问的具体模板（问题列表、方案对比格式）在后续独立 proposal 中细化。

## 六、后续动作

若本提案通过：

1. 产出 decision 到 decisions/
2. 产出 `docs/specs/engineering/sihankor-dev-governance.sih.md`（stage 1/3）
3. 后续所有引擎开发以此为流程参照
4. 并行推进 gap 实体定义 proposal
