---
id: 260628-1100-engineering-mapping
stage: 1/3
upstream: 260627-1100-dao-on-natural-convergence
---
# 司衡工程映射

> 本文档逐条追踪司衡哲学概念到工程实现的映射状态。基于 V3 R4 审计发现的 L3-L5 断裂，诚实记录每条映射链的当前状态、代码证据与偏差。所有 L 级别基于实际源码验证，与初步判断有差异处已标注说明。Phase 0-2 工程补全后，链 1（L5->L4）、链 5（L5->L4）、链 13（L2->L1）的 L 级别已基于代码变更更新。

## 映射分级定义

**L1 完整映射**：哲学概念有精确的工程对应，可机械验证，无歧义。

**L2 近似映射**：工程对应存在但与哲学原意有偏差，偏差可量化。

**L3 降格映射**：哲学概念在工程层被简化为字符串、枚举或 if-else，原语义丢失。

**L4 装饰性映射**：工程实现与哲学概念无实质联系，仅为标签挂载。

**L5 无映射**：哲学概念在工程层无任何对应。

## 映射链总览

以下为 14 条映射链的 L 级别汇总。道层 5 条，法层 5 条，机制层 4 条。

| 编号 | 映射链 | L 级别 |
| ---- | ---- | ---- |
| 1 | 道一 -> 产出方差度量 | L4 |
| 2 | 道二 -> 意图恢复流程 | L2 / L5 |
| 3 | 道三 -> 信息损耗检测 | L1 / L4 |
| 4 | 道四a -> 间隙声明 | L1 |
| 5 | 道四b -> 跨版本一致性检查 | L4 |
| 6 | 知止 -> G1 Scope Boundary | L5 / L3 |
| 7 | 顺因 -> G2 Causal Alignment | L2 |
| 8 | 有度 -> G3 Proportionality | L5 / L3 |
| 9 | 损补 -> G4 Trade-off Management | L5 / L3 |
| 10 | 顺势 -> G5 Trend Alignment | L5 / L2 |
| 11 | 鉴九段式 -> 检验流程 | L5 |
| 12 | F/G/J -> validator 严重级 | L3 |
| 13 | stage 生命周期 -> 状态机 | L1 |
| 14 | decided-by -> 人类决策强制 | L2 |

链 6、8、9、10 出现双 L 级别：法论定义的 G1-G5 度量管道为 L5，但 iCT 实现了降格的五法检验（L2-L3）。此为初步判断未覆盖的发现，详见各条说明。

## 道层映射

### 链 1：道一 -> 产出方差度量

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L4 |
| 哲学要求 | 道一可证伪条件：找到无协调约束下产出方差为零的过程 |
| 工程现状 | 度量采集已嵌入，指标计算未实现 |

Phase 0-2 工程补全后，`index_document`（indexer.rs 第 77-94 行）在每次文档验证时采集 `ValidationCompleted` 事件，记录 Fatal/Guideline/Judgment 计数和通过状态，写入 metrics 表（database.rs 第 110-118 行 schema）。`SihDatabase` trait 扩展了 `record_metric`、`query_metrics`、`get_latest_snapshot` 三个方法（database.rs 第 28-42 行）。

但方差计算逻辑（跨文档比较、时间序列聚合）未实现。已有数据点为单次验证的原始计数，缺乏"文档风格不一致率"、"架构漂移率"、"字段差异度"等操作化指标的计算逻辑。L4 而非 L3 的原因：采集与计算之间存在明确的工程缺口，原始数据无法直接反映"产出方差"这一构念。

`ICL::diagnose_divergences`（icl.rs 第 236 行）的启发式诊断仍为发散的症状检测，不是道一定义的产出方差度量。

### 链 2：道二 -> 意图恢复流程

| 项目 | 内容 |
| ---- | ---- |
| 文档生成侧 | L2 |
| 代码修改侧 | L5 |

文档生成侧：`GrillingEngine`（grilling.rs）通过四个元规则追问（道二/顺因/有度/知止）收敛意图，构建结构化提示词，注入 validator 约束。这是意图恢复的近似实现。偏差在于：固定四问模板，非完整意图恢复流程；产出为提示词模板，意图恢复依赖外部 Agent 执行。

代码修改侧：零实现。`ICL::analyze`（icl.rs 第 30 行）分析文档治理定位与关系图谱，不从代码恢复意图。无任何代码到意图的追溯机制。道二声称"意图先于代码"，但工程层仅覆盖文档侧的意图显式化，代码侧的意图恢复完全缺失。

### 链 3：道三 -> 信息损耗检测

| 项目 | 内容 |
| ---- | ---- |
| 规格层 | L1 |
| dao_trace | L4 |

规格层：道论诚实引用 Shannon 1948 作为 external-anchor，锚定声明精确无歧义。道三的工程映射在规格层完整。

dao_trace：`ICT::build_dao_trace`（ict.rs 第 386 行）遍历五法检验结果，为非 Pass 项产出静态字符串（如"道二 + 道三"配描述文本）。`GrillingEngine::build_dao_trace`（grilling.rs 第 331 行）返回固定字符串，与 nature 参数无关（参数名为 `_nature`）。二者均无信息损耗检测逻辑，dao_trace 仅为装饰性标签，与道三的 Shannon 锚定无实质联系。

### 链 4：道四a -> 间隙声明

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L1（规格层） |

道论将道四a声明为 tautology，锚定 Godel 不完备性定理，认识论标签精确。`GrillingEngine::build_sections`（grilling.rs 第 306 行）为所有 nature 注入 `@limitations` 章节提示，但仅为模板注入，无内容校验。规格层的重言式诚实声明构成完整映射，工程层的 @limitations 注入为辅助手段。

### 链 5：道四b -> 跨版本一致性检查

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L4 |
| 哲学要求 | 纵向审计间隙随时间增大趋势，增大速率与规则数正相关 |
| 工程现状 | ProjectSnapshot 采集已嵌入，跨版本比较未实现 |

Phase 0-2 工程补全后，`project_status`（governance.rs 第 271-280 行）在每次调用时采集 `ProjectSnapshot` 事件，记录文档总数、规则总数（硬编码为 **14**）、按 stage 分布、按 nature 分布，写入 metrics 表。`get_latest_snapshot`（database.rs 第 399-402 行）可查询最近一次快照。

但跨版本比较逻辑（同一项目不同时间点的快照 diff）未实现。已有数据点为单个时间点的项目截面，缺乏"跨版本验证结果差异"和"规则数量时序"两个操作化指标的时序对比逻辑。L4 而非 L3 的原因：截面数据采集与纵向比较之间存在明确工程缺口，单次快照无法反映"间隙增长速率"这一构念。

`V-G-08`（validator.rs 第 441 行）的 stage X 废弃标记检查仍为废弃标记检查，非间隙增长度量。

## 法层映射

### 重要发现：iCT 五法检验

iCT（ict.rs）实现了五法检验函数：`check_shunyin`、`check_youdou`、`check_zhizhi`、`check_sunbu`、`check_shunshi`。这是五法在工程层的降格实现，将哲学原则简化为 if-else 规则。但法论定义的 G1-G5 工程验证方法（度量管道）仍为 L5。初步判断将链 6-10 标为 L5，仅反映了度量管道的缺失，未记录 iCT 的降格实现。以下逐条记录两层状态。

### 链 6：知止 -> G1 Scope Boundary

| 项目 | 内容 |
| ---- | ---- |
| G1 度量管道 | L5 |
| iCT zhizhi 检查 | L3 |

G1 验证方法要求度量治理投入（规则数、审查轮次、维护工时）与产出方差的相关性。代码中无此度量管道。

iCT `check_zhizhi`（ict.rs 第 177 行）实现边界检查：不修改非 .sih.md 文件（第 191 行）、不判断哲学对错（第 182 行）、不管理 drafts（第 204 行）。这是知止原则的降格应用，将"治理投入与方差成正比"简化为"不越界操作"的 if-else。`targets_drafts` 为桩函数恒返回 false（ict.rs 第 487 行），drafts 检查实际未生效。

### 链 7：顺因 -> G2 Causal Alignment

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L2 |

iCL `resolve_upstream_chain`（icl.rs 第 66 行）通过 `db.resolve_chain` 追溯上游授权链。`is_legal_chain`（icl.rs 第 201 行）检查 nature 链合法性（spec->proposal->decision 等）。iCT `check_shunyin`（ict.rs 第 38 行）检测逆写（修改上游文档）与越级（1/3 执行 ratify 级操作）。

偏差可量化为三项。`modifies_upstream` 使用字符串包含匹配判断是否修改上游（ict.rs 第 94 行），非语义级分析。`would_create_cycle` 为 MVP 桩函数恒返回 false（ict.rs 第 513 行），环检测未实现。无代码级追溯，仅文档到文档的引用链检查。

### 链 8：有度 -> G3 Proportionality

| 项目 | 内容 |
| ---- | ---- |
| G3 度量管道 | L5 |
| iCT youdou 检查 | L3 |

G3 验证方法要求评估各治理域风险等级、统计规则数量与严格度、检验风险与力度正相关。代码中无规则数审计、无风险等级评估。

iCT `check_youdou`（ict.rs 第 120 行）实现力度匹配：Critical 级发散配 NoAction 判 Fail（第 129 行），Info 级配 Archive 判 Fail（第 139 行）。将"规则数与风险成正比"降格为"action 力度与发散严重度匹配"的 if-else，原语义中的规则数审计与风险等级评估丢失。

### 链 9：损补 -> G4 Trade-off Management

| 项目 | 内容 |
| ---- | ---- |
| G4 文档要求 | L5 |
| iCT sunbu 检查 | L3 |

G4 要求每条决策记录权衡（ADR 三段式：背景->决策->后果），统计规则总量趋势，定期修剪冗余规则。代码中无 ADR 内容强制校验、无规则趋势追踪、无修剪机制。`grilling.rs` 为 decision 注入 ADR 章节模板（第 266 行），但无内容校验。

iCT `check_sunbu`（ict.rs 第 235 行）实现方向检查：高重叠重复配 NoAction 判 Fail（第 255 行，该损不损），Gap 配 Archive 判 Fail（第 263 行，该补却损）。将"损有余补不足"降格为方向反置检测的 if-else，原语义中的权衡记录与规则趋势追踪丢失。

### 链 10：顺势 -> G5 Trend Alignment

| 项目 | 内容 |
| ---- | ---- |
| G5 变更率追踪 | L5 |
| iCT shunshi 检查 | L2 |

G5 要求纵向追踪力度变化、区域力度分化（地势）、认知源数量影响（人势）。代码中无变更率追踪、无区域维度、无认知源维度。

Phase 0-2 工程补全尝试在 iCT `check_shunshi`（ict.rs 第 299-307 行）嵌入变更率度量采集，但因 `verify` 是纯同步关联函数，签名中无 db 参数，无法执行异步数据库操作。仅添加了 TODO 注释描述采集方案（查询 metrics 表中最近的 ValidationCompleted 记录数，构造审查次数/变更次数比值），未实现采集逻辑。变更率采集待后续重构（将 db 注入 verify 或在调用方采集）时补全。L5 维持不变。

iCT `check_shunshi`（ict.rs 第 310 行起）实现时势维度：3/3 禁用暧昧措辞（第 316 行），1/3 禁用强制措辞（第 323 行），root 保护（第 332 行），stage X 保护（第 352 行）。时势维度有近似实现，故评为 L2。但 2/3 恒返回 Conditional（第 364 行）为硬编码，地势与人势维度完全缺失。

## 机制层映射

### 链 11：鉴九段式 -> 检验流程

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L5 |
| 工程现状 | 零实现 |

鉴论第四节明确声明："当前工程层状态：零实现。九段式目前仅存在于哲学文档中，未在确定性引擎或任何工程机制中落地。" 九段式标注为 constructed-framework，处置为暂缓。

iCL/iWW/iCT 三机体系是独立的认知-决策-验证流程，与九段式无关。iCL 文档注释称执行"四步分析法前三步"（icl.rs 第 13 行），但这是三机体系自身的分析方法，非九段式的九段检验规程。三机的 iCT 五法检验（顺因/有度/知止/损补/顺势）与九段式的反推检验（反证/反例/可证伪条件等）是不同的检验框架。

### 链 12：F/G/J -> validator 严重级

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L3 |

`ViolationSeverity` 枚举（models.rs 第 182 行）定义 Fatal/Guideline/Judgment 三级，对应 F/G/J。力度通过枚举显式赋值，rule_id 以 V-F/V-G/V-J 前缀隔离命名空间。E-F 前缀的工程规则在 validator 中无实现，仅 V-F 前缀的验证规则存在。

降格点：J 的语义被反转。旧哲学层定义 J（矩）为"精确判定 pass/fail"的强机械判定，代码实现为"静默记录"（仅计数不阻断）。validator.rs 第 57 行注释与 models.rs 第 174 行注释均明确记录此反转，但哲学层未同步修正。F/G/J 的力度梯度被降格为三值枚举，原语义中的"风险匹配力度"丢失。

### 链 13：stage 生命周期 -> 状态机

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L1 |

`Stage` 枚举（models.rs 第 7 行）定义 Propose/Resolve/Ratify/Deprecated/Superseded 五态。`is_referenceable`（第 88 行）返回 Resolve/Ratify，`is_terminal`（第 92 行）返回 Deprecated/Superseded。`from_str`（第 96 行）解析字符串到枚举。

Phase 0-2 工程补全后，`can_transition_to`（models.rs 第 57-86 行）已实现完整的状态转换矩阵。覆盖全部转换组合：Propose->Resolve/Resolve->Ratify/Propose->Ratify 的正向递进，Resolve->Propose/Ratify->Resolve/Ratify->Propose 的 Reopen 路径，活跃态到 Deprecated/Superseded 的废弃路径，以及 Deprecated/Superseded 的终态锁定（不可转换）。相同 stage 返回 Err（"same stage"）。所有转换规则可机械验证，无歧义。

`is_valid`（models.rs 第 46 行）恒返回 true，此为设计决策而非遗漏：Stage 枚举的 5 种状态都是合法状态，非法字符串已在 `from_str` 中通过返回 None 处理（第 51 行注释）。转换条件的校验已由 `can_transition_to` 承担，`is_valid` 无需重复此职责。

`can_transition_to` 当前仅提供查询能力（models.rs 第 56 行注释），未被 validator 强制调用。`V-F-03`（validator.rs 第 244 行）仍调用 `doc.stage.is_valid()` 检查 stage 合法性，因 `is_valid` 恒 true，此规则实际无校验效果。L1 而非 L2 的原因：映射本身的完整性和精确性已满足 L1 标准（完整的转换矩阵、机械可验证），validator 未调用 `can_transition_to` 是集成层面的问题，不影响映射链本身的 L 级别判定。

### 链 14：decided-by -> 人类决策强制

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L2 |

`V-F-06`（validator.rs 第 497 行）检查 decided-by 不得以 "ai" 开头，`is_ai_prefixed_decided_by`（第 571 行）实现前缀检查（不区分大小写）。此检查已修复，阻断 ai-auto、ai-assist 等所有 AI 前缀值。`V-F-07`（第 514 行）禁止非 decisions/ 文档携带 decided-by。

偏差可量化为两项。`V-G-09`（缺失 decided-by，validator.rs 第 465 行）仅为 Guideline 级警告，非 Fatal 阻断，2/3 以上 decision 缺失 decided-by 不被强制拦截。`validate_governance` 依赖 `file_path` 推断 nature（第 466 行），当 file_path 为 None 时 nature 推断失败，decided-by 检查被跳过。governance.rs 第 166 行 `get_document` 调用 `validate_document` 时传入 None，该路径下 decided-by 检查完全失效。

## 断裂分析

### 问题一：当前最大的断裂点在哪里

最大断裂在道一的产出方差度量（链 1，L4）。道一是五法共通之源，其可证伪条件要求度量产出方差。Phase 0-2 已嵌入 ValidationCompleted 事件采集（每次文档验证记录 F/G/J 计数），但方差计算逻辑（跨文档比较、时间序列聚合）仍未实现。这导致知止（G1）与有度（G3）的验证方法失去度量基础：G1 的"治理投入与方差成正比"无方差数据，G3 的"规则数与风险成正比"无风险度量。五法中两法的验证方法因道一度量缺失而无法执行。

次要断裂在 G1-G5 度量管道（链 6-10，均 L5）与鉴九段式（链 11，L5）。法论定义的五条工程验证方法全部缺失，iCT 的五法检验为降格实现（L2-L3），不构成法论定义的度量管道。链 10（顺势 G5）的变更率采集因 iCT verify 函数签名限制（纯同步、无 db 参数）仅添加了 TODO 注释，待后续重构补全。鉴论自身声明零实现，九段式仅存于哲学文档。

### 问题二：哪些断裂是哲学概念不可工程化导致的

鉴九段式（链 11）。鉴论第三节诚实声明自证循环：检验者与被检验者同源于司衡体系，鉴检验五维天道所得结果被用来证明鉴有效，鉴的有效性又支撑其检验资格。鉴的认识论标签为 constructed-framework，有效性未经外部工程数据验证。在有效性确立前工程化等同于固化未经验证的框架，违背鉴自身"候选标记"约束。此断裂部分源于哲学概念自身的认识论限制。

道四b的间隙精确测量（链 5）。道四b要求度量"治理与实践的间隙"及其增长速率。间隙的精确定义与度量在概念层面存在困难：何为间隙、如何量化语义未覆盖空间、跨何种粒度比较，均无统一定义。此断裂部分源于概念的可度量性限制。

### 问题三：哪些断裂是工程实现能力不足导致的（可修复）

道一产出方差度量（链 1）。可通过定义方差指标（变更分歧率、协调覆盖率）、采集度量数据实现。当前缺失是工程优先级问题，非概念不可行。

G1-G5 度量管道（链 6-10）。知止的投入-方差度量、有度的规则数审计、损补的规则趋势追踪、顺势的变更率追踪，均可通过扩展数据库 schema、增加度量采集点实现。iCT 已有降格实现，度量管道是补全方向。

stage 转换条件（链 13，已修复）。`can_transition_to`（models.rs 第 57-86 行）已实现完整的转换矩阵，L2 升至 L1。但 `V-F-03`（validator.rs 第 244 行）仍调用 `is_valid`（恒 true）而非 `can_transition_to`，此集成遗漏不影响映射级别判定。

decided-by 路径覆盖（链 14）。`get_document` 传入 None 导致检查跳过，可通过传入文件路径或在 Document 结构中持久化 nature 字段修复。

顺因环检测（链 7）。`would_create_cycle` 为桩函数，可通过调用 `db.resolve_chain` 实现真实环检测。

### 问题四：哪些断裂是映射方式错误导致的（可修复）

F/G/J 的 J 语义反转（链 12）。代码将 J 从"精确判定 pass/fail"反转为"静默记录"，validator.rs 与 models.rs 注释记录了此反转但未同步哲学层。修复方式：要么修正哲学层 J 的定义以匹配代码语义，要么修正代码以匹配哲学层定义。当前状态是映射方式不一致，代码注释已承认但未解决。

dao_trace 装饰性映射（链 3）。`build_dao_trace` 声称追溯道/法原则，实际产出固定字符串或模板文本，与信息损耗检测无关。修复方式：要么实现真实的道追溯逻辑（将检验结果映射到具体道层主张与锚定），要么移除 dao_trace 字段及其追溯声称。当前状态是映射名实不符。

道二代码侧映射缺失（链 2）。工程映射假设意图恢复覆盖文档与代码两侧，实际仅文档生成侧有实现。修复方式：明确声明代码侧意图恢复为未实现，或在 iCL 中增加代码分析能力。当前状态是映射范围虚报。

## 附录

### 代码引用索引

- validator.rs：V-F-01 至 V-F-07、V-G-02 至 V-G-09、V-J-01 验证规则
- models.rs：Stage 枚举（第 7 行）、`can_transition_to`（第 57 行）、ViolationSeverity 枚举（第 182 行）、Document 结构（第 128 行）
- database.rs：`SihDatabase` trait（第 11 行）、metrics 表 schema（第 110 行）、`record_metric`/`query_metrics`/`get_latest_snapshot`
- metrics.rs：`MetricEvent` 枚举定义
- indexer.rs：`index_document`（第 50 行）、`ValidationCompleted` 采集（第 77 行）、`IndexCompleted` 采集（第 104 行）
- ict.rs：ICT 五法检验、`check_shunshi` 变更率 TODO（第 299 行）、build_dao_trace（第 396 行）
- iww.rs：IWW 决策建议生成
- icl.rs：ICL 认知分析（第 30 行）、上游链追溯（第 66 行）、发散诊断（第 236 行）
- grilling.rs：GrillingEngine 追问引擎、build_dao_trace（第 331 行）
- governance.rs：MCP 服务、`project_status`（第 222 行）、`ProjectSnapshot` 采集（第 271 行）、full_analysis 三机流转

### DEPS

- 260627-1100-dao-on-natural-convergence
  - 道论，四道定义与可证伪条件
- 260628-1000-fa-on-governance-principles
  - 法论，五法与 G1-G5 工程验证方法
- 260627-1100-jian-on-verification
  - 鉴论，九段式处置与零实现声明
