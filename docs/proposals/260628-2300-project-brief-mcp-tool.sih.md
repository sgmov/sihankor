---
id: 260628-2300-project-brief-mcp-tool
stage: 3/3
---
# 项目简报 MCP 工具提案

## 一、问题

### 1.1 上下文压力

司衡当前 2375 次对话轮次消耗 7M input + output tokens，Cache Read 达 206M。根源是每个 agent 启动时需要加载大量上下文，而这些上下文每次都从 KV cache 重新读取。

### 1.2 人工复述成本

每个 agent 启动时，维护者（moc）必须复述项目当前状态：哪些 branch 在跑、当前治理阶段、下一步优先级。这是一种可自动化的重复消耗。

### 1.3 现有数据源未被充分利用

司衡已有的数据源（project_status 快照、trail_context 行迹、git 状态、ci-check 输出）可以拼接出完整的项目状态摘要，但目前没有工具把它们聚合起来。

## 二、方案

### 2.1 新增 MCP 工具：sihankor_project_brief

在 `src/observe/brief.rs` 中实现，约 100 行 Rust。工具调用时从已有数据源聚合信息，拼成纯文本返回。

### 2.2 数据源

| 数据源 | 内容 | 复用逻辑 |
| --- | --- | --- |
| project_status | 项目快照 | 复用现有查询逻辑 |
| trail_context | 最新行迹 | 复用 Trail 数据结构 |
| git 状态 | branch、dirty 状态 | 复用 git2 集成 |
| ci-check 输出 | CI 状态 | 复用现有 CI 查询 |

### 2.3 输出形式

纯文本字符串（不超过 2000 tokens），格式示例：

```
## 项目状态简报
时间: 2026-06-28 22:00
分支: main (clean)
CI: passing

## 最新行迹
- 260628: 观测窗设计决策，门外模式验证通过

## 活跃 Branch
- ci-self-govern: CI 自治理阶段 1
- dsr-4-ripgrep: 已完成，待 merge

## 治理状态
- 违规总量: 21G / 0F
- stage 2/3 文档: 12 个待 ratify
```

### 2.4 约束

- 不存文件
- 不建索引
- 不改写任何现有数据结构
- agent 启动时调用一次，结果作为 system prompt 的补充输入

## 三、实现计划

### 3.1 src/observe/brief.rs

约 100 行 Rust，实现 `project_brief` 函数：

1. 查询 project_status 快照
2. 查询 trail_context 最新 N 条（上限 5 条）
3. 执行 git status 获取 branch 和 dirty 信息
4. 查询 ci-check 输出
5. 拼接为纯文本返回

### 3.2 MCP 工具注册

在 `src/observe/mod.rs` 中导出 `brief` 模块，在 `src/mcp_server/` 中注册工具名 `sihankor_project_brief`。

### 3.3 AGENTS.md 更新

在 Branch Convention 之前增加一句：agent 启动后第一件事是调用 sihankor_project_brief 获取上下文，以工具调用替代人工复述。

## 四、验收标准

- cargo build 通过
- cargo test --all-targets 通过
- sihankor_project_brief 工具可调用，返回格式正确的纯文本
- 输出不超过 2000 tokens
- 不破坏现有 project_status 和 trail_context 的任何功能
