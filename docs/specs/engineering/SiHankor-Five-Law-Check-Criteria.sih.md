---
id: 260616-2000-five-law-check-criteria
stage: 2/3
upstream: 260613-1650-sihankor-mind-design
---

# 五法检验评判标准

> iCT 方圆机执行五法检验的 pass/fail/conditional 评判标准定义。上游：Mind-Design。

## 一、总则

iCT 方圆机接收 iCL 的 `cognition` 和 iWW 的 `decision_proposal`，对五法（顺因/有度/知止/损补/顺势）执行逐条检验。每条法输出 `LawCheck { law, result: Pass|Fail|Conditional, note }`。

五法检验的**检验对象**是 decision_proposal 中的 recommended_action（以及 alternatives），不是文档本身（文档本身的合规性由 validator 的 13 规则覆盖）。

**整体判据 `overall`**：

- 任意一条法返回 Fail → overall = Fail
- 无 Fail 且任意一条法返回 Conditional → overall = Conditional
- 全部 Pass → overall = Pass

## 二、顺因（ShunYin）

> Canon §2.1：意图先于规范，规范先于实现。任何逆因果方向的操作都是违道。

| 属性 | 内容 |
|------|------|
| 检验对象 | action 是否尊重治理链因果方向 |
| 输入 | cognition.governance_position（nature/stage/upstream_chain/role） + decision_proposal.recommended_action |

### Pass

以下全部满足：

1. **不逆写上游**：如果 action 涉及修改文档，目标不是当前文档的上游文档（upstream_chain 中的文档不可被当前文档的 action 直接修改）。Reopen 级修正（修改上游 Canon 等法层文档）只有在 action.kind = HumanReview 时才合法——因为人类才能决定是否 Reopen。
2. **不跳过治理阶段**：action 不会将 stage 1/3 的文档直接提升到 3/3（跳过 2/3 decision 阶段）。action 不将对 proposal 文档执行 ratify 级操作。
3. **引用方向一致**：如果 action 涉及修改 upstream 引用，新 upstream 是合法的上级（nature 链合法）。不出现 spec 引用 proposal 作为上游（下游 nature 应在链的下游侧）。

### Fail

任一条满足：

1. **逆写上游**：action 提议直接修改 upstream_chain 中的文档（非 HumanReview 模式下修改法层文档）。
2. **越级操作**：action 对 stage 1/3 文档执行 ratify 级操作，或对 proposal nature 文档执行 spec 级验证要求。
3. **逆向引用**：action 提议将更下游 nature 的文档设为 upstream（如 spec 的上游设为 proposal）。

### Conditional

任一条满足：

1. **HumanReview 标记**：action.kind = HumanReview 且触及上游文档修改——HumanReview 模式下允许突破常规，但 iCT 无法判定人类决定是否合道，标记为 Conditional。
2. **治理链边界**：文档 nature 链合法性判断中，涉及 `note`→`proposal` 等灰色区域时（note 的因果角色尚未完全定义），标记为 Conditional。

### 检验逻辑

```text
check_shunyin(cognition, action) -> LawCheck:
    // R1: 逆写检查
    if action.modifies_upstream(cognition.upstream_chain) and action.kind != HumanReview:
        return Fail("尝试逆写上游文档")
    
    // R2: 越级检查
    if cognition.stage == "1/3" and action.targets_ratify_level():
        return Fail("对 1/3 文档执行 ratify 级操作")
    if cognition.nature == "proposal" and action.targets_spec_verification():
        return Fail("对 proposal 文档执行 spec 级验证")
    
    // R3: 引用方向检查
    if action.changes_upstream and not is_legal_nature_chain(action.new_upstream_nature, cognition.nature):
        return Fail("上游 nature 链不合法")
    
    // 边界
    if action.kind == HumanReview and action.modifies_upstream(cognition.upstream_chain):
        return Conditional("HumanReview 模式下的上游修改需人工判断")
    
    return Pass
```

## 三、有度（YouDu）

> Canon §2.2：规约不多不少。过度规约 = 刻意有为 = 违道；不足规约 = 放任发散 = 无收。

| 属性 | 内容 |
|------|------|
| 检验对象 | action 力度与发散严重度的匹配性 |
| 输入 | cognition.divergence_diagnosis（severity/type） + decision_proposal.recommended_action |

### Pass

以下全部满足：

1. **力度匹配严重度**：Critical 级发散 → action 力度 ≥ HumanReview（不允许 NoAction 应对 Critical）；Warning 级发散 → action 力度 ≥ Merge/HumanReview（不允许 NoAction 应对 Warning 级重复）；Info/Benign → NoAction 合法。
2. **不过度反应**：单个 Info 级发散不触发 Archive 或 Canon Reopen 级 action。多个 Warning 不会自动升级为"全部归档"。
3. **action.alternatives 的多样性**：至少有 1 个替代方案的力度与推荐方案不同（展示有度的不同选项）。

### Fail

任一条满足：

1. **力度不足**：Critical 级发散（IntentDrift/ReferenceBreak）推荐 NoAction。
2. **力度过度**：Info 级发散推荐 Archive 或修改法层文档（Canon Reopen）。

### Conditional

任一条满足：

1. **边界模糊**：发散 severity 为 Warning 但 confidence < 0.7——可能 iCL 诊断不准确，过度反应的风险存在。
2. **多重发散混合**：同时存在 Critical 和 Info 发散，推荐 action 可能对某些发散过度、对某些发散不足。

### 检验逻辑

```text
check_youdou(cognition, action) -> LawCheck:
    let max_severity = cognition.divergence_diagnosis.max_severity()
    let has_low_confidence = cognition.divergence_diagnosis.any(|d| d.confidence < 0.7)
    
    // R1: 力度不足
    if max_severity == Critical and action.kind == NoAction:
        return Fail("Critical 级发散不应忽视")
    
    // R2: 力度过度
    if max_severity == Info and action.kind in [Archive, CanonReopen]:
        return Fail("Info 级发散不应触发归档级操作")
    
    // 边界
    if max_severity == Warning and has_low_confidence:
        return Conditional("低 confidence Warning，力度判定存在不确定性")
    if has_mixed_severity():
        return Conditional("混合严重度，单一 action 可能不太精确")
    
    return Pass
```

## 四、知止（ZhiZhi）

> Canon §2.3：不是所有东西都需要规约，不是所有规约都需要 ratify，不是所有问题都能通过治理解决。

| 属性 | 内容 |
|------|------|
| 检验对象 | action 是否逾越 Mind 和治理的边界 |
| 输入 | cognition（governance_position/relation_graph） + decision_proposal |

### Pass

以下全部满足：

1. **不判断哲学对错**：action 的描述中不含"这个哲学定义是错误的"等对哲学文档的否定性判断。对 `docs/specs/philosophy/` 下的文档，action 只能为 HumanReview 或其他间接操作。
2. **不修改非 .sih.md 文件**：affected_documents 中的路径均为 .sih.md 或 .sih/ 内的治约文件。
3. **不对 archive/ 文档提修改建议**（顺势也会检查此项，知止侧重"不该管的不要管"）。
4. **drafts/ 文档不作为治理链节点**：不推荐将 knowledge/drafts/ 中的非 .sih.md 文件纳入 upstream 链。

### Fail

任一条满足：

1. **越权判断**：action 对法层文档（Canon）做出明确否定判断或直接修改建议（非 HumanReview 模式）。
2. **修改非治约文件**：affected_documents 中包含 .rs / .md（非 .sih.md）等非治约文件——Mind 不可修改代码。

### Conditional

1. **间接影响边界模糊**：affected_documents.indirect 中包含非治约文件——这些文件可能受治理变更影响但不应被 Mind 直接修改。标记 Conditional 要求人类确认影响范围。

### 检验逻辑

```text
check_zhizhi(cognition, action) -> LawCheck:
    // R1: 不判断哲学对错
    if action.description.contains_philosophical_negation() and target_is_philosophy_doc():
        return Fail("不可对哲学文档做出否定性判断")
    
    // R2: 不修改非 .sih.md 文件
    if action.affected_documents.direct.any(|p| !p.ends_with(".sih.md") and !p.starts_with(".sih/")):
        return Fail("不可直接修改非治约文件")
    
    // R3: 不管理非治约文档
    if action.targets_drafts_dir():
        return Fail("drafts/ 中的文档不可纳入治理链")
    
    // 边界
    if action.affected_documents.indirect.any(|p| is_non_governance_file(p)):
        return Conditional("间接影响包含非治约文件")
    
    return Pass
```

## 五、损补（SunBu）

> Canon §2.4：去冗余、减发散、填空白、补缺失。不是随机增删，而是有方向的调节。

| 属性 | 内容 |
|------|------|
| 检验对象 | action 的 损/补 方向是否正确 |
| 输入 | cognition.divergence_diagnosis + cognition.relation_graph + decision_proposal |

### Pass

全部满足：

1. **损的方向正确**：对 Duplication 发散推荐 Merge 或 Archive——减少冗余。对已过时的文档推荐 Archive——减少维护负担。不推荐"保留所有重复"（NoAction for High/Exact duplication）。
2. **补的方向正确**：对 ReferenceBreak/Gap 发散推荐补充缺失文档（或标记 HumanReview 用于创建新文档）——填补空白。不推荐对 Gap 做 Archive。
3. **损补不同时作用于同一对象**：不对同一个文档既建议损（归档）又建议补（扩展）。

### Fail

任一条满足：

1. **方向反置**：对 Duplication（overlap=High/Exact）推荐 NoAction（该损不损）。对 Gap 推荐 Archive（该补却损——空白没填反而删了引用源）。
2. **损补同时冲突**：对同一文档同时建议 merge（补）和 archive（损），且未说明先后顺序。

### Conditional

1. **overlap=Partial 的重复**：诊断为 Partial 重复时，推荐 merge 可能过度（多视角讨论被消灭），推荐 NoAction 可能不足（分散维护精力）。标记 Conditional。
2. **BenignDivergence**：良性发散本身不需要损，但伴随 Critical 发散时的综合性决策可能模糊。

### 检验逻辑

```text
check_sunbu(cognition, action) -> LawCheck:
    let div_types = cognition.divergence_diagnosis.types()
    let high_dups = cognition.relation_graph.duplicates.filter(|d| d.overlap in [Exact, High])
    
    // R1: 方向反置检查
    if not high_dups.is_empty() and action.kind == NoAction:
        return Fail("存在高重叠重复但未建议合并/归档")
    
    if div_types.contains(Gap) and action.kind == Archive:
        return Fail("存在 Gap 不应建议归档")
    
    // R2: 损补冲突
    if action.has_conflicting_merge_archive():
        return Fail("对同一文档同时建议 merge 和 archive")
    
    // 边界
    if div_types.contains(Duplication) and only_partial_overlap():
        return Conditional("Partial 重叠的重复，merge/no_action 选择不确定")
    
    return Pass
```

## 六、顺势（ShunShi）

> Canon §2.5：不该收敛时收敛 = 拔苗助长；该收敛时不收敛 = 错失时机。

| 属性 | 内容 |
|------|------|
| 检验对象 | action 力度、措辞是否匹配文档 stage |
| 输入 | cognition.governance_position（stage/nature） + decision_proposal |

### Pass

全部满足：

1. **措辞匹配 stage**：propose 阶段（stage 1/3）的文档，action 描述用"可能"措辞。ratify 阶段（stage 3/3）的文档，action 描述用"应"措辞。不对 archive/（stage X）文档生成修改建议（= NoAction 或 HumanReview 仅用于迁移）。
2. **力度匹配职责**：root 文档（ChainRole::Root）不作为 merge 到其他文档的目标（root 是权威源，不能被合并）。auth 文档的修改建议力度不低于 HumanReview（因为下游多，影响广）。
3. **不创建死循环**：action 不会导致文档的 upstream 形成环（通过 resolve_chain 预检）。

### Fail

任一条满足：

1. **措辞跨阶段**：对 stage 3/3 文档使用"可能建议"等弱措辞（该严不严）。对 stage 1/3 文档使用"必须"等强措辞（拔苗助长）。
2. **root 文档被合并**：推荐将 root 文档 merge 到其他文档（root 应保持独立权威）。
3. **形成引用环**：action 变更 upstream 引用后，形成 `A → B → A` 的循环依赖。

### Conditional

1. **transitional stage**：文档 stage 为 2/3（resolve）时，措辞力度介于"可能"和"应"之间——标记 Conditional。
2. **新文档**：action 建议创建新文档时（尚无 stage），无法判断顺势匹配度。

### 检验逻辑

```text
check_shunshi(cognition, action) -> LawCheck:
    // R1: 措辞匹配
    if cognition.stage == "3/3" and action.description.contains_weak_hedging():
        return Fail("ratify 文档应使用确定性措辞")
    if cognition.stage == "1/3" and action.description.contains_mandatory_language():
        return Fail("propose 文档不应使用强制性措辞")
    
    // R2: root 保护
    if cognition.role == Root and action.kind == Merge:
        return Fail("不应将 root 文档 merge 到其他文档")
    
    // R3: 环检测
    if action.changes_upstream and would_create_cycle():
        return Fail("upstream 变更将形成引用环")
    
    // R4: archive 文件
    if cognition.stage == "X" and action.kind != NoAction and action.kind != HumanReview:
        return Fail("不应修改已归档文档")
    
    // 边界
    if cognition.stage == "2/3":
        return Conditional("2/3 阶段措辞力度存在弹性空间")
    
    return Pass
```

## 七、整体判据

```text
overall_verdict(checks: Vec<LawCheck>) -> Verdict:
    if checks.any(|c| c.result == Fail):  return Fail
    if checks.any(|c| c.result == Conditional): return Conditional
    return Pass
```

当 overall = Fail 时，decision_proposal 被引擎拒绝执行，标记 `human_review_required`。当 overall = Conditional 时，引擎执行但附加警告，仍需人类确认。

### 设计决策：stage 2/3 的 Conditional 模式

顺势 §六 将 stage 2/3（resolve）设为系统性的 Conditional 触发条件。这意味着多数活跃文档的决策都会产生 Conditional 标记——这不是缺陷，而是对 2/3 作为过渡阶段的正确反映：resolve 阶段的措辞本就不应像 propose 那样宽松，也不应像 ratify 那样严格。Conditional 在此处的作用是提醒人类确认措辞力度，而非暗示 criteria 有问题。

## 八、dao_trace 生成

每条返回 Fail 或 Conditional 的法检验，需在 `dao_trace` 中记录对应道的依据：

| 法 | 对应的道 | trace 格式 |
|----|---------|-----------|
| 顺因 | 道二 + 道三 | `"逆因果链：{action} 试图修改上游 {upstream}，违反道二（意图先于代码）"` |
| 有度 | 道一 | `"力度失配：{action.severity} vs {max_divergence_severity}，违反道一（过犹不及）"` |
| 知止 | 道一 + 道四 | `"逾矩：{action} 超出治理边界，违反道四（治理不完备）"` |
| 损补 | 道一 + 道四 | `"方向错误：{action} 对 {div_type} 执行了错误的损补方向，违反道一（定向调节）"` |
| 顺势 | 道一 | `"力度失时：{action} 在 {stage} 阶段力度不匹配，违反道一（治理有节奏）"` |

## 九、自指局限性（道四声明）

本评判标准自身受道四约束。以下盲区为结构性限制，不可通过细化规则消除：

### 9.1 iCL 依赖

iCT 的检验建立在 iCL 认知产出（`cognition`）之上。如果 iCL 误诊发散类型或遗漏 relation_graph 中的 gap，iCT 将基于错误输入做判断。**iCT 不校验 iCL 的准确性**——这是 Mind 内部的分层信任模型：iCL 负责认知，iCT 负责验证决策，两者边界清晰，但信任链的断裂风险需声明。

### 9.2 NLP 不确定性

顺势 §六 R1（措辞匹配）依赖对中文描述的自然语言判断（`contains_weak_hedging` / `contains_mandatory_language`）。中文措辞的"可能"与"应"之间没有明确算法边界——关键词匹配（如检测"必须"/"应当"/"不可"）只能作为近似，无法替代语义理解。实现时需：
- 用关键词匹配作为 first-pass（高 precision，低 recall）
- 将 NLP 不确定性纳入 confidence 字段
- 标记所有 NLP-based Fail 为"建议确认"

### 9.3 gap→divergence 边界

`relation_graph.gaps` 记录被引用但缺失的文档 id。**gap 是客观事实（observation），divergence 是主观判断（judgment）**。只有被 iCL 判定为 `DivergenceType::ReferenceBreak` 的 gap 才会触发五法检验。这意味着：
- 一个 gap 被 iCL 忽略 = iCT 不会对 gap 做出反应
- 这是正确的分层设计（iCL 判断相关性，iCT 不越权替代 iCL），但需显式声明

### 9.4 无法机械验证的约束

以下约束不在 iCT 的可机械验证范围内，标记为 `[human-review]`：
- 损补的"损比补更难"非对称性（需人类判断 损 的 justification 是否充分）
- 知止的"哲学否定性判断"（需语义理解，无法关键词匹配）
- 有度的"action.alternatives 至少 1 个力度不同"（需比较语义而非枚举 ActionKind）
