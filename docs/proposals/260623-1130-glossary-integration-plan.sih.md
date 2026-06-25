---
id: 260623-1130-glossary-integration-plan
stage: 1/3
upstream: 260622-1325-grilling-engine-decision
---

# glossary 集成与术语墙修复计划

## 一、问题

追问引擎人工测试暴露了两个耦合问题：

### 1.1 术语墙

Builder 用户在四个追问面前全部回答"我不知道"。追问引擎的问题中夹杂中英文术语（nature、stage、upstream、顺因、有度、知止），不熟悉司衡哲学的用户无法理解。

### 1.2 glossary 未加载

`glossary/zh.yml` 定义了 15 个核心术语的权威中文定义，但追问引擎、Techne、validator、kanban、MCP tool descriptions 均未读取该文件。术语定义存在但未被消费。

两个问题同源：**人定义过的术语，没有注入到机器产出的文本里。**

## 二、影响范围

- **P0 追问引擎 questions**：含术语 nature, stage, upstream, 顺因, 有度, 知止。Builder 首次接触面
- **P0 追问引擎 constraints**：含术语 F-01~J-01, C-01, 道法术语。Agent 生成提示词
- **P1 Techne section outlines**：含术语 正名, 顺因, 有度。文档蓝图
- **P2 Validator dao_trace**：含术语 有度, 损补, 知止。验证结果
- **P2 MCP tool descriptions**：含术语 iCL, 顺因/有度/知止。tool 列表
- **P2 Kanban summaries**：含术语 nature, stage 显示名。看板

## 三、方案

### 3.1 术语格式

统一采用中文（English）格式。首次出现附定义，后续出现仅用中文。

```text
首次出现：类型（nature）：spec = 系统定义 | proposal = 变更提案 | decision = 架构决策 | note = 实践笔记
后续出现：类型（nature）
纯内部：nature
```

### 3.2 GlossaryLoader

```rust
// src/core/glossary.rs

pub struct GlossaryLoader;

impl GlossaryLoader {
    pub fn load(path: &Path) -> Result<Glossary, Error>;

    /// 为术语提供首次出现的带定义文案
    pub fn first_use(&self, term: &str) -> String;
    // "类型（nature）：spec = 系统定义 | proposal = 变更提案..."

    /// 为术语提供后续出现的简洁文案
    pub fn reuse(&self, term: &str) -> String;
    // "类型（nature）"

    /// 为道法术语提供带溯源的提示文案
    pub fn dao_hint(&self, law: &str, detail: &str) -> String;
    // "有度（力度匹配）：收敛恰到好处，规约不多不少。此处要求：表格不超过 3 列"
}
```

启动时加载一次（glossary/zh.yml < 2KB），全局共享。

### 3.3 分步修复

#### P0：追问引擎（questions + constraints）

| 当前 | 修复后 |
|------|--------|
| 这份文档的 nature 是什么？ | 这份文档的类型（nature）是什么？spec = 系统定义 / proposal = 变更提案 / decision = 架构决策 / note = 实践笔记 |
| 它的上游是谁？ | 它的上游文档（upstream）是谁？上游 = 授权本文档变更的已有文档 id |
| stage 应该是 1/3 还是 2/3？ | 成熟度（stage）：1/3 = 初稿待讨论 / 2/3 = 方案已收敛可推进 / 3/3 = 已定稿 |
| 顺因/有度/知止标题 | 顺因（Shun-Yin · 因果链可追溯）：尊重因果方向。意图先于规范，规范先于实现 |
| [F-05] 正文禁止出现水平线 | [F-05 · 有度] 正文禁止出现水平分隔线 ---。有度 = 力度匹配，规约不多不少 |
| [G-06] 禁止 emoji | [G-06 · 损补] 禁止 emoji 及其他非 ASCII 符号（U+2014 等）。损补 = 损有余补不足 |

修改文件：`src/mind/grilling.rs`（注入 glossary 引用，重写 questions/constraints/hints 文案）

#### P1：Techne section outlines

| 当前 | 修复后 |
|------|--------|
| 一、正名：xxx 是什么 | 一、定义（正名）：xxx 是什么 |
| 二、顺因：治理依据 | 二、溯因（顺因）：治理依据。顺因 = 尊重因果方向 |
| 三、有度：xxx 的边界 | 三、边界（有度）：xxx 的边界。有度 = 规约不多不少 |

修改文件：`src/mcp_server/governance.rs` `build_section_outline()` 函数

#### P2：Validator / MCP / Kanban

| # | 修改 |
|---|------|
| 4 | validator dao_trace 字段：从 "有度" 改为 "有度（力度匹配，规约不多不少）" |
| 5 | MCP tool descriptions：追加 glossary 术语定义 |
| 6 | kanban summaries：nature/stage 显示名用 glossary 文案 |

### 3.4 不纳入本次范围

- en.yml 英文术语表加载：英文术语表当前仅作参照，暂不加载到产出文本中
- Glossary 热更新：启动时加载一次，不支持运行时修改 glossary 后热更新

## 四、验收标准

| 标准 | 验证方法 |
|------|---------|
| 追问引擎四个问题中无未定义术语 | 人工测试：Builder 用户能理解问题含义 |
| 约束注入每条 F/J/G/C 规则附道法解释 | 检查 `sihankor_propose_answers` 返回的 constraints 字段 |
| GlossaryLoader 正确解析 zh.yml | 单元测试：15 个术语的 first_use/reuse 输出 |
| 已有测试不回归 | `cargo test` 全通过 |
| 自身通过 validator | `sihankor validate` 无 Error |

## 五、影响

| 文档/代码 | 影响 |
|-----------|------|
| src/core/glossary.rs | 新增：GlossaryLoader（~120 LOC） |
| src/mind/grilling.rs | 修改：questions/constraints/hints 文案（~80 LOC 变更） |
| src/mcp_server/governance.rs | 修改：section outlines（~30 LOC 变更） |
| src/core/validator.rs | 修改：dao_trace 文案（~10 LOC 变更） |
| src/core/kanban.rs | 修改：摘要文案（~15 LOC 变更） |
| 无新文档 | 不需要新建 spec/reference |

总代码量：新增 ~120 LOC + 修改 ~135 LOC。

## @limitations

1. 术语定义存在歧义空间——"道"的 glossary 定义为"代码工程的因果必然性"，Builder 用户仍可能不理解"因果必然性"。glossary 本身也需要 Builder 版本。
2. 中文（English）格式增加文本长度——每条术语附英文约增加 15-30 字符，四个问题总计增加约 100 字符。在 MCP text 返回中可接受，在窄终端中略显拥挤。
3. 不覆盖 iCL/iWW/iCT 的 JSON 输出——这些是 machine-facing，保持原术语。

## DEPS

- 260623-1100-grilling-engine-builder-gap：术语墙 gap 记录
- 260622-1325-grilling-engine：追问引擎设计提案
- glossary/zh.yml：术语表权威源
