---
id: 260616-1214-gap-entity-definition
stage: 1/3
upstream:
  - 240610-1030-on-sihankor-canon
  - 260616-1200-sihankor-dev-governance
---

# gap 实体定义

> gap 是道四在语义层的可观测表现——spec 声明的意图与代码实现之间的可验证不一致。当前体系中 gap 只是概念级存在，没有法层定义、术层格式和治理生命周期。引擎开发链第五步（闭合验证）需要 gap 作为可引用的实体。

## 一、法层定义（Canon 新增）

gap 是司衡治理体系中的一等实体：**spec 声明的意图与代码实现之间的可验证不一致**。

gap 的构成性条件：

- **可引用**：gap 必须声明在 specs/ 文档中，有确定的位置（文档 id + 锚点）
- **可验证**：gap 必须是可被 engine 或人类检验的——"可能存在 bug"不是 gap，"refund_timeout 在 spec 中声明为 24h 但在代码中为 48h"是 gap
- **有所有者**：gap 归属于某个 spec 文档

gap 生命周期：

```
open ──→ in-progress ──→ closed
  ↓           ↓
  ↓           ↓ (reopen: 验证发现未解决)
  ↓           ↓
  └──→ invalidated
```

| 状态          | 含义                                      | 触发                                |
| ------------- | ----------------------------------------- | ----------------------------------- |
| `open`        | gap 已被识别并记录，尚未有人处理          | 初始创建                            |
| `in-progress` | 有 proposal 正在处理此 gap                | proposal stage ≥ 2/3 且引用了此 gap |
| `closed`      | gap 已被验证解决                          | engine 验证通过，或人类确认         |
| `invalidated` | gap 不再适用（spec 已变更、代码已删除等） | 相关文档变更时标记                  |

## 二、术层格式（Conventions 新增）

### 2.1 gap 声明

在 specs/ 文档中以 `@gap` 标注声明：

```markdown
@gap: refund-timeout-mismatch
  scope: specs/payment.sih.md#24h-refund
  description: spec 声明退款超时为 24 小时，代码中 REFUND_DEADLINE_HOURS 为 48
  severity: high
  status: open
  created: 2026-06-16
```

### 2.2 proposal 引用 gap

```yaml
# proposal frontmatter 或正文中
addresses-gap: refund-timeout-mismatch
```

proposal 3/3 后，engine 自动将引用的 gap 状态推进到 `in-progress`。

### 2.3 gap 闭合

gap 闭合由 engine 在 semantic.yml 验证通过后自动执行，或由人类手动标记。闭合需附验证证据：

```markdown
@gap: refund-timeout-mismatch
  status: closed
  closed-by: 260616-xxxx-fix-refund-timeout
  verified: engine-semantic-check
  closed-date: 2026-06-17
```

## 三、与现有机制的关系

| 机制                       | 定位                              | 与 gap 的关系                                                     |
| -------------------------- | --------------------------------- | ----------------------------------------------------------------- |
| `@limitations`             | spec 作者声明"这份规约已知不完备" | limitations 是 gap 的潜在来源——一个 limitation 可能衍生出多个 gap |
| `@deviation`               | 单次执行的故意偏离                | deviation 是 gap 的实例——某次执行故意制造了一个 gap               |
| `fidelity`（semantic.yml） | 单个语义映射的精度                | fidelity: broken 对应的就是一个 gap                               |

## 四、引导阶段 gap 占位

engine 尚未实现——gap 的自动化管理（状态流转、proposal 引用校验、闭合验证）在引导阶段由人手动执行。

当前过渡格式 `[GAP: 简述]` 在 gap 实体 ratify 后批量替换为 `@gap` 正式格式。

## 五、影响文件

- Canon：$6 新增 gap 条目（定义见 $一）
- Conventions：新增 gap 格式章节（定义见 $二）
- Engineering-Mapping：道四 → 工程实现的映射表新增 gap 行
