---
id: 260623-1130-glossary-integration-decision
stage: 3/3
upstream: 260623-1130-glossary-integration-plan
decided-by: moc
---

# glossary 集成采纳决策

## 背景

追问引擎人工测试暴露了 Builder 术语墙：四个问题中夹杂中英文术语（nature、stage、upstream、顺因、有度、知止），不熟悉司衡哲学的用户无法理解。根因是 `glossary/zh.yml` 定义了 15 个核心术语，但追问引擎、Techne、validator、kanban 均未加载。

[glossary 集成计划](../proposals/260623-1130-glossary-integration-plan.sih.md)（stage 2/3）设计了 GlossaryLoader + 中文（English）格式 + P0/P1/P2 分步修复。

## 方案选择

| 维度 | 决策 | 法依据 |
|------|------|--------|
| 术语格式 | **采纳**：中文（English）。首次出现附定义，后续仅用中文 | 有度（最小必要标注） |
| GlossaryLoader | **采纳**：启动时加载 glossary/zh.yml，全局共享 | 损补（补足术语不注入的缺失） |
| P0 追问引擎 | **采纳**：重写 questions/hints/constraints 文案，注入术语定义 | 顺因（Builder 入口必须先修） |
| P1 Techne | **采纳**：section outlines 标题附中文（English） | 顺势（P0 后自然推进） |
| P2 Validator/Kanban | **采纳**：dao_trace 和摘要文案用 glossary | 顺势 |
| en.yml 加载 | **不采纳** | 知止（英文术语表暂作参照，不加载到产出文本） |
| Glossary 热更新 | **不采纳** | 知止（启动时加载一次即可） |

## 备选方案

| 方案 | 描述 | 不采纳理由 |
|------|------|-----------|
| A：纯英文术语 | 全部用 English，不附中文 | Builder 不理解英文术语 |
| B：纯中文术语 | 全部用中文，不附 English | Engineer 习惯中英混合，且 glossary 定义了中英对 |
| **C：中文（English）（采纳）** | 首次附定义，后续仅中文 | Builder 和 Engineer 都能消费 |

## ADR

### decided-by

moc

### 理由

1. **顺因**：追问引擎（3/3）→ 追问引擎 decision（3/3）→ glossary 集成计划（2/3→3/3）。因果链完整。
2. **有度**：P0 只修追问引擎（~120 LOC 新增 + ~80 LOC 修改），不触及 Mind 三机流转和 validator 规则逻辑。
3. **知止**：明确排除 en.yml 加载和热更新。不试图一次性解决所有术语问题。
4. **损补**：补足人定义过的术语未注入到机器产出文本的 gap。Builder 术语墙是入口阻塞——不修这个，后续追问引擎对 Builder 无效。

### 后果

#### 正面后果

| 后果 | 说明 |
|------|------|
| Builder 可理解追问 | 四个问题不再含未定义术语 |
| 约束注入附道法解释 | 每条 F/G/J/C 规则标注法溯源 |
| 自指验证 | 本 decision 引用的计划文档（2/3）如果在追问引擎实现后被重新追问生成，将自动 2/3 起步且术语自带定义 |

#### 代价与风险

| 代价 | 说明 |
|------|------|
| 文本长度增加 | 每条术语附英文和定义增加 15-80 字符，四个问题总计约 +200 字符 |
| GlossaryLoader 新增依赖 | `.sih/index.db` 之外新增对 `glossary/zh.yml` 的读取 |
| 中文（English）格式未经验证 | Builder 可能仍不理解"类型（nature）"，需实现后进行二次人工测试 |

### 可证伪条件

| 条件 | 证伪方法 | 时间窗口 |
|------|---------|---------|
| Builder 能理解追问问题 | 人工测试：Builder 用户不再回答"我不知道" | 实现后 1 周内 |
| 约束注入覆盖所有 F/G/J/C 规则 | 检查 sihankor_propose_answers 返回的 constraints 字段 | 实现后立即 |
| 不引入回归 | cargo test + cargo clippy 全通过 | 每次提交 |

## DEPS

- 260623-1130-glossary-integration-plan：glossary 集成计划
- 260622-1325-grilling-engine-decision：追问引擎采纳决策
- glossary/zh.yml：术语表权威源
