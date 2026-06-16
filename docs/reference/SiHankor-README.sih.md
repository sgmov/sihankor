---
id: 260615-1500-sihankor-readme
stage: 3/3
upstream: 240602-0900-on-sihankor
---

# 司衡（SiHankor）

司衡是一个承认治理自身也不完备的代码工程收敛引擎。

**核心理念**：代码工程的意图天然发散（道一），收敛不会自动发生。司衡是代码工程自身长出来的收敛机制——通过文档治理、意图恢复、合规验证，让代码不偏离意图。

## 三条阅读路径

### 使用司衡（5 分钟）

你在使用司衡治理你的项目。看 AGENTS.md，它告诉你：

- 项目文档如何组织（`docs/` 目录结构）
- 文档如何编写（frontmatter 字段、stage 语义）
- 治理流程如何运作（proposal → decision → spec 变更）

不需要读完哲学论著。使用司衡只需要知道你的项目放在哪、怎么写、怎么流转。

### 理解司衡为什么成立（30 分钟）

你需要理解司衡凭什么可信。阅读顺序：

1. [司衡论](../specs/philosophy/On-SiHankor.sih.md) — 总纲，建立全貌
2. [司衡哲学纲要](./SiHankor-Philosophy-Compendium.sih.md) — 核心概念的权威定义
3. [司衡道论](../specs/philosophy/On-SiHankor-Tao.sih.md) — 四条道的完整阐述
4. [司衡鉴论](../specs/philosophy/On-SiHankor-Assay.sih.md) — 反推九段式与检验方法论
5. [司衡法论](../specs/philosophy/On-SiHankor-Canon.sih.md) — 收敛五法与文档治理规则

### 参与司衡开发（60 分钟）

你需要理解司衡的工程实现。阅读顺序：

1. 上面的"理解司衡"路径
2. [命名哲思](./SiHankor-Onomastic-Philosophy.sih.md) — 中英代号三重对齐
3. [工程映射](../specs/engineering/SiHankor-Engineering-Mapping.sih.md) — 哲学到工程的完整映射
4. [引擎设计摘要](../specs/engineering/SiHankor-Engine-Design-Summary.sih.md) — MCP 架构与数据模型
5. [Mind 设计](../specs/engineering/SiHankor-Mind-Design.sih.md) — 四步分析法与三机流转
6. [文档约定](../specs/engineering/SiHankor-Document-Conventions.sih.md) — stage/id/frontmatter 术层展开

## 目录概览

```
docs/
  specs/          # 系统定义（司衡的系统恰好是哲学）
    philosophy/   # 道、法、术、鉴、元
    engineering/  # 引擎设计、文档约定
  proposals/      # 变更提议
  decisions/      # 决策记录
  reference/      # 参照标准（概念纲要、命名哲思）
  knowledge/
    drafts/       # 构思碎片
    notes/        # 实践洞察
  archive/        # 废弃文档

.sih/
  config.yml      # 引擎配置
  semantic.yml    # 意图↔代码语义映射
  events/         # 自动事件记录
  index.db        # SQLite 文档索引

glossary/
  zh.yml          # 中文术语定义
  en.yml          # 英文翻译映射
```
