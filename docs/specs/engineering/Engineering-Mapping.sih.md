---
id: 260628-1100-engineering-mapping
stage: 1/3
upstream: 260627-1100-dao-on-natural-convergence
---
# 司衡工程映射

> 本文档逐条追踪司衡哲学概念到工程实现的映射状态。基于 V3 R4 审计发现的 L3-L5 断裂，诚实记录每条映射链的当前状态、代码证据与偏差。所有 L 级别基于实际源码验证，与初步判断有差异处已标注说明。Phase 0-2 工程补全后，链 1/5 从 L4 升级到 L2，链 13 从 L2 升级到 L1。L5 度量管道补全后，链 6/8/9/10 从 L5 升级到 L2。行迹机制与 DDD 候选术授权后，新增链 15/16，均为 L5（零实现）。

## 映射分级定义

**L1 完整映射**：哲学概念有精确的工程对应，可机械验证，无歧义。

**L2 近似映射**：工程对应存在但与哲学原意有偏差，偏差可量化。

**L3 降格映射**：哲学概念在工程层被简化为字符串、枚举或 if-else，原语义丢失。

**L4 装饰性映射**：工程实现与哲学概念无实质联系，仅为标签挂载。

**L5 无映射**：哲学概念在工程层无任何对应。

## 映射链总览

以下为 16 条映射链的 L 级别汇总。道层 5 条，法层 5 条，机制层 6 条。

| 编号 | 映射链 | L 级别 |
| ---- | ---- | ---- |
| 1 | 道一 -> 产出方差度量 | L2 |
| 2 | 道二 -> 意图恢复流程 | L2 |
| 3 | 道三 -> 信息损耗检测 | L1 / L4 |
| 4 | 道四a -> 间隙声明 | L1 |
| 5 | 道四b -> 跨版本一致性检查 | L2 |
| 6 | 知止 -> G1 Scope Boundary | L2 / L3 |
| 7 | 顺因 -> G2 Causal Alignment | L2 |
| 8 | 有度 -> G3 Proportionality | L2 / L3 |
| 9 | 损补 -> G4 Trade-off Management | L2 / L3 |
| 10 | 顺势 -> G5 Trend Alignment | L2 / L2 |
| 11 | 鉴九段式 -> 检验流程 | L5 |
| 12 | F/G/J -> validator 严重级 | L3 |
| 13 | stage 生命周期 -> 状态机 | L1 |
| 14 | decided-by -> 人类决策强制 | L2 |
| 15 | 道二/道三 -> 行迹记录 | L5 |
| 16 | 道二/道三 -> DDD 战术设计 | L5 |

链 6、8、9、10 出现双 L 级别：法论定义的 G1-G5 度量管道为 L5，但 iCT 实现了降格的五法检验（L2-L3）。此为初步判断未覆盖的发现，详见各条说明。

## 道层映射

### 链 1：道一 -> 产出方差度量

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L2 |
| 哲学要求 | 道一可证伪条件：找到无协调约束下产出方差为零的过程 |
| 工程现状 | 指标计算已实现，MCP 工具已暴露查询 |

Phase 0-2 工程补全后，`index_document`（indexer.rs 第 77-94 行）在每次文档验证时采集 `ValidationCompleted` 事件，记录 Fatal/Guideline/Judgment 计数和通过状态，写入 metrics 表（database.rs 第 110-118 行 schema）。`SihDatabase` trait 扩展了 `record_metric`、`query_metrics`、`get_latest_snapshot` 三个方法（database.rs 第 28-42 行）。

指标计算补全后，`compute_variance_metric`（metrics.rs 第 85 行）从 `ValidationCompleted` 历史记录聚合计算产出方差指标。计算内容包括：通过率（passed 文档占比）、平均 Fatal 违规数、平均 Guideline 违规数、Fatal 违规数的总体标准差（`fatal_count_stddev`，作为产出方差的直接度量）、按 nature 分组的通过率与平均 Fatal 违规数。结果以 `VarianceMetric` 结构（metrics.rs 第 47 行）返回，统计窗口取记录的最早与最晚 `created_at`。

MCP 工具 `variance_metric`（governance.rs 第 707 行）已暴露查询能力。该工具查询最近 **100** 条 `ValidationCompleted` 记录，调用 `compute_variance_metric` 计算指标，格式化为人类可读文本报告返回。

剩余限制：标准差是产出方差的近似度量，仅覆盖验证违规维度，未覆盖架构漂移和字段差异维度。效度威胁见操作化规格。L2 而非 L1 的原因：工程对应存在但与哲学原意有偏差，偏差可量化为度量维度的覆盖缺口。

`ICL::diagnose_divergences`（icl.rs 第 236 行）的启发式诊断仍为发散的症状检测，不是道一定义的产出方差度量。

### 链 2：道二 -> 文档侧意图恢复

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L2 |
| 哲学要求 | 文档生成前意图必须显式化 |

文档生成侧：`GrillingEngine`（grilling.rs）通过四个元规则追问（道二/顺因/有度/知止）收敛意图，构建结构化提示词，注入 validator 约束。这是意图恢复的近似实现。偏差在于：固定四问模板，非完整意图恢复流程；产出为提示词模板，意图恢复依赖外部 Agent 执行。

代码侧意图恢复不在本映射链中。代码侧意图恢复的唯一归位目标是链 15（行迹机制，道三）。iCL 设计原则为只读文档不读代码——代码侧意图恢复不在 iCL 职能范围内，此乃知止的正确执行。

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
| L 级别 | L2 |
| 哲学要求 | 纵向审计间隙随时间增大趋势，增大速率与规则数正相关 |
| 工程现状 | 跨版本快照差异计算已实现，MCP 工具已暴露查询 |

Phase 0-2 工程补全后，`project_status`（governance.rs 第 271-280 行）在每次调用时采集 `ProjectSnapshot` 事件，记录文档总数、规则总数（取自 `RULE_COUNT` 常量，已替代硬编码 **14**）、按 stage 分布、按 nature 分布，写入 metrics 表。`get_latest_snapshot`（database.rs 第 399-402 行）可查询最近一次快照。

指标计算补全后，`compute_snapshot_diff`（metrics.rs 第 235 行）比较两次 `ProjectSnapshot` 的差异。计算内容包括：文档数变化（`docs_delta`）、规则数变化（`rules_delta`）、各 stage 文档数变化（`docs_by_stage_delta`）、各 nature 文档数变化（`docs_by_nature_delta`）、Fatal 违规总数变化（`fatal_violations_delta`），并产出两个间隙信号：`rules_grew`（规则数是否增长）与 `docs_grew`（文档数是否增长）。差异结果以 `SnapshotDiff` 结构（metrics.rs 第 178 行）返回。`compute_latest_snapshot_diff`（metrics.rs 第 268 行）提供便捷查询，从数据库取最近两次快照并计算差异，不足两次时返回 `None`。

MCP 工具 `snapshot_diff`（governance.rs 第 720 行）已暴露查询能力。该工具调用 `compute_latest_snapshot_diff`，不足两次快照时返回提示文本，否则格式化为人类可读的差异报告。

剩余限制：仅比较相邻两次快照，未做长期趋势分析；规则修正噪声未过滤（操作化规格中的效度威胁）。L2 而非 L1 的原因：工程对应存在但与哲学原意有偏差，偏差可量化为纵向分析的覆盖缺口与噪声未过滤。

`V-G-08`（validator.rs 第 441 行）的 stage X 废弃标记检查仍为废弃标记检查，非间隙增长度量。

## 法层映射

### 重要发现：iCT 五法检验

iCT（ict.rs）实现了五法检验函数：`check_shunyin`、`check_youdou`、`check_zhizhi`、`check_sunbu`、`check_shunshi`。这是五法在工程层的降格实现，将哲学原则简化为 if-else 规则。但法论定义的 G1-G5 工程验证方法（度量管道）仍为 L5。初步判断将链 6-10 标为 L5，仅反映了度量管道的缺失，未记录 iCT 的降格实现。以下逐条记录两层状态。

### 链 6：知止 -> G1 Scope Boundary

| 项目 | 内容 |
| ---- | ---- |
| G1 度量管道 | L2 |
| iCT zhizhi 检查 | L3 |

G1 度量管道已实现：`compute_rule_density`（metrics.rs）计算各 nature 的规则密度（total_rules / nature_docs），消费 `count_by_nature` 和最近 100 条 `ValidationCompleted` 记录。产出方差数据取自链 1 的 `avg_fatal_by_nature`，两者并列展示为相关性检验积累数据。当前规则数不足以做统计显著性检验（仅 6 个 nature），`correlation_note` 诚实声明样本不足。

MCP 工具 `rule_density`（governance.rs）已暴露查询能力，格式化为人类可读文本报告返回。

偏差可量化为两项：(1) 规则密度仅反映投入维度，相关性分析需依赖链 1 数据，引入传递误差；(2) 规则当前不按 nature 分配，所有 nature 共享同一规则池，`density_by_nature` 的差异仅反映文档分布。L2 而非 L1 的原因见操作化规格。

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
| G3 度量管道 | L2 |
| iCT youdou 检查 | L3 |

G3 度量管道已实现：`compute_rule_audit`（metrics.rs）从 `RULE_REGISTRY` 聚合规则数分布（按治理域：Frontmatter/Structure/Reference/Governance，按严格度：F/G/J）和 Fatal 级规则占比。纯函数，不查数据库，无外部依赖。

MCP 工具 `rule_audit`（governance.rs）已暴露查询能力，格式化为人类可读文本报告返回。

偏差可量化为两项：(1) 规则审计仅反映定义侧严格度，不反映执行侧触发率；(2) 各治理域风险等级的评估依赖间接推断，未独立操作化。L2 而非 L1 的原因见操作化规格。

iCT `check_youdou`（ict.rs 第 120 行）实现力度匹配：Critical 级发散配 NoAction 判 Fail（第 129 行），Info 级配 Archive 判 Fail（第 139 行）。将"规则数与风险成正比"降格为"action 力度与发散严重度匹配"的 if-else，原语义中的规则数审计与风险等级评估丢失。

### 链 9：损补 -> G4 Trade-off Management

| 项目 | 内容 |
| ---- | ---- |
| G4 文档要求 | L2 |
| iCT sunbu 检查 | L3 |

G4 度量管道已实现：`compute_tradeoff_coverage`（metrics.rs）扫描全量文档中 nature 为 decision 的文档内容，检测 `## 背景`/`## 决策`/`## 后果` 三个 Markdown 二级标题下方是否有非空内容，计算 ADR 覆盖率。纯函数不查数据库，输入为已加载的 Document 列表。

MCP 工具 `tradeoff_coverage`（governance.rs）已暴露查询能力，调用 `get_all_documents` 加载全量文档后计算覆盖率。

`rule_changes_note` 诚实声明规则增删比率需累积 ProjectSnapshot 历史数据，当前不可计算。偏差可量化为：ADR 覆盖率仅度量"是否记录"而不度量"记录质量"。L2 而非 L1 的原因见操作化规格。

iCT `check_sunbu`（ict.rs 第 235 行）实现方向检查：高重叠重复配 NoAction 判 Fail（第 255 行，该损不损），Gap 配 Archive 判 Fail（第 263 行，该补却损）。将"损有余补不足"降格为方向反置检测的 if-else，原语义中的权衡记录与规则趋势追踪丢失。

### 链 10：顺势 -> G5 Trend Alignment

| 项目 | 内容 |
| ---- | ---- |
| G5 变更率追踪 | L2 |
| iCT shunshi 检查 | L2 |

G5 度量管道已实现：`compute_trend_alignment`（metrics.rs）从 `ValidationCompleted` 和 `IndexCompleted` 记录计算审查频率-变更频率比值（`review_change_ratio`）。比值接近 1 表示审查与变更同步，远小于 1 表示审查滞后。统计窗口取两类记录 created_at 的最小/最大值。

MCP 工具 `trend_alignment`（governance.rs）已暴露查询能力，各查询 100 条记录后计算指标。度量计算与 iCT `check_shunshi` 完全解耦，不嵌入 `verify` 方法。

偏差可量化为三项：(1) 仅覆盖时势维度，地势与人势维度未操作化；(2) 未区分"响应性审查"与"例行审查"；(3) 变更统计粒度为文档级索引事件。L2 而非 L1 的原因见操作化规格。iCT 时势维度维持 L2（3/3 禁用暧昧措辞、1/3 禁用强制措辞、root 保护、stage X 保护）。

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

### 链 15：道二/道三 -> 行迹记录

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L5 |
| 哲学要求 | 意图形成过程可被恢复（道三：代码自晦，意图必复） |
| 工程现状 | 零实现 |

行迹机制是道三"代码自晦，意图必复"和道二"意图先于代码"在机制层的直接展开。spec-coding 记录意图的产出形态（规范），行迹记录意图的形成过程（方向性转折及其理由）。行迹同时承接链 2 代码侧意图恢复的归位：不是从代码反推意图，而是从行迹记录恢复意图。

当前零实现。行迹机制的写入由治理层（MCP 工具）承担，读取由 iCL 在意图定位中纳入。iCL 的扩展不违反其设计原则：仍然只读不写，信息来源从文档扩展到文档加行迹。知止边界：只记录方向性转折，不记录探索过程细节。行迹准入不是机械判断，由 LLM 辅助层评估准入条件（方向性转折、可重复遭遇、非已被覆盖），确定性引擎机械裁决排除条件（ADR/应论已覆盖），人类治理者最终决定。数据成熟度为待验证，待跨 agent 实践积累后再决定完整工程化。行迹以 .sih.md 文档形式存在于 knowledge/trails/ 目录，对开发者、外部审阅者、社区贡献者披露。

### 链 16：道二/道三 -> DDD 战术设计

| 项目 | 内容 |
| ---- | ---- |
| L 级别 | L5 |
| 哲学要求 | 代码结构侧的意图显式化 |
| 工程现状 | 零实现，待九段式检验 |

DDD 的核心洞察已融入五法（法层观察来源）：限界上下文对应顺因（意图边界先于实现边界），聚合根对应知止（治理力度应有边界）。DDD 的战术设计（聚合、实体、值对象、领域事件、仓储等模式）是 spec-coding 之外的候选术：spec-coding 从文档侧显式化意图，DDD 从代码结构侧显式化意图。

当前零实现且待九段式检验。不立即纳入术层的理由：顺因（需先论证 DDD 与五法的映射关系）、鉴的暂缓（DDD 尚未经九段式检验）、知止（spec-coding 尚未完全落地）。待九段式完成暂缓处置、DDD 通过检验、spec-coding 的 L 级别提升到 L2 以上后再考虑纳入。

## 断裂分析

### 问题一：当前最大的断裂点在哪里

最大断裂在 G1-G5 度量管道（链 6-10，均 L5）与鉴九段式（链 11，L5）。法论定义的五条工程验证方法全部缺失，iCT 的五法检验为降格实现（L2-L3），不构成法论定义的度量管道。链 10（顺势 G5）的变更率采集因 iCT verify 函数签名限制（纯同步、无 db 参数）仅添加了 TODO 注释，待后续重构补全。鉴论自身声明零实现，九段式仅存于哲学文档。

道一的产出方差度量（链 1）已从 L4 升至 L2。`compute_variance_metric` 已从 ValidationCompleted 历史记录聚合计算通过率、Fatal 违规标准差与按 nature 分组统计，MCP 工具 `variance_metric` 已暴露查询能力。但标准差仅覆盖验证违规维度，未覆盖架构漂移和字段差异维度，知止（G1）与有度（G3）仍缺乏完整的度量基础：G1 的"治理投入与方差成正比"仅有验证违规维度的方差数据，G3 的"规则数与风险成正比"无风险度量。道四b的跨版本一致性检查（链 5）已从 L4 升至 L2，`compute_snapshot_diff` 已实现相邻快照差异计算，但仅比较相邻两次快照，未做长期趋势分析。

### 问题二：哪些断裂是哲学概念不可工程化导致的

鉴九段式（链 11）。鉴论第三节诚实声明自证循环：检验者与被检验者同源于司衡体系，鉴检验五维天道所得结果被用来证明鉴有效，鉴的有效性又支撑其检验资格。鉴的认识论标签为 constructed-framework，有效性未经外部工程数据验证。在有效性确立前工程化等同于固化未经验证的框架，违背鉴自身"候选标记"约束。此断裂部分源于哲学概念自身的认识论限制。

道四b的间隙精确测量（链 5）。道四b要求度量"治理与实践的间隙"及其增长速率。链 5 已实现相邻快照差异计算（`compute_snapshot_diff`），但间隙的精确定义与度量在概念层面仍存在困难：何为间隙、如何量化语义未覆盖空间、跨何种粒度比较，均无统一定义。当前快照差异仅覆盖文档数、规则数、stage/nature 分布、Fatal 违规等可计数维度，语义未覆盖空间的量化仍无定义。此断裂部分源于概念的可度量性限制。

### 问题三：哪些断裂是工程实现能力不足导致的（可修复）

道一产出方差度量（链 1）。`compute_variance_metric` 已实现验证违规维度的方差度量（通过率、Fatal 违规标准差、按 nature 分组统计）。可进一步扩展的方向：架构漂移维度与字段差异维度的度量，以及变更分歧率、协调覆盖率等补充指标。当前仅覆盖验证违规维度，扩展为工程优先级问题，非概念不可行。

G1-G5 度量管道（链 6-10）。知止的投入-方差度量、有度的规则数审计、损补的规则趋势追踪、顺势的变更率追踪，均可通过扩展数据库 schema、增加度量采集点实现。iCT 已有降格实现，度量管道是补全方向。

stage 转换条件（链 13，已修复）。`can_transition_to`（models.rs 第 57-86 行）已实现完整的转换矩阵，L2 升至 L1。但 `V-F-03`（validator.rs 第 244 行）仍调用 `is_valid`（恒 true）而非 `can_transition_to`，此集成遗漏不影响映射级别判定。

decided-by 路径覆盖（链 14）。`get_document` 传入 None 导致检查跳过，可通过传入文件路径或在 Document 结构中持久化 nature 字段修复。

顺因环检测（链 7）。`would_create_cycle` 为桩函数，可通过调用 `db.resolve_chain` 实现真实环检测。

### 问题四：哪些断裂是映射方式错误导致的（可修复）

F/G/J 的 J 经法论校准（链 12）。法论 §有度 J 原定义为"精确判定 pass/fail"，混淆了判定方式与阻断行为两个独立维度。经 260628-1600-mapping-fix-trio 修复：法论力度表拆为四列（力度/判定方式/阻断行为/适用场景），J = 精确机械判定 + 静默记录，校准标准来自 G3，校准程序来自 G2。代码注释从"以代码语义为准"改为"经 G3 校准"。已修正。

dao_trace 已降级为 law_violation_summary（链 3）。原 `build_dao_trace` 声称追溯道/法原则，实际产出硬编码字符串。经 260628-1600-mapping-fix-trio 修复：重命名为 `LawViolationSummary`，职责收缩为描述法检结果不做道层归因，道层归因由工程映射表承担。GrillingEngine 的 `build_dao_trace`（返回固定字符串，`_nature` 忽略）已删除。L4→L3，诚实化降级。

道二代码侧映射对象错误（链 2）。代码侧意图恢复原映射到 iCL，但 iCL 的职能是文档认知分析，不是代码意图恢复。三机的设计原则是只读不写、只分析文档不分析代码。修复方式：代码侧意图恢复归位到链 15（行迹记录），链 2 只覆盖文档侧。当前状态是映射对象错误已修正，链 2 从 L2/L5 变为 L2。

## 附录

### 代码引用索引

- validator.rs：V-F-01 至 V-F-07、V-G-02 至 V-G-09、V-J-01 验证规则
- models.rs：Stage 枚举（第 7 行）、`can_transition_to`（第 57 行）、ViolationSeverity 枚举（第 182 行）、Document 结构（第 128 行）
- database.rs：`SihDatabase` trait（第 11 行）、metrics 表 schema（第 110 行）、`record_metric`/`query_metrics`/`get_latest_snapshot`
- metrics.rs：`MetricEvent` 枚举定义、`VarianceMetric` 结构（第 47 行）、`compute_variance_metric`（第 85 行）、`SnapshotDiff` 结构（第 178 行）、`compute_snapshot_diff`（第 235 行）、`compute_latest_snapshot_diff`（第 268 行）
- indexer.rs：`index_document`（第 50 行）、`ValidationCompleted` 采集（第 77 行）、`IndexCompleted` 采集（第 104 行）
- ict.rs：ICT 五法检验、`check_shunshi` 变更率 TODO（第 299 行）、build_law_violation_summary（第 396 行）
- iww.rs：IWW 决策建议生成
- icl.rs：ICL 认知分析（第 30 行）、上游链追溯（第 66 行）、发散诊断（第 236 行）
- grilling.rs：GrillingEngine 追问引擎
- governance.rs：MCP 服务、`project_status`（第 222 行）、`ProjectSnapshot` 采集（第 271 行）、`variance_metric` 工具（第 707 行）、`snapshot_diff` 工具（第 720 行）、full_analysis 三机流转

### DEPS

- 260627-1100-dao-on-natural-convergence
  - 道论，四道定义与可证伪条件
- 260628-1000-fa-on-governance-principles
  - 法论，五法与 G1-G5 工程验证方法
- 260627-1100-jian-on-verification
  - 鉴论，九段式处置与零实现声明
