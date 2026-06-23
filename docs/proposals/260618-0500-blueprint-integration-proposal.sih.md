---
id: 260618-0500-blueprint-integration-proposal
stage: 3/3
upstream: 240610-1030-on-sihankor-canon
---

# 蓝皮书融入提案：CI 守护 + 验码 + 治理追溯标记

> 基于外部 AI 生成的司衡进化蓝图（`knowledge/drafts/siheng-engine-blueprint.md`），经三轮反推九段检验，提出三项可融入现有体系的能力增强，一项被否决。

## 一、正名：要解决的 gap

### 1.1 蓝皮书的意图

蓝皮书的核心意图是让司衡从"被动分析"走向更完整的治理闭环。它提出了 18 个主张，经与现有体系对照，5 个存在冲突或空白（C1~C5），3 个经检验后存活或被校准。

### 1.2 当前 gap

现有体系存在三个可被蓝皮书洞察补足的空白：

| gap | 描述 | 蓝皮书来源 |
|-----|------|-----------|
| G-CI | CI 阶段缺少**结构化治理报告**。format-lint/code-lint 返回 pass/fail，但不告知开发者"为什么不合规、依据哪条法"。 | C1（校准版） |
| G-FUZZ | code-lint（静态）无法检测**运行时鲁棒性问题**（除零、空指针、资源泄漏）。虽不属于"意图对齐"治理的核心职责，但作为附加验证有价值。 | C3（校准版） |
| G-TRACE | 形迹日志记录了治理历史，但开发者**无法从代码变更直接看到**"这次变更是否经过了治理"。 | C5（替代方案） |

### 1.3 蓝皮书中被否决的主张

| 主张 | 否决理由 | 依据 |
|------|---------|------|
| C5 原版（Agent 思辨认证） | 认证对象不可标识；认证信息与 CI 守护冗余；证伪条件（Bug 率）不可执行 | 反推检验 #3，判定不通过 |

## 二、方案对比

### 方案 A：全量采纳蓝皮书（不可行）

将蓝皮书 7 阶段状态机完整实现，包括后台主动监察 Agent、沙盒反证、Agent 认证。

**优点**：哲学闭环最完整。

**缺点**：C1（主动监察）违反 Mind 只建议不决定的架构；C3（沙盒反证）引入运行时安全风险；C5（Agent 认证）无法落地。已被三轮反推检验否决。

### 方案 B：零采纳（保守）

保持现有体系不变，不引入蓝皮书的任何主张。

**优点**：零风险，体系稳定。

**缺点**：放弃了三个经检验存活的增量改进（CI 守护增强、验码、治理追溯标记）。蓝皮书的洞察被浪费。

### 方案 C：校准融入（推荐）

仅融入经反推检验存活的三项能力，每项严格限定在现有 Mind 约束和 CI 守护框架内。

**优点**：每项能力有明确的 gap 驱动、有道法依据、有可证伪条件。不破环现有架构。

**缺点**：哲学完整性不如方案 A（但方案 A 不可行）。

### 推荐方案 C，理由

顺因：蓝皮书是外部 proposal（1/3），现有体系是已 ratify 的 spec（3/3）。因果方向：从现有体系审视蓝皮书，取可融入者融入，不可融入者记录否决理由。不反向。

有度：三项能力均属增量增强，不要求重构现有代码。每项独立可实施、独立可验证、独立可回退。

## 三、三项能力详述

### 3.1 CI 守护增强（C1 校准版）

**要解决的问题**：format-lint/code-lint 的 CI 输出是二进制 pass/fail。开发者收到"CI failed"时，不知道违反了哪条规则、规则的法层依据是什么、如何修复。

**方案**：扩展 CI 守护，在 PR 下发布**结构化违规报告**。

**报告格式**：

```text
[SiHankor CI Guardian] 3 violations detected:

[F] L-16 violation at src/mind/iww.rs:142
    unwrap() hides error gap. Rule: 错误路径须暴露间隙，不得隐藏为 panic.
    Fix: Replace with Result<T, E> propagation or proper error handling.
    Dao: 道四（规约与实现必有间隙） Fa: 有度

[G] G-04 violation at docs/specs/engineering/SiHankor-Mind-Design.sih.md
    Missing @limitations section. Rule: 规范必须声明自身的不完备性.
    Fix: Add @limitations section documenting known blind spots.
    Dao: 道四 Fa: 损补

[J] J-02 violation at src/common/generic_service.rs:28
    Function body exceeds 50 lines (currently 67).
    Fix: Consider extracting helper functions.

All violations are based on deterministic rules. Human override: reply "override <rule-id>" to acknowledge and bypass.
```text

**过滤规则**：

| 违规级别 | 行为 |
|---------|------|
| F（戒） | CI 失败 + 发布评论（阻断） |
| G（规） | CI 通过 + 发布评论（不阻断） |
| J（矩） | 仅静默记录到形迹日志 |

**道法依据**：

| 设计点 | 法 | 理由 |
|--------|----|------|
| 结构化报告（规则编号+位置+修复建议） | 损补 | 精确告知违规位置和修复路径 |
| F 阻断/G 评论/J 静默 | 有度 | 力度匹配违规级别 |
| 所有违规附道法追溯 | 顺因 | 每条规则可追溯至道和法 |
| 确定性规则触发 | 知止 | 不做语义判断 |

**工程影响**：需扩展 `core/validator` 产出的 `ValidationResult`，增加违规级别和修复建议字段。CI 集成需新增 GitHub/GitLab API 评论能力（可选：默认输出到 stdout，CI 平台自行处理）。

**验收标准**：

1. 对一次包含 3 类违规（F/G/J 各一）的 PR，CI 输出包含 3 条结构化报告，每条含规则编号、位置、道法追溯
2. F 级违规导致 CI 非零退出码，G/J 级不导致
3. 人工 override 后 CI 仍通过

### 3.2 验码（C3 校准版）

**要解决的问题**：code-lint（clippy 静态分析）可检测已知反模式（`unwrap()`、缺文档、命名违规），但无法检测运行时崩溃（除零、空指针、资源泄漏）。

**正名声明**：这是**验码**，不是**鉴**。鉴（反推九段式）检验道层主张的真伪。验码检验代码的运行时鲁棒性。两者不可混淆。

**方案**：在 CI 守护框架内集成现有 Rust fuzz 工具，不新建 fuzz 引擎。

**实现**：

- 利用 `cargo test` 运行项目已有测试
- 可选集成 `proptest`：开发者在关键函数上声明输入契约（`#[proptest]`），CI 自动运行属性测试
- 仅判定 crash/panic/超时（硬失败），不判定"规约偏离"（语义判断）
- 测试失败→CI 非零退出码（经 C1 规则），不触发自动代码重生成

**不做什么**：

- 不自动生成边界输入（输入策略由开发者在 `proptest` 中定义）
- 不执行任意代码（仅运行项目已有测试框架）
- 不判定"代码是否完成了意图"（那是语义判断，人类的事）

**与 code-lint 的关系**：互补，非重叠。

| | code-lint（静态） | 验码（动态） |
|---|---|---|
| 检测对象 | AST 结构、命名、文档 | 运行时行为 |
| 发现什么 | 已知反模式 | 崩溃、panic、超时 |
| 假阳性率 | 低 | 低（仅硬失败） |
| 法层依据 | L-14~L-19 | L-16（错误路径须暴露间隙） |

**道法依据**：

| 设计点 | 法 | 理由 |
|--------|----|------|
| 不新建 fuzz 引擎，集成现有工具 | 知止 | 不做工具已做到的事 |
| 仅判定硬失败 | 知止 | 不假装能做语义判断 |
| 测试失败由 C1 规则处置 | 顺因 | CI 守护是阻断决策层，验码是检测层 |
| 输入策略由开发者定义 | 知止 | 不假装机器比人类更懂业务语义 |

**工程影响**：最小。无需新增 Rust 模块。CI 配置增加 `cargo test` + 可选 `cargo proptest` 步骤。验码的结构化输出复用 C1 的报告格式。

**验收标准**：

1. 对包含除零 panic 的 PR，验码检测到崩溃并输出结构化报告（函数名、panic 消息、位置）
2. 验码在 60 秒内完成（对 <10 个 `proptest` 函数的项目）
3. 验码的假阳性率 <5%（即 95% 的硬失败报告确为代码 bug，非环境问题）

### 3.3 治理追溯标记（C5 替代方案）

**要解决的问题**：用户查看 Git 历史时，无法快速判断"这次 commit 是否经过了司衡治理检查"。形迹日志有记录，但与代码历史分离。

**正名**：这是**治理追溯标记**，不是 Agent 认证。标记绑定到**每次代码变更**（commit），不绑定到 Agent（模糊实体）。标记不声明"Agent 是好的"，只声明"这次变更通过了以下检查"。

**方案**：CI 守护通过后，在 commit message 或 PR merge message 中自动追加治理标记。

**标记格式**：

```text
Governance: siheng-ci-passed
  lint=ok (0 F-violations, 2 G-violations overridden)
  verify=ok (验码: 3/3 proptest passed, 0 crashes)
  rules=L-14..L-19 (all passed)
  checked-at=2026-06-18T05:00:00Z
```text

**获取方式**：

```bash
git log --grep="Governance:" --oneline    # 查看所有经过治理的 commit
git log --grep="Governance: siheng-ci-passed"  # 仅看通过的
```

**不做什么**：

- 不认证 Agent
- 不评分 Agent
- 不绑定到具体模型版本
- 治理标记不替代 CI 守护：每次变更仍需重新检查

**道法依据**：

| 设计点 | 法 | 理由 |
|--------|----|------|
| 标记绑定到 commit，不绑定到 Agent | 顺因 | 因果链：代码→CI 检查→标记。Agent 不在这个链上 |
| 基于已有 CI 守护输出 | 知止 | 不新增基础设施 |
| 每次变更即时标记 | 有度 | 粒度匹配 |
| 标记进入 Git 历史 | 损补 | 补足了"这次变更是否经过治理"的可追溯性 |

**工程影响**：CI 脚本中增加一步：将 C1 结构化报告的摘要追加到 commit message（通过 `git commit --amend` 或 PR merge template）。极低风险：标记是纯文本，不对代码行为产生任何影响。

**验收标准**：

1. `git log --grep="Governance:"` 可检索到所有经 CI 守护检查的 commit
2. 标记内容包含 lint 结果、验码结果、规则列表、时间戳
3. 标记不改变 commit 的代码内容（仅 metadata）

## 四、对现有规约的影响

| 受影响文档 | 影响 | 变更类型 |
|-----------|------|---------|
| `Engine-Design-Summary.sih.md` | $五 MCP 工具清单：不新增工具，仅扩展 CI 守护报告格式 | 轻量修订 |
| `Mind-Design.sih.md` | 无需修改。三项能力均在 Mind 的三个硬约束内运行 | 无影响 |
| `Canon.sih.md` | 无需修改。F/G/J 过滤方向已是 Canon 的现有定义 | 无影响 |
| `Dev-Governance.sih.md` | 治理链补充：CI 守护→治理追溯标记的自动追加步骤 | 轻量修订 |
| `code-lint` proposal | 补充：code-lint（静态）与验码（动态）的关系声明 | 轻量修订 |
| `format-lint` proposal | 补充：结构化报告的格式规范 | 轻量修订 |

无新 spec 需要建立。三项能力均是对已有 CI 流程的增量增强。

## 五、验收标准汇总

| 能力 | 验收标准 | 可证伪条件 |
|------|---------|-----------|
| CI 守护增强 | F/G/J 分级报告正确输出 | 如果 PR 评论中超过 30% 被开发者标记为 noise，则降级报告频率 |
| 验码 | 检测运行时崩溃，假阳性 <5% | 如果 6 个月内假阳性率 >10%，则仅报告不阻断（降级为 G 级） |
| 治理追溯标记 | `git log --grep` 可检索 | 如果 3 个月后开发者从未使用该命令检索，则标记无实际价值，考虑静默记录 |

## 六、实施顺序

```text
Phase 1: CI 守护增强（结构化报告）
  -> Phase 2: 治理追溯标记（复用 CI 守护输出）
    -> Phase 3: 验码（需 proptest 集成，依赖开发者主动标注）
```

Phase 1 和 Phase 2 是 CI 输出格式的改进，零架构风险。Phase 3 需要开发者配合（在关键函数上加 `#[proptest]`），属渐进采纳。

## 七、被否决项

**C5（Agent 思辨认证）不通过。** 反推检验记录见 `knowledge/notes/260617-0900-c1-c3-falsification-records.sih.md`（C1/C3）及 `knowledge/notes/260618-0800-c5-falsification-record.sih.md`（C5，待存档）。

不通过理由摘要：

1. **认证对象不可标识**。Agent 是模型×prompt×工具×知识库的流体组合，任何认证标记在颁发瞬间即开始与真实 Agent 脱节。
2. **认证信息冗余**。CI 守护 + 形迹日志已经通过治理产出物间接治理了生产者。Agent 认证不增加可观测内容。
3. **证伪条件不可执行**。Bug 率不可归因于 Agent 认证这一单一变量。无法构造可比对照组。

## DEPS

- 240610-1030-on-sihankor-canon：法论（F/G/J 三级力度定义）
- 240602-1000-on-sihankor-assay：鉴论（反推九段式方法论）
- 260613-1650-sihankor-mind-design：Mind 设计规范（三个硬约束）
- 260611-0000-sihankor-engine-design-summary：引擎设计摘要（现有架构）
- 260616-1200-sihankor-dev-governance：引擎开发治理（proposal 格式规范）
- 260617-0900-c1-c3-falsification-records：C1/C3 反推检验记录
- 260618-0800-c5-falsification-record：C5 反推检验记录（待存档）

## SEE-ALSO

- `knowledge/drafts/siheng-engine-blueprint.md`：蓝皮书底稿
- 260616-2130-code-canon-derivation：代码术层约束的法层推导（L-14~L-19）
- 260616-2100-code-lint-proposal：code-lint 设计
