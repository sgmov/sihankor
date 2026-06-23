---
id: 260622-1325-grilling-engine
stage: 2/3
upstream: 260613-1650-sihankor-mind-design
---

# 追问引擎设计提案

## 一、问题

司衡当前覆盖治理链的中游（文档验证）和下游（AST 对齐），但**上游缺失**：用户意图在变成 proposal 之前，没有机制帮助用户精确化自己的意图。

Pocock skills 的 grilling 和 domain-modeling 提供了概念参考，但它们是人-Agent 交互的 Markdown 技能，不是治理引擎的一部分。

**需要**：一个嵌入司衡治理链的追问引擎，用元规则（道二/顺因/有度/知止）驱动追问方向，在意图 -> proposal 的过程中减少"用户说不清自己要什么"的间隙。

## 二、能力

### 2.1 触发条件

用户通过 MCP tool 调用 `sihankor_propose`，或 Mind 检测到新文档进入 `drafts/` 时自动触发。

### 2.2 追问模型

每个追问由一条元规则驱动：

| 元规则               | 追问                                                      | 目的               |
| -------------------- | --------------------------------------------------------- | ------------------ |
| 道二（意图先于代码） | "这份文档的 nature 是什么？spec/proposal/decision/note？" | 定位文档的治理身份 |
| 顺因（因果链可追溯） | "它的上游是谁？哪个已有文档授权了这个变更？"              | 建立 upstream 链   |
| 有度（力度匹配）     | "它的 stage 应该是 1/3 还是可以直接 2/3？"                | 确定治理深度       |
| 知止（知道不做什么） | "这个变更的范围明确吗？有什么明确不在范围内的？"          | 防范围膨胀         |

追问是**一次性对话**：所有问题一次性给出，用户一次性回答。不采用轮询式追问（Pocock 的风格），因为 Solo 模式的工程师不需要被反复打断。

### 2.3 提示词生成

追问完成后，产出结构化的提示词，包含：

- proposal 的 frontmatter 模板（id/stage/upstream 预填）
- 章节结构（问题描述/变更内容/影响分析/验收标准）
- 可证伪条件模板
- @limitations 占位

### 2.4 与现有 Mind 的关系

不修改 Mind 的三机流转。追问引擎是 Mind 的**上游扩展**：：在 iCL 之前运行。Mind 现有的四步分析对的是"已有文档"，追问引擎对的是"还没成文的意图"。

## 三、工程架构

### 3.1 模块

```text
src/mind/grilling.rs     # 追问引擎核心（~500 LOC）
  ├── trigger()           # 触发检测
  ├── questions()         # 元规则→追问映射
  └── prompt_template()   # 追问结果→提示词
```

### 3.2 MCP Tool

新增 MCP tool: `sihankor_propose`

- 输入: 用户的自然语言意图
- 输出: 追问结果 + 结构化提示词

### 3.3 修正闭环复用

追问 -> AI 生成 proposal -> 验证发现违规 -> 修正提示词：此闭环复用已有的 validator，不新增修正逻辑。

## 四、验收标准

| 标准                                      | 验证方法                                          |
| ----------------------------------------- | ------------------------------------------------- |
| 追问覆盖四条元规则（道二/顺因/有度/知止） | 单元测试：元规则 -> 追问映射                      |
| 提示词包含完整的 frontmatter 模板         | unit test: prompt_template 输出                   |
| 追问结果可被 validator 验证               | 集成测试：追问 -> 生成 proposal -> validator 检查 |
| 不自指违规                                | 本提案本身通过司衡 validator 检查                 |
| Code 模式追问更重（加教育层）             | 后续迭代，不在本提案范围                          |

## 五、不在范围内

- Code 模式的重追问（教育性追问）
- domain-modeling 的词汇管理（CONTEXT.md）
- grilling session 的轮询式追问（一次一题）
- 外部 LLM 调用（提示词由用户或外部 Agent 消费，追问引擎本身不调用 LLM）

## 六、影响

| 文档               | 影响                               |
| ------------------ | ---------------------------------- |
| Mind-Design spec   | 补充：追问引擎作为 Mind 的上游扩展 |
| 无新 spec 需要建立 | -                                  |

## DEPS

- 260613-1650-sihankor-mind-design：Mind 设计规范
- 240610-1030-on-sihankor-canon：法论（顺因/有度/知止定义）
- 240602-1000-on-sihankor-assay：鉴论（反推九段式）

## SEE-ALSO

- Pocock skills (grilling + domain-modeling)：概念参考
- Superpowers brainstorming：流程参考
