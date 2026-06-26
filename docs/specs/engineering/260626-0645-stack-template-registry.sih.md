---
id: 260626-0645-stack-template-registry
stage: 1/3
upstream: 260622-1400-sihankor-core-positioning
---

# 技术栈模板注册表

## 问题

当前司衡没有技术栈推荐机制。LLM 在无约束条件下生成代码时，技术栈的选择完全依赖其训练数据的偏差，导致：

1. **小白用户**不知道如何选择技术栈，甚至不知道"技术栈"这个概念
2. **专家用户**需要重复配置相同约束（每次新项目重新写一遍规则）
3. **约束注入**没有起点：约束注册表需要一个初始的来源
4. **项目初始化**没有标准化流程：每个项目从空白开始

## 解决方案

技术栈模板注册表：一套 YAML 定义的、可扩展的项目模板系统，通过 3-5 个可回答的生活问题映射到默认技术栈，同时允许专家完全覆盖。

## 模板数据结构

```yaml
# .sih/stack-registry/defaults/basic-web-api.yml
id: basic-web-api                    # 模板唯一 ID
description: "标准 Web API 项目"      # 人类可读描述
version: 1                           # 版本号，走治理链递增

match:                               # 匹配条件（小白模式的入口）
  project_type: ["web-api", "backend", "api"]
  scale: ["personal", "small-team"]
  budget: ["free", "low"]

tech_stack:                          # 技术栈定义
  language: rust                     # 编程语言
  web_framework: axum                # Web 框架
  database: sqlite                   # 数据库
  orm: diesel                        # ORM
  auth: axum-extra                   # 鉴权方案
  deploy: self-hosted                # 部署方式
  ci: github-actions                 # CI 系统

constraints:                         # 约束注册表初始条目
  - id: C-REPO-001
    text: "所有数据库操作必须通过 Repository 层"
    severity: error
  - id: C-ROUTE-001
    text: "API 路由必须带有版本前缀 /v1"
    severity: error

prompts:                             # 追问引擎用的提示词模板
  intent_questions:
    - "你的项目需要用户登录吗？"
    - "数据存在哪里（本地/云端）？"
  constraint_hints:
    - "本项目使用 Axum + SQLite，生成代码时注意 ORM 语法"
  generation_hints:
    - "使用 Diesel 的 schema.rs 自动生成数据库模型"
```

## 与 config.yml 的集成

```yaml
# .sih/config.yml
stack:
  template: basic-web-api              # 选中的模板
  overrides:                           # 专家覆盖（可选）
    database: postgres                 # 只改写数据库
    web_framework: actix               # 只改写 Web 框架
  constraints:                         # 自定义约束（追加）
    - "所有 API 必须记录请求日志"
  disabled_constraints:                # 禁用的默认约束
    - C-ROUTE-001
```

config.yml 是项目的配置入口。模板是默认值，overrides 是专家修改，constraints 是追加项。三者在运行时合并为最终的约束注册表。

## 小白入口的交互流程

```
用户: "我想做一个猫咪照片分享网站"
  │
  ├─ 追问引擎（小白模式）
  │   1. "有多少人会用？"
  │     ○ 就我自己 | ○ 朋友之间 | ○ 公开的
  │   2. "照片存在哪里？"
  │     ○ 本机硬盘 | ○ 手机相册 | ○ 云盘
  │   3. "你能接受多少费用？"
  │     ○ 免费     | ○ 每月 50 元 | ○ 无所谓
  │
  ├─ 司衡内部映射
  │   个人使用 + 本机 + 免费
  │   → 匹配 basic-web-api 模板
  │   → 生成项目骨架 + ADR + 约束注册表
  │   → 写入 .sih/config.yml
  │
  └─ 用户感知
      └── 项目已创建，可以开始写代码了
```

## 专家入口

```
专家: "新项目，Rust + Axum + Postgres"
  │
  ├─ 直接指定模板
  │   sihankor init --template basic-web-api --override database=postgres
  │
  ├─ 或从某个模板开始再修改
  │   1. 选择 basic-web-api 模板
  │   2. 在 config.yml 中添加 overrides
  │   3. 添加自定义约束
  │
  └─ 专家模式不经过生活问题，直接进入技术配置
```

## 模板的治理链

模板本身不走司衡的治理链：它们不是 .sih.md 文档，而是 YAML 配置文件。但模板的提案和审阅可以通过治理链进行：

```
docs/proposals/260700-1200-add-mobile-template.sih.md
  └── 提案新增 mobile-app 模板
  └── 审阅: 是否合道？
  └── 通过 → 模板写入 .sih/stack-registry/custom/
```

内置模板（`.sih/stack-registry/defaults/`）不可删除，但可以被覆盖。自定义模板（`.sih/stack-registry/custom/`）完全由用户管理。

## 模板匹配算法

```rust
/// 匹配算法伪代码
fn select_template(answers: &Answers, registry: &Registry) -> &Template {
    // 1. 精确匹配：所有条件都命中
    for template in &registry.templates {
        if template.match_all(&answers) {
            return template;
        }
    }

    // 2. 加权匹配：计算每个模板的匹配度
    let scored = registry.templates.iter()
        .map(|t| (t.match_score(&answers), t))
        .max_by_key(|(score, _)| *score);

    // 3. 保底：返回最通用的模板
    scored.unwrap_or(&registry.default)
}
```

## 约束注入机制

模板选择后，约束自动注入到 LLM 上下文：

```
MCP Resource: sihankor://constraints
  └── 返回当前项目所有活跃约束
  └── LLM 在生成代码时自动读取
  └── 约束来源:
      ├── 模板自带的约束
      ├── config.yml 中的自定义约束
      └── 后续通过 validator 添加的约束
```

约束注入不是 LLM 提示词的一部分：它是一个 MCP Resource，LLM 按需读取。这样不会污染 LLM 的上下文，且约束可以实时更新。

## 可证伪条件

本 spec 被推翻，如果：

1. 发现 YAML 模板无法覆盖的技术栈差异（如：需要动态计算的技术组合）
2. 小白模式的生活问题无法映射到任何已知模板
3. config.yml 的 overrides 机制导致约束冲突无法自动合并

## 实现阶段

```
P0 (MVP)
  ├── .sih/config.yml 增加 stack 字段
  ├── 3 个内置模板 (basic-web-api, serverless-web, cli-tool)
  └── 约束注入 MCP Resource

P1 (小白入口)
  ├── 追问引擎小白模式 (3 个生活问题)
  ├── 模板匹配算法
  └── 自动生成 ADR + 项目骨架

P2 (专家能力)
  ├── sihankor init CLI 交互
  ├── overrides 合并逻辑
  └── 自定义约束追加

P3 (扩展)
  ├── 社区模板提交流程
  ├── 模板版本管理
  └── 模板使用统计 + 后悔信号关联
```
