---
id: 260628-1500-trail-mechanism
stage: 3/3
upstream: 260627-1030-sihankor-philosophy
---
# 行迹机制提案

## 一、问题

### 1.1 跨 agent 上下文丢失

司衡引擎在多个 agent、多轮会话中构建。每个新 agent 启动时没有前序对话的记忆和上下文。当前机制层只有 spec-coding（意图的产出形态）和三机（规范的验证），意图的形成过程没有被任何机制持久化。

### 1.2 道三的直接命中

道三声明"代码自晦，意图必复"。spec-coding 记录意图的最终形态（规范），不记录意图如何到达这个形态（转折过程）。当新 agent 需要理解决策背后的意图时，只能从最终产物反推，反推的间隙随时间增大（道四b）。

### 1.3 应论的经验证据

应论的 23 条挑战条目全部从已丢弃的对话中重新提取。鉴论 4.4 节承认九段式"完整设计意图可能未完全写入文档"。这些都是行迹未被记录导致的重复消耗。

## 二、授权依据

### 2.1 道层授权

道二（意图先于代码）：意图形成过程属于代码工程活动的核心部分，不是治理的元记录。

道三（代码自晦，意图必复）：行迹是意图恢复的原材料，是道三在机制层的直接展开。

### 2.2 与 spec-coding 的关系

行迹与 spec-coding 覆盖意图生命周期的不同阶段：

| 机制 | 覆盖阶段 | 持久化内容 | 道层依据 |
| ---- | ---- | ---- | ---- |
| spec-coding | 意图 -> 规范 | 意图的产出形态（规范文档） | 道二 |
| 行迹 | 意图形成过程 | 方向性转折及其理由 | 道三 |

二者互补不重叠：spec-coding 记录"意图变成了什么"，行迹记录"意图是如何变成这样的"。

## 三、机制层授权

### 3.1 机制层 5.1 补充

在总纲机制层 5.1 的现有两项（spec-coding、三机）之后，新增第三项：

行迹（Trail）：记录意图形成的关键转折点，是道三"意图必复"在机制层的直接展开。行迹持久化方向性转折（从 A 转向 B 及其理由），不记录探索过程的细节。行迹记录的写入由治理层（MCP 工具）承担，读取由 iCL 在意图定位中纳入。

### 3.2 知止边界

行迹只记录方向性转折：意图方向发生根本转变的时刻及转变理由。不记录：探索过程的细节讨论、已被 ADR 覆盖的决策结果、一次性的实现细节。

行迹粒度为细粒度：每个转折点对应一条行迹。行迹数量随转折点线性增长，非指数增长。膨胀风险不在数学结构，在转折点定义模糊导致阈值漂移：从只记录根本转折滑向记录所有意见交换。知止约束通过准入门槛防止阈值漂移。

### 3.3 行迹准入的分工

行迹准入不是机械判断，是 LLM 辅助评估加人类裁决。三个准入条件均为语义判断，无法机械化：

**准入条件**（全部满足才记录，由 LLM 辅助层评估）：

- 方向性转折：意图方向发生根本转变，不是细节调整
- 可重复遭遇：未来 agent 或审阅者可能独立遇到相同问题
- 非已被覆盖：该转折的理由未被 ADR 或应论条目覆盖

**排除条件**（满足任一则不记录，由确定性引擎机械裁决）：

- ADR 已覆盖（文档 ID 匹配）
- 应论条目已覆盖（条目 ID 匹配）

裁决分工遵循鉴论 2.2 节"裁决权属于确定性引擎"：

| 层 | 职责 | 产出 |
| ---- | ---- | ---- |
| 确定性引擎 | 检查机械可验证的排除条件 | 裁决：排除或放行 |
| LLM 辅助层 | 评估转折点是否满足三个准入条件 | 候选建议加置信度 |
| 人类治理者 | 对 LLM 候选建议做最终判断 | 决定是否记录 |

### 3.4 数据成熟度

行迹机制当前为待验证。与应几类似，先建立哲学授权和机制定位，待跨 agent 实践积累后再决定完整工程化。初始工程映射 L 级别标为 L5（零实现）。

### 3.5 行迹的数据结构（经原型验证 + 全量重评估修正）

行迹原型 T1-T5 揭示了设计提案与实际使用之间的偏差。本次重评估（260628-1443-trail-design-reevaluation）进一步基于 J1-J6 六个判断点修正数据结构：

每条行迹记录包含四类字段：

**锚定字段**（将行迹绑定到所属文档链）：

| 字段 | 内容 | 示例 |
| ---- | ---- | ---- |
| trace_id | 行迹唯一标识 | 260628-1900-trail-t1 |
| anchor_doc | 链归属文档 ID（单值） | 260628-1700-dsr-cycle |

anchor_doc 的语义是 **这条行迹归属到的主链文档 ID**——即行迹记录后果影响最大的文档。它不指"导致转折的源头文档"（源头常常不是文档，如 T2 的 MCP server stdio 限制；T3 的 SiHankor-A 外部系统），也不指"转折时正在处理的对象"（处理对象在多文档交叉工作时会有歧义）。链归属语义直接对应 iCL 的查询逻辑（见 §4.4）。

次级关联文档（如 T1 的 mapping-fix-trio 提案、T4 的 rebuild_index 源码、T5 的 infer_nature 源码）写入 `rationale` 或 `consequences` 字段，由全文搜索二次命中。

**转折字段**（记录转折本身）：

| 字段 | 内容 | 示例 |
| ---- | ---- | ---- |
| trace_type | 行迹类型 | direction_shift / method_selection / discovery |
| turning_point | 转折描述 | 原计划 DSR 优先，交叉审阅发现工程集成缺失后调整顺序 |
| rationale | 转折理由 | 堵漏优先于冒险，基线数据需要诚实化 |
| consequences | 后果 | DSR 启动晚一个周期，但基线质量更高 |

**溯源字段**（记录参与者和来源）：

| 字段 | 内容 | 示例 |
| ---- | ---- | ---- |
| agents_involved | 参与 agent 异构配置标识（reserved） | DS-v4-pro / GLM-5.1 |
| created_at | 记录时间 | 260628-1900 |

agents_involved 当前无数据（DSR-1 单 agent），定义为可选 reserved 字段，待多 agent 场景出现时启用。

**披露字段**（控制行迹的可见性）：

| 字段 | 内容 | 示例 |
| ---- | ---- | ---- |
| disclosure_level | 披露级别（reserved，默认 public） | public / internal |

disclosure_level 当前全部 public，定义为可选 reserved 字段，待敏感场景出现时启用。

行迹类型三种（精简自原 4 种）：

- direction_shift（方向转折）：意图方向发生根本转变的时刻及转变理由。T1 属于此类
- method_selection（方法选择）：意图不变、在约束条件下选择实施方法。T2（rebuild_index 替代 MCP server）和 T4（cp -r 替代 ln/mv）属于此类
- discovery（间隙发现）：跨治理系统的格式间隙、工程约束揭示的设计缺口等非决策类发现。T3（SiHankor-A Stage 格式间隙）、T5（Archive Nature 缺口）属于此类；T4 的"暴露 docs_dir 硬编码"方面也属于此类

删除原 decision_origin 和 divergence 类型，理由：二者为多 agent 场景设计（前者追溯"单一还是共识"，后者记录"交叉分歧"），DSR-1 单 agent 执行无数据。原型 0 数据是设计正确的信号而非遗漏。多 agent 类型的设计推迟到有真实数据时——基于真实数据定义比基于猜测定义可靠。

原设计中曾存在的 `summary` 字段被删除，理由：与 `turning_point` 信息完全重叠（"意图方向从 DSR 优先变为工程修复优先" 与"修复顺序从 DSR 优先改为工程集成优先"是同一件事的不同措辞）。

行迹存储格式为知识库文档合集。行迹少于 20 条时保持单文件格式（如 knowledge/trails/260628-1900-dsr1-trails.sih.md），超过 20 条后按时间分文件。

### 3.6 行迹的披露级别

行迹对司衡开发者、外部审阅者、社区贡献者披露。披露分两级：

- public：对所有人披露，包括外部审阅者和社区贡献者。方向转折和方法选择默认 public
- internal：仅对司衡开发团队披露。未收敛的开放问题标记为 internal，待收敛后升级为 public

### 3.7 行迹的生命周期

行迹遵循司衡文档生命周期：propose -> resolve -> ratify。新行迹以 propose 形式加入，经验证后 resolve，最终 ratify 为定论行迹。行迹不可删除，只能标记为 superseded（被新行迹取代）。

行迹以 .sih.md 文档形式存在于 knowledge/trails/ 目录，遵循司衡文档目录治理。行迹的披露不依赖 iCL：外部审阅者和社区贡献者可以直接阅读行迹文档，不需要通过三机。

行迹文档的 nature 值为 `trail`（独立于 note）。其在文档目录结构中的定位：

| Directory | Nature | Stage | Description |
| --------- | ---- | ----- | ----------- |
| `knowledge/trails/` | trail | 1/3->2/3->3/3 | Intent turning points and direction shifts |

trail nature 与 note nature 的区别：note 记录"我们学到了什么"，trail 记录"意图在何处发生了方向性转折"。两者语义不同，各自独立索引和查询。

### 3.8 行迹与现有机制的关系

行迹、relation_graph.gaps、ADR 是三个不同的抽象层，职责独立：

| 抽象层 | 产生方式 | 答什么问题 | 例子 |
| ---- | ---- | ---- | ---- |
| relation_graph.gaps | 文档结构自动推导 | "缺什么？" | "archive nature 是 None" |
| discovery 行迹 | 人在转折点记录 | "何时发现 + 为什么重要" | "DSR-1 时发现 archive nature 是 None，因为 infer_nature 没处理 archive 目录" |
| ADR | 决策 ratify 后定稿 | "决定做什么" | "决定 docs_dir 改为可配置"（如果做的话） |

三层独立维护，不自动同步。理由：

1. 语义不同——gap 答"缺什么"（declarative），trail 答"何时发现 + 为什么"（narrative），ADR 答"决定做什么"（prescriptive）。强行翻译会丢失信息
2. 重叠是 feature——同一发现（如 archive nature 缺口）出现在 trail 和 gap 中，人类审阅可交叉确认；如果只在 trail 中未来可能被遗忘；如果只在 gap 中丢失发现时刻和原因
3. 零翻译规则——三层各自定义，演化互不干扰
4. 升级路径明确——trail 不直接升级为 ADR。trail 是 chronicle，ADR 是 ratification。一个 method_selection trail 在事后被讨论、决议后，才会单独写 ADR；ADR 引用 trail 作为"决策来源"

discovery 行迹的"事实陈述"部分可能与 relation_graph.gaps 重叠，method_selection 行迹的"决策结果"部分可能被后续 ADR 引用——这些是正常的引用关系，不是同步关系。

## 四、与三机的关系

### 4.1 iCL 的扩展

iCL 的第一项职能"意图定位"天然匹配行迹的恢复面。iCL 已具备 db 访问权和上游链追溯能力。行迹记录存储在数据库中时，iCL 在 `governance_position` 分析中纳入行迹：当分析某个 decision 文档时，同时检索与其关联的行迹记录，将意图形成过程纳入 Cognition 产出。

这个扩展不违反 iCL 的设计原则：仍然只读不写，仍然只产出 Cognition，信息来源从文档扩展到文档加行迹。

### 4.2 记录面不在三机职能范围

三机的设计原则是"只产出不写入"。行迹记录的写入走治理流程，由 MCP 工具暴露写入接口，由 agent 在关键转折点主动调用。写入操作由 indexer 和治理层承担，与三机边界一致。

### 4.3 MCP 工具接口设计（最小可行）

知止原则：行迹机制自身的治理投入必须与其产出方差成正比。在行迹不足 20 条时，投入完整 CRUD 接口是过度治理。

**写接口：单工具 record_trail**

```rust
#[tool(description = "[SiHankor] Record a trail entry (direction shift / method selection / discovery)")]
pub async fn record_trail(
    &self,
    Parameters(TrailRecord {
        anchor_doc,
        r#type,
        turning_point,
        rationale,
        consequences,
        agents_involved,
    }): Parameters<TrailRecord>,
) -> String
```

参数映射到数据结构：anchor_doc（链归属文档 ID）、type（方向转折/方法选择/间隙发现）、turning_point（转折描述）、rationale（转折理由）、consequences（后果）、agents_involved（可选 reserved 字段）。

产出：在 knowledge/trails/ 下追加一条行迹。当前行迹数不足 20 条，追加到最晚的近期的行迹集合文档；超过 20 条后新开文档。

**读接口：不新增工具，复用已有索引机制**

行迹文档被索引后，通过现有 `search_docs` 工具查询。按 `anchor_doc: {id}` 搜索即可找到所有关联行迹。设计依据：原型数据全部包含 anchor_doc 引用，搜索引擎可匹配。

```mermaid
flowchart LR
    A["Agent 遇到转折点"] --> B["调用 record_trail"]
    B --> C["写入 knowledge/trails/"]
    C --> D["rebuild_index 索引后<br/>search_docs 可检索"]
    D --> E["iCL analyze 读取行迹"]
```

### 4.4 iCL 扩展设计

iCL 的 analyze 方法当前产出三项：governance_position、relation_graph、divergence_diagnosis。

扩展后新增第四项 trail_context：

```
iCL.analyze(doc) 产出（扩展后）:
  - governance_position (nature/stage/upstream_chain/role_in_chain)
  - relation_graph (references/duplicates/conflicts/gaps)
  - divergence_diagnosis (divergence[])
  + trail_context: {
      trails: TrailRef[],  // 与本文档上游链关联的行迹
      trail_count: usize,
    }
```

trail_context 的采集过程：iCL 在完成 `resolve_upstream_chain` 后，对链上的每个文档 ID 执行一次 `search_content` 搜索 `anchor_doc: {id}` 模式，收集所有关联行迹。这一步不新增数据库查询——search_content 已存在。

将此扩展表述同步更新到工程映射链 15。

### 4.5 知止边界（价值门槛）

行迹机制本身受知止约束。原提案基于数字门槛（5 次调用 + 3 次 analyze + 10 条降级），重评估后改为价值门槛——数字本身无依据，使用频率不等于使用价值。

**V1（最低价值门槛 / 保留门槛）**：1 个 DSR 周期内，至少 1 条行迹揭示了**未被 ADR/Settle 覆盖**的发现或决策。

满足 V1 意味着行迹机制产生独立信息，不能被现有机制替代。

DSR-1 验证：T3（SiHankor-A stage 格式间隙）、T4（docs_dir 硬编码）、T5（archive nature 缺口）均不在 ADR/Settle 中。V1 已满足。

**V2（持续价值门槛 / 推进门槛）**：3 个 DSR 周期后，行迹数线性增长（不是 0、不是衰减），且每周期至少 1 条 unique discovery。

满足 V2 → 推进到 L4（写 Rust stub 实现 record_trail 和 trail_context）
不满足 V2 但满足 V1 → 维持 L5
两个都不满足 → 降级行迹机制优先级

价值门槛与数字门槛的关键差异：数字门槛衡量使用频率（5 次调用）；价值门槛衡量边际信息（行迹是否揭示了 ADR/Settle 未覆盖的内容）。知止的本质是产出方差——新信息才是方差，不是调用次数。

## 五、工程映射

### 5.1 新增映射链

新增链 15：道二/道三 -> 行迹记录。初始 L 级别 L5（零实现）。

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L5 |
| 哲学要求 | 意图形成过程可被恢复（道三：代码自晦，意图必复） |
| 工程现状 | 零实现 |

链 15 承担两个道层主张的工程展开：

- 道三（代码自晦，意图必复）：行迹是意图恢复的原材料。道三要求意图可恢复，行迹记录意图形成过程使恢复成为可能
- 道二（意图先于代码）：意图形成过程属于代码工程活动的核心部分。行迹记录的正是意图如何形成的

链 15 同时承接链 2 代码侧意图恢复的归位。链 2（道二 -> 意图恢复流程）的代码侧原映射到 iCL，但 iCL 的职能是文档认知分析，不是代码意图恢复。代码侧意图恢复的正确映射对象是行迹机制：不是从代码反推意图，而是从行迹记录恢复意图。

### 5.2 链 2 的修正

链 2 修正为只覆盖文档侧（grilling 的四问收敛 + spec-coding 的规范产出），L 级别从 L2/L5 变为 L2。代码侧意图恢复归位到链 15，L5 转移到链 15。

### 5.3 链 3 的修正

链 3（道三 -> 信息损耗检测）的 dao_trace 名实不符问题作为独立问题处理。行迹是道三的新展开，不替代 dao_trace 的现有功能。dao_trace 的重命名与职责调整在名实相符修正提案中处理。

## 附录

### DEPS

- 260627-1030-sihankor-philosophy
  - 哲学总纲，机制层授权的源头
  - [司衡哲学总纲](../specs/philosophy/SiHankor-Philosophy.sih.md)
- 260627-1100-dao-on-natural-convergence
  - 道论，道三为行迹提供道层根基
  - [司衡道论](../specs/philosophy/Tao-On-Natural-Convergence.sih.md)

### SEE-ALSO

- 260627-1600-settle-on-recurring-challenges
  - 应论，行迹与应辨的跨 agent 上下文恢复互补
  - [司衡应论](../specs/philosophy/Settle-On-Recurring-Challenges.sih.md)
