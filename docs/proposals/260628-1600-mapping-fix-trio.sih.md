---
id: 260628-1600-mapping-fix-trio
stage: 3/3
upstream: 260628-1100-engineering-mapping
---

# 工程映射三连修复

> 修复工程映射中三处名实不符：dao_trace 降级、J 语义校准、链 2 映射归位。

## 一、问题总览

工程映射审计揭示了三处名实不符。三者共享一个根因：**映射链的声称与代码实现之间存在结构性不一致，且不一致的方向是代码单方面偏离了哲学层定义或映射链声明**。

| 编号 | 问题 | 当前状态 | 违反 |
|------|------|----------|------|
| 链 3 | `dao_trace` 声称追溯道层原则，实际产出硬编码字符串 | L4 装饰性映射 | 名实不符 |
| 链 12 | J（矩）哲学定义为"精确判定 pass/fail"，代码实现为"静默记录" | 代码注释承认反转，哲学层未同步 | 顺因（G2） |
| 链 2 | 代码侧意图恢复映射到 iCL，但 iCL 只分析文档不分析代码 | 映射错位 | 映射目标错误 |

三者都需要"诚实化 + 归位"而非"降级或扩展"。

## 二、修复一：链 3 — dao_trace 降级

### 2.1 问题

`DaoTrace` 结构体（`types.rs:238`）和两处 `build_dao_trace`（`ict.rs:396`、`grilling.rs:331`）产出内容与道层追溯无关：

- `ict.rs:396-450`：对每条非 Pass 的五法检查，硬编码配对道层字符串（如 `"道二 + 道三"`），不基于实际道层主张或锚定
- `grilling.rs:331-334`：返回固定字符串，参数 `_nature` 被忽略

二者均声称追溯"道/法原则"，实际产出为静态文本。L4 评级准确——装饰性映射。

### 2.2 修复：重命名 + 降级 + 职责收缩

**重命名**：`DaoTrace` → `LawViolationSummary`，字段 `dao` → `laws`，`trace` → `detail`。含义从"道层追溯"降级为"法检结果摘要"。

**注意**：`dao` 字段重命名为 `laws` 不是纯机械操作。当前 `dao` 字段内容为道层归因字符串（如"道二 + 道三"），重命名后 `laws` 字段内容需从道层归因变更为法层引用（如"顺因"）。此为语义变更。

传播链：

| 位置 | 当前 | 修改为 |
|------|------|--------|
| `types.rs:238` — struct | `DaoTrace { dao, trace }` | `LawViolationSummary { laws, detail }` |
| `types.rs:211` — `Verification` 字段 | `dao_trace: Vec<DaoTrace>` | `law_violation_summary: Vec<LawViolationSummary>` |
| `ict.rs:396` — 函数 | `build_dao_trace()` | `build_law_violation_summary()` |
| `ict.rs:25,30` — 调用点 | `dao_trace` | `law_violation_summary` |
| `grilling.rs:55` — `GenerationPlan` 字段 | `dao_trace: String` | 删除此字段 |
| `grilling.rs:331` — 函数 | `build_dao_trace(_nature)` | 删除整个方法 |
| `grilling.rs:169,185` — 调用点 | `let dao_trace = ...` | 删除 |
| `server/mod.rs:108-130` | `verification.dao_trace` | `verification.law_violation_summary` |
| `governance.rs:428` — MCP tool description | `"dao trace"` | `"law violation summary"` |

**GrillingEngine 特殊处理**：GrillingEngine 不做五法检验，其 `build_dao_trace` 返回固定字符串，`_nature` 参数被忽略。删除整个方法和字段，`GenerationPlan` 不再声称产出道追溯。

**职责收缩**：降级后，`LawViolationSummary` 只描述法检结果（哪条法 Fail 了、为什么 Fail），不做道层归因。道层归因由工程映射表承担——映射表已经逐条记录了每条法从哪条道推导。

**L 级别**：从 L4 降为 L3。不是因为功能变弱，是之前的 L4 基于虚假声称。降级是诚实化。

## 三、修复二：链 12 — J 语义校准

### 3.1 问题

法论 §有度 第 164 行定义 J（矩）为"精确判定，pass/fail"。代码将 J 实现为"静默记录"（仅计数不阻断），代码注释（`models.rs:224-229`、`validator.rs:171-175`）承认反转并以"代码语义为准"。

违反顺因（G2）：规约先于实现，实现不能单方面修改规约。

### 3.2 根因：F/G/J 被压扁的二维空间

原始定义混淆了两个独立维度——**判定方式**（机械 vs 人工）与**阻断行为**（阻断 vs 标记 vs 记录）。

| | 阻断 (block) | 标记 (warn) | 记录 (record) |
|---|---|---|---|
| **机械判定** | F（戒）| — | J（矩）← 代码的真实位置 |
| **混合判定** | — | G（规）| — |

J 的哲学定位落在"机械判定 + 阻断"格子，工程实践发现这个格子实际对应的是 F。J 在工程中的真实需求落在"机械判定 + 记录"格子——需要精确的机械检查，但不阻断。

### 3.3 修复：分解维度 + 校准

**法论 §有度力度表**（第 160-164 行）从三列扩展为四列：

| 力度 | 判定方式 | 阻断行为 | 适用场景 |
| ---- | ---- | ---- | ---- |
| 戒 F | 机械判定 | **阻断**（违反即拒绝） | 导致系统性不可维护性的操作 |
| 规 G | 混合判定 | **标记**（偏离须声明） | 偏离应被看见但不阻断的操作 |
| 矩 J | **精确机械判定** | **记录**（静默计数，不阻断） | 可机械检查但属风格性判断的操作 |

新增校准说明（插入法论 §有度 第 164 行之后）：

> **校准 2026-06-28**：J（矩）原定义为"精确判定，pass/fail"。工程实践发现此定义混淆了判定方式与阻断行为。经校准，J 的"精确判定"特性保留——J 级规则的判定方式仍然是精确机械判定——但阻断行为从"pass/fail 阻断"调整为"静默记录，仅计数不阻断"。校准标准来自 G3（有度：力度与风险匹配——J 级属低风险风格性检查，应对应低力度静默记录）；校准程序来自 G2（顺因：哲学层显式承认并重新定义，代码不单方面修改规约）。

**代码注释校准**：

`models.rs:224-229` 和 `validator.rs:171-175` 从"代码单方面反转，以代码语义为准"改为"经 G3 校准：J = 精确机械判定 + 静默记录。判定方式保留机械精确性，阻断行为由 pass/fail 校准为静默记录。见法论校准 2026-06-28。"

**同步目标**：
- `SiHankor-Terminology-Lineage.sih.md`：司衡层 J 定义引用同步更新，道家源出（《离娄》）不变
- `SiHankor-Reconstruction-Plan.md:578,600,602`：标记为"已通过 260628-1600 法论校准解决"
- `R4-engineering-mapping-audit.md`：不动（历史审计记录）

**不修改的对象**：代码执行逻辑不变、现有 J 级规则（如 V-J-01）不变、`ViolationSeverity` 枚举名（`Judgment`）保留。

## 四、修复三：链 2 — 映射归位

### 4.1 问题

工程映射链 2 将"道二 → 意图恢复流程"映射到 iCL，声称覆盖文档侧和代码侧。实际 iCL 只分析文档（`.sih.md`），不从代码恢复意图。

问题不是"代码侧未实现"（那只是 L5），而是"**映射到了错误的目标**"。iCL 的设计原则是只读文档不读代码——代码侧意图恢复不在 iCL 的职能范围内，这是知止的正确执行，不是遗漏。

### 4.2 修复：链 2 收窄 + 代码侧归位

**链 2 收窄为仅文档侧**：

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L2 |
| 哲学要求 | 文档生成前意图必须显式化 |
| 工程现状 | GrillingEngine 四问收敛 + spec-coding 规范产出 |
| 偏差 | 固定四问模板，非完整意图恢复流程；产出为提示词模板，意图恢复依赖外部 Agent 执行 |

链 2 不再声称覆盖代码侧。

**代码侧归位**：代码侧意图恢复的唯一归位目标是链 15（行迹机制，道三）。行迹的读取由 iCL 承担（从行迹记录恢复意图形成过程），写入由治理层（MCP 工具）承担。当前 L5，不修改。

## 五、三连修复的相互关系

三者不是独立的补丁，共享同一个深层模式：

**代码在实践中发现了哲学定义/映射链声明的不精确之处，但处理方式是"单方面偏离"而非"通过治理流程校准"。**

修复的统一原则：**诚实声明当前状态，将代码的工程发现通过顺因流程反向同步到哲学层，而不是代码单方面修改规约。**

## 六、验证方式

修复完成后，以下检查应通过：

1. `cargo build` 编译通过（重命名涉及多个文件）
2. `cargo test` 测试通过（重命名可能影响测试中的字段引用）
3. grep `dao_trace`/`DaoTrace` 在 `src/` 下返回零匹配（grilling 删除后）
4. grep `law_violation_summary`/`LawViolationSummary` 在所有受影响文件中一致
5. 法论 §有度力度表新增"判定方式"和"阻断行为"两列
6. 法论 §有度新增校准说明（2026-06-28）
7. `models.rs` 和 `validator.rs` 的 J 注释从"以代码语义为准"改为"经 G3 校准"
8. 工程映射链 2 不再声称覆盖代码侧，代码侧归位到链 15（行迹）
9. 工程映射问题三和问题四的措辞同步更新

## DEPS

- 260628-1100-engineering-mapping
  - 工程映射，修复的母文档
  - [工程映射](../specs/engineering/Engineering-Mapping.sih.md)
- 260628-1000-fa-on-governance-principles
  - 法论，J 语义校准的修改目标
  - [法论](../specs/philosophy/Canon-On-Governance-Principles.sih.md)

## SEE-ALSO

- 260628-1500-trail-mechanism
  - 行迹机制提案，链 15 的定义来源
  - [行迹机制](../proposals/260628-1500-trail-mechanism.sih.md)
- R4-engineering-mapping-audit
  - [R4 工程映射审计](../../review-results-v3/R4-engineering-mapping-audit.md)
