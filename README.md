---
id: 240613-1733-readme
stage: 3/3
---

# SiHankor

司衡 — 面向 AI 编码的治理运行时（Governance Runtime）

## 概述

SiHankor（司衡）是面向 LLM agent 主导代码生成场景的治理运行时。它不是代码生成器，而是确保生成的代码对齐人的意图——通过**追问收敛**澄清意图、通过**约束注入**调度外部 Agent、通过**产出验证**检查合规、通过**修正闭环**驱动再生成。

"MCP"是司衡的查询接口之一，不是它的本体。司衡的真正形态是**治理运行时 + 强制接入点**：CI Hook、IDE Hook、Agent 编排中间件在 agent 工作流外部施加"必-为"约束，MCP/CLI 让 agent 自主查询规约与决策。

"司"意为职能管理，"衡"意为度量与平衡。司衡承认治理自身的不完备性（道四），是一个持续收敛的治理系统。

## 架构

```
┌─────────────────────────────────────────────────────┐
│  司衡治理运行时 (sihankor-core)                      │
│  ─────────────────────────────────────────────       │
│  • Mind 三机 (iCL 认知 / iWW 决策 / iCT 验证)       │
│  • GrillingEngine 追问引擎（元规则驱动意图收敛）     │
│  • Validator 六域 F/G/J 验证 (22+ rules)            │
│  • Metrics 度量管道（方差/趋势/密度/审计）           │
│  • Glossary 术语一致性守护                           │
│  • Indexer 文档发现与索引                            │
│  • Format-lint 字符级合规检查                        │
│  • Project state 治理链状态机                        │
│  • Kanban 看板生成                                   │
└────────────┬──────────────────────┬─────────────────┘
             │                      │
       ┌─────▼─────┐         ┌──────▼──────┐
       │ 强制接入点 │         │  查询接口   │
       │ (必-为)   │         │  (MCP/CLI)  │
       └───────────┘         └──────┬──────┘
                                    │
                              ┌─────▼─────┐
                              │ 22 MCP    │
                              │ tools     │
                              └───────────┘
```

## 快速开始

```bash
cargo build --release
cargo run
```

服务启动后通过标准输入/输出与 MCP 客户端通信。

## 技术栈

| 组件     | 选型                       |
| :------- | :------------------------- |
| 语言     | Rust 2024 edition          |
| 运行时   | Tokio（异步运行时）        |
| MCP 框架 | rmcp                       |
| 序列化   | serde / serde_json         |
| 存储     | SQLite（默认）/ PostgreSQL |

## 项目结构

```text
sihankor/
  src/
    main.rs                # 入口，启动 MCP 服务器
    server/mod.rs          # HTTP API + full_analysis 三机流转
    bin/rebuild_index.rs   # 离线索引重建 CLI
    core/
      models.rs            # Stage, Frontmatter, Document, ViolationSeverity
      database.rs          # SihDatabase trait + SQLite 后端
      parser.rs            # Markdown + frontmatter 解析
      validator.rs         # 六域 F/G/J 验证规则 (V-F/V-G/V-J)
      indexer.rs           # 文档发现与索引管道
      metrics.rs           # 度量管道（方差/快照/密度/审计/趋势/权衡）
      glossary.rs          # 术语表与一致性守护
      project_state.rs     # 治理链状态机
      kanban.rs            # 看板生成
      orchestrator.rs      # 管道编排
    mind/
      types.rs             # Mind 共享类型 (Cognition, DecisionProposal, Verification)
      icl.rs               # iCL 认知分析（治理定位、关系图谱、发散诊断）
      iww.rs               # iWW 决策提议（推荐动作、替代方案、理由）
      ict.rs               # iCT 五法验证（顺因/有度/知止/损补/顺势）
      grilling.rs          # 追问引擎（元规则四问 → 约束提示词）
    mcp_server/
      governance.rs        # 22 个治理 MCP 工具
    fmt/                   # Format-lint 字符级合规检查
  docs/
    specs/
      philosophy/          # 哲学七论（道/法/鉴/应/术语/论证/总纲）
      engineering/         # 工程规范 + 核心定位 + 工程映射
    decisions/             # 决策记录 (ADR)
    proposals/             # 变更提议
    knowledge/
      notes/               # 实践洞察

## 许可

MIT
