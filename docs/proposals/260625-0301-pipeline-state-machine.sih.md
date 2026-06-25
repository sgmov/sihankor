---
id: 260625-0301-pipeline-state-machine
stage: 3/3
upstream: 260622-1325-grilling-engine
---

# 流程推进层：从文档生命周期到项目生命周期

## 一、问题

### 1.1 断链不是个别现象，是系统性缺失

当前司衡每份文档独立管理 stage（1/3→2/3→3/3），但文档之间的推进依赖纯手工操作。后果：

```text
人写 note (1/3)
  → 人写 proposal (1/3, upstream = note)
    → validator: G-07, note 是 1/3 不可引用 ← 断链
  → 人推进 note 到 2/3 → 需要 decision
    → decision (1/3) → G-07 ← 又断链
```

每份新文档创建一个上游引用就制造一条断链。这不是某份文档的问题——是**文档生命周期和项目生命周期之间没有衔接层**。

### 1.2 两个具体症状

| 症状 | 表现 |
|------|------|
| 追问后产出的文档仍是 1/3 | 追问引擎已经收敛了意图——nature 确定、upstream 确认、知止明确。产出物理应 2/3 起步 |
| Builder 不知道下一步该做什么 | 有 57 份文档，12 件未完成，但 kanban 是静态视图，不告诉人"你应该推进哪一份" |

### 1.3 根因

Specoder 有四层状态（ProjectState → Skill Chain → Task → Spec Status）。司衡保留了文档自身的生命周期（stage），但丢掉了项目级的流程推进层。

## 二、方案

### 2.1 核心设计：两层状态机

```text
项目层: ProjectState (.sih/project.json)
  当前阶段, 活跃提案, 待确认检查点, 文件变更追踪

文档层: Document.stage (已有，不修改)
  1/3 → 2/3 → 3/3
```

两层之间通过规则耦合：

| 规则 | 方向 | 说明 |
|------|------|------|
| 追问收敛后文档自动 2/3 | 项目层→文档层 | 追问引擎完成回答后，产出的文档 stage 自动设为 2/3 |
| upstream 就绪则 document 可推进 | 文档层→文档层 | 被引用的文档全部 ≥2/3 后，当前文档可由 1/3 推进 2/3 |
| 当前阶段决定推荐操作 | 项目层→人 | 根据 current_stage 和未完成文档，suggest_next_action 输出建议 |

### 2.2 ProjectState 数据结构

```rust
// .sih/project.json
struct ProjectState {
    current_stage: String,          // "spec" | "proposal" | "decision" | "code" | "verify"
    active_proposal: Option<String>, // 当前活跃的 proposal id
    last_action: String,            // 上次完成的操作描述
    pending_checkpoints: Vec<Checkpoint>, // 待人工确认的检查点
    last_updated: DateTime,
}

struct Checkpoint {
    id: String,
    label: String,                  // "请确认追问引擎的四个回答"
    document_id: String,
    decision: Option<String>,       // "confirmed" | "revised" | null
}
```

### 2.3 stage 自动推进规则

**规则 1：追问收敛 → 2/3**

当追问引擎完成全部四个问题的回答后，生成的文档 stage 设为 2/3。
理由：追问已确认 nature、upstream、stage、知止——四个核心治理属性就绪。1/3 的语义是"意图未收敛"，但追问就是收敛动作。收敛后的产物不应该是 1/3。

**规则 2：upstream 就绪 → 可推进**

当文档的 upstream 全部 ≥ 2/3 后，validator 不再报告 G-07。
同时 ProjectState.pending_checkpoints 中增加一条："upstream 已全部就绪，是否推进当前文档到 2/3？"

**规则 3：G-07 修改**

当前 G-07 禁止引用 1/3 文档。修改为：禁止引用 1/3 文档，除非引用者是同一批次追问引擎产出的文档，且 1/3 文档的追问已完成。

### 2.4 suggest_next_action

```rust
fn suggest_next_action(state: &ProjectState, db: &dyn SihDatabase) -> Vec<String> {
    // 1. 有阻塞 → 提示解阻塞
    // 2. 有 1/3 文档且 upstream ≥ 2/3 → 提示推进
    // 3. 有 2/3 proposal 无 decision → 提示起草 decision
    // 4. 当前阶段 spec → 提示下一阶段 proposal
    // 5. 兜底 → 显示 kanban
}
```

不照搬 Specoder 的 Python if-else 技能链。用 Rust 的 match 表达式，基于 ProjectState + 文档 stage 的状态组合推导推荐操作。

### 2.5 与现有系统的关系

| 现有系统 | 影响 |
|---------|------|
| validator | 新增一条规则：追问收敛后文档未设为 2/3 时报告 G-11 warning |
| 追问引擎 | sihankor_propose_answers 返回的 PromptTemplate 中 stage 改为追问回答中确认的值（不再默认 1/3） |
| kanban | kanban 增加项目阶段指示器，suggest_next_action 作为 kanban 的一部分展示 |
| Mind (iCL/iWW/iCT) | 不修改。ProjectState 是 Mind 之外的独立层 |

### 2.6 替代方案

| 方案 | 描述 | 不采纳理由 |
|------|------|-----------|
| A：照搬 Specoder 完整任务系统 | tasks.json + TaskStatus 状态机 + Skill Chain | 过重。司衡的粒度是治理文档，不是开发任务。任务系统留给外部 Agent |
| B：手动 stage 推进保持不变 | 不新增 ProjectState，仅靠人的纪律 | 已知失败——当前断链状态就是证据 |
| **C：两层状态机（采纳）** | ProjectState + Document.stage，追问后自动 2/3 | 最小增量，最大因果效果 |

## 三、验收

| 标准 | 验证方法 |
|------|---------|
| 追问收敛后的文档 stage 为 2/3 | 调用 sihankor_propose_answers 后检查 PromptTemplate.frontmatter.stage |
| 1/3 文档的 upstream ≥ 2/3 时 validator 提示推进 | 单元测试：创建 1/3 proposal，其 upstream 为 2/3，validator 返回 G-11 |
| suggest_next_action 返回非空建议 | 单元测试：给定 ProjectState + 数据库状态，输出至少一条建议 |
| ProjectState 持久化到 .sih/project.json | 集成测试：创建 ProjectState → 保存 → 读取 → 验证字段一致 |
| 已有测试不回归 | cargo test 全通过 |

## 四、不在范围内

- 不照搬 Specoder 的 Skill Chain（Python if-else 规则引擎）
- 不做完整任务系统（tasks.json, TaskStatus）
- 不做合约引擎（跨层勾稽）
- 不自动推进 stage——自动 2/3 只适用于追问收敛后的新文档。已有文档的 stage 推进仍需人工确认

## @limitations

1. 追问收敛后自动 2/3 的前提是四个问题全部回答。如果用户跳过某个问题（空回答），该文档仍为 1/3。
2. suggest_next_action 的推荐基于状态规则推导，不基于机器学习或历史数据。推荐可能不覆盖所有边缘情况。
3. ProjectState 当前只追踪一个活跃 proposal——多提案并行时可能出现状态竞争。此限制在单 Agent 工作流中不触发。
4. 两层状态机的"项目阶段"概念尚未与工厂产线 kanban 的列对齐——待看板 spec 推进后统一。

## DEPS

- 260622-1325-grilling-engine：追问引擎设计提案（3/3）。本提案修改追问引擎的 stage 默认值
- 260613-1650-sihankor-mind-design：Mind 设计规范。本提案不修改 Mind，但 ProjectState 是 Mind 之外的独立层
- 240610-1030-on-sihankor-canon：法论。stage 自动推进规则需顺因/有度法依据
