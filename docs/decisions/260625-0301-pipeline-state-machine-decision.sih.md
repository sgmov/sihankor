---
id: 260625-0301-pipeline-state-machine-decision
stage: 3/3
upstream: 260625-0301-pipeline-state-machine
decided-by: moc
---

# 流程推进层采纳决策

## 背景

司衡每份文档独立管理 stage，但文档之间的推进依赖纯手工操作，导致系统性断链。追问引擎已收敛意图，产出的文档却仍默认 1/3，触发 G-07 循环。用户（包括 Builder）不知道下一步该做什么。

[流程推进层提案](../proposals/260625-0301-pipeline-state-machine.sih.md)（stage 2/3）设计了两层状态机（ProjectState + Document.stage）和三阶段自动推进规则。

## 方案选择

| 维度 | 决策 | 法依据 |
|------|------|--------|
| 两层状态机 | **采纳**：ProjectState（项目层）+ Document.stage（文档层） | 有度（力度分层，互不重叠） |
| 追问收敛后自动 2/3 | **采纳**：追问四个问题全部回答后，产出文档 stage = 追问中确认的值 | 顺因（追问就是收敛动作，收敛后产物不应是 1/3） |
| upstream 就绪提示推进 | **采纳**：被引用文档全部 ≥2/3 后，pending_checkpoints 提示可推进 | 顺势（不自动推进，留人确认） |
| suggest_next_action | **采纳**：基于 ProjectState + 文档状态推导推荐操作 | 损补（补充缺失的流程推进层） |
| 完整任务系统 | **不采纳** | 知止（司衡治理文档，不治理开发任务） |

## 备选方案

| 方案 | 描述 | 不采纳理由 |
|------|------|-----------|
| A：照搬 Specoder | 完整 tasks.json + Skill Chain + TaskStatus | 过重。司衡的粒度是治理文档，不是开发任务 |
| B：保持现状 | 纯手工 stage 推进 | 已知失败——当前断链状态就是证据 |
| **C：两层状态机（采纳）** | ProjectState + Document.stage | 最小增量，最大因果效果 |

## ADR

### decided-by

moc

### 理由

1. **顺因**：追问引擎（3/3）→ 流程推进层（3/3）。因果方向不逆。追问收敛是 stage 自动推进的因——追问已回答了 nature/upstream/stage/知止，2/3 起步是果。
2. **有度**：ProjectState 约 4 个字段，不新增 Rust 模块。规则修改 ≤3 条（G-07 修改 + G-11 新增 + 追问引擎 stage 默认值修改）。
3. **知止**：明确不做完整任务系统和合约引擎。ProjectState 只管理文档推进状态，不管理代码实现进度。
4. **损补**：补足了 Specoder→司衡的过程中丢失的项目级状态层。补足了 Builder 在追问后仍然不知道下一步该做什么的 gap。

### 后果

#### 正面后果

| 后果 | 说明 |
|------|------|
| 断链消失 | 追问后产出的文档从 2/3 起步，引用链不再被 G-07 打断 |
| Builder 有方向 | suggest_next_action 输出明确的"你应该推进哪份文档" |
| 自指验证 | 本 decision 引用的 proposal 是 2/3——如果 pipeline state machine 已实现，追问引擎会在生成 proposal 时就建议 2/3 起步，proposal 的 DEPS 引用的法论不会被 iCT 误判为"修改声明" |

#### 代价与风险

| 代价 | 说明 |
|------|------|
| ProjectState 持久化 | .sih/project.json 是新增运行时状态文件，需加入 .gitignore |
| suggest_next_action 可能不准 | 基于规则推导，不覆盖所有边缘情况 |
| 1/3 仍有合法存在 | 未经过追问的草稿仍然从 1/3 起步——但大部分新文档将经过追问引擎 |

### 可证伪条件

| 条件 | 证伪方法 | 时间窗口 |
|------|---------|---------|
| 追问后断链消失 | 统计追问引擎使用前后 G-07 违规数对比 | 上线后 3 个月 |
| Builder 不再说"不知道下一步" | 统计 suggest_next_action 调用次数和 Builder 的"不知道"次数对比 | 上线后 3 个月 |
| 自动 2/3 不引入伪造收敛 | 抽样检查自动 2/3 的文档，确认追问回答质量 | 上线后 1 个月 |

## DEPS

- 260625-0301-pipeline-state-machine：流程推进层提案
- 260622-1325-grilling-engine：追问引擎提案（3/3）
- 240610-1030-on-sihankor-canon：法论
