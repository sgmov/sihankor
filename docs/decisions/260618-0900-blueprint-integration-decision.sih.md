---
id: 260618-0900-blueprint-integration-decision
stage: 3/3
upstream: 240610-1030-on-sihankor-canon
decided-by: moc
---

# 蓝皮书融入决策：CI 守护 + 验码 + 治理追溯标记

## 背景

外部 AI 生成的司衡进化蓝图（`knowledge/drafts/siheng-engine-blueprint.md`）提出 18 个主张。经三轮反推九段检验（C1/C3/C5），与现有体系对照后，三项能力经校准后存活，一项被否决。校准后的能力融入方案见 [蓝皮书融入提案](../proposals/260618-0500-blueprint-integration-proposal.sih.md)（stage 2/3）。

## 方案选择

| 维度 | 决策 | 法依据 |
|------|------|--------|
| C1（主动监察） | 采纳校准版：**CI 守护增强**。PR 下发布 F/G/J 分级结构化违规报告，F 级阻断，G 级评论，J 级静默 | 有度（力度匹配）+ 损补（精确告知） |
| C3（沙盒反证） | 采纳校准版：**验码**。在 CI 中运行已有测试+proptest，仅判定硬崩溃，不判定语义偏离 | 知止（不越界） |
| C5（Agent 认证） | **否决**。替代方案：**治理追溯标记**。每次 commit 的 merge message 追加治理检查摘要，不认证 Agent | 顺因（因果链在代码不在 Agent） |
| C5 原版 | 否决。认证对象不可标识、信息冗余、证伪条件不可执行 | 知止（不做不可执行的鉴） |

## 备选方案

| 方案 | 描述 | 不采纳理由 |
|------|------|-----------|
| A：全量采纳 | 蓝皮书 7 阶段状态机完整实现，包括后台主动监察 Agent、沙盒反证、Agent 认证 | C1（主动监察）违反 Mind 只建议不决定的架构；C3 引入运行时安全风险；C5（Agent 认证）经反推检验判定不通过 |
| B：零采纳 | 保持现有体系不变，不引入蓝皮书任何主张 | 放弃三个经检验存活的增量改进（CI 守护增强、验码、治理追溯标记）。三个 gap（G-CI/G-FUZZ/G-TRACE）持续存在 |
| **C：校准融入（采纳）** | 仅融入经反推检验存活的三项能力 | （见理由） |

## ADR

### decided-by

moc

### 理由

1. **顺因**：蓝皮书是外部 proposal，现有体系是已 ratify 的 spec。取可融入者融入，因果方向不逆。
2. **有度**：三项均为增量增强，不重构现有代码。每项独立可实施、可验证、可回退。
3. **知止**：C5（Agent 认证）被否决。治理追溯标记是更简的替代方案。
4. **损补**：CI 守护补足"为什么不合规"的解释性；验码补足 code-lint 无法检测的运行时崩溃；治理追溯标记补足"这次变更是否经过治理"的可追溯性。

### 后果

#### 正面后果

| 能力 | 预期效果 |
|------|---------|
| CI 守护增强 | 开发者从"CI failed"升级到"为什么 failed + 怎么修"。降低治理规则的认知门槛 |
| 验码 | 补充 code-lint（静态）无法检测的运行时崩溃。提升代码鲁棒性 |
| 治理追溯标记 | 开发者通过 `git log --grep` 可快速判断哪些 commit 经过了治理 |

#### 代价与风险

| 能力 | 代价 |
|------|------|
| CI 守护增强 | PR 评论数量增加。若报告质量不高（报告过长、信息噪音），开发者可能忽略 |
| 验码 | 需开发者在关键函数上主动添加 `#[proptest]` 才有价值。无开发者配合时验码降级为 `cargo test`（已有 CI 能力） |
| 治理追溯标记 | merge message 变长。标记信息本身是形迹日志的副本，不新增信息量 |

### 修正项追踪

提案审查中发现的 7 个问题，修正状态确认：

| 编号 | 问题 | 提案修正状态 |
|------|------|-------------|
| F1 | `git commit --amend` 鸡与蛋问题 | 已修正：改为 PR merge message template + 临时文件（`.siheng/governance-marker.txt`） |
| F2 | Engine-Design-Summary 引用错误 | 已修正：改为"无直接影响" |
| S1 | 提案无 @limitations | 已修正：提案新增 §七（6 项声明） |
| S2 | C5 记录路径不存在 | 已修正：C5 检验记录已存档（`260618-0800-c5-falsification-record`） |
| S3 | 治理追溯标记写入方式缺失 | 已修正：补充完整写入流程（CI→临时文件→PR merge template→merge commit message） |
| S4 | Phase 1→2 接口耦合未声明 | 已修正：在提案 @limitations 第 4 项声明，接口契约在 Phase 1 实施时定义 |
| G1 | 5 列表格违规 | 已豁免：标注可读性原因 |

### 被否决项

**C5（Agent 思辨认证）不通过。** 反推检验记录：`knowledge/notes/260617-0900-c1-c3-falsification-records.sih.md`（C1/C3）及 `knowledge/notes/260618-0800-c5-falsification-record.sih.md`（C5）。

否决理由：

1. 认证对象（Agent）不可标识：Agent 是模型×prompt×工具×知识库的流体组合
2. 认证信息与 CI 守护冗余：现有产出物治理已覆盖
3. 证伪条件（Bug 率）不可归因、不可执行

### 实施顺序

```text
Phase 1: CI 守护增强（结构化报告）
  -> Phase 2: 治理追溯标记（复用 CI 守护输出）
    -> Phase 3: 验码（需 proptest 集成）
```

### 对下游规约的影响

| 文档 | 影响 |
|------|------|
| `Engine-Design-Summary.sih.md` | 无直接影响 |
| `Dev-Governance.sih.md` | 治理链补充 CI 守护→标记的自动追加步骤 |
| `code-lint` proposal | 补充 code-lint（静态）与验码（动态）的关系声明 |
| `format-lint` proposal | 补充结构化报告的格式规范 |

## 决策的可证伪条件

本决策在以下任一条件满足时应被重新审查（Reopen）：

| 编号 | 条件 | 触发动作 |
|------|------|---------|
| D1 | CI 守护增强上线 6 个月后，PR 评论中超过 30% 被开发者标记为 noise 或忽略 | 降级报告频率，或改为仅 F 级评论 |
| D2 | 验码上线 6 个月后假阳性率 >10%（即 10 次硬失败报告中超过 1 次为环境/配置问题而非代码 bug） | 验码降级为仅报告不阻断（G 级） |
| D3 | 治理追溯标记上线 3 个月后，调查显示开发者从未使用 `git log --grep="Governance:"` 检索 | 标记静默记录到形迹日志，不注入 merge message |
| D4 | 蓝皮书的原始可证伪条件被证实（治理介入后长期 Bug 率未显著低于非治理代码） | 重新评估三项能力的整体有效性，考虑回退 |

## @limitations

1. **决策范围**：本决策仅涉及蓝皮书中经反推检验存活的三项能力（C1/C3 校准版 + C5 替代方案）。蓝皮书其他主张（如多 Agent 协同协议、VSCode 插件）未在本次决策中处理。
2. **验码的前提**：验码的价值依赖开发者在关键函数上主动添加 `#[proptest]`。如果开发者不配合，验码退化为运行已有测试（与现有 CI 的 `cargo test` 等价）。
3. **单 Agent 场景的增量价值**：治理追溯标记在单 Agent 开发场景（大多数情况）的增量价值较低：：每次 commit 经过同一个 CI 流程，标记仅反映 CI 状态。标记在多 Agent 协作或长期维护场景中价值更大。
4. **本决策自身的可证伪条件依赖数据采集**：D1-D4 的可证伪条件需要对 PR 评论反馈、假阳性率、标记使用频率的数据采集。如果数据未采集，证伪条件不可执行。数据采集机制需在 Phase 1 实施时建立。
5. **决策不替代持续审查**：本决策的 ratify 不意味着三项能力永久有效。每次 CI 守护运行本身就是对治理效力的持续检验。

- [蓝皮书融入提案](../proposals/260618-0500-blueprint-integration-proposal.sih.md)
- [法论](../specs/philosophy/On-SiHankor-Canon.sih.md)
- [鉴论](../specs/philosophy/On-SiHankor-Assay.sih.md)
- [Mind 设计规范](../specs/engineering/SiHankor-Mind-Design.sih.md)
- [引擎设计摘要](../specs/engineering/SiHankor-Engine-Design-Summary.sih.md)
- [引擎开发治理](../specs/engineering/SiHankor-Dev-Governance.sih.md)
- [C1/C3 反推检验记录](../knowledge/notes/260617-0900-c1-c3-falsification-records.sih.md)
- [C5 反推检验记录](../knowledge/notes/260618-0800-c5-falsification-record.sih.md)

## SEE-ALSO

- `knowledge/drafts/siheng-engine-blueprint.md`：蓝皮书底稿
- 260616-2130-code-canon-derivation：代码术层约束的法层推导
- 260616-2100-code-lint-proposal：code-lint 设计
