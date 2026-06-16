---
id: 260616-1216-id-timestamp-automation
stage: 1/3
---

# id 时间戳自动化

> AI agent 创建司衡文档时，id 的时间戳段频繁与实际时间不匹配。根因：AI 无实时时钟感知。

## 当前缓解

`.sih/scripts/sih-id`：3 行 bash 脚本，输出 `date "+%y%m%d%H%M"`。创建文档前运行此脚本获取时间戳前缀。

## 长期方案

作为 engine 的第一个 MCP tool `sih_generate_id`：

```
input:  语义短名 (string)
output: "YYMMDDHHMM-语义短名" (string)
impl:   chrono::Local::now() → format
```

5 行 Rust。从"人自律"变为"机器保证"。AGENTS.md 中约束 AI agent 创建文档前必须调用此 tool。

## 关联

- MVP parser proposal: `260616-1214-engine-mvp-parser`
- 可作为 engine 启动后第一个实现的 MCP tool（比 parser 更简单）
