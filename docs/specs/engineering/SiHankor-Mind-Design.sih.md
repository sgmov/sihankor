---
id: 260613-1650-sihankor-mind-design
stage: 2/3
upstream: 240610-1030-on-sihankor-canon
---

# 司衡思维核心（Mind）设计规范

> **术层文档。** 司衡思维核心（sihankor-mind）的工程设计规范：架构、三机流转模型、MCP 工具定义、与现有引擎的边界。不重新定义道和法，只展示道和法在 Mind 中的工程化展开。法层原则见[《司衡法论》](../philosophy/On-SiHankor-Canon.sih.md)，哲学约束概述见[《司衡工程映射》$八](./SiHankor-Engineering-Mapping.sih.md#八sihankor-mind-的哲学约束)。

## 一、破题：Mind 是什么

### 1.1 存在理由：几层缺口

[《司衡工程映射》$八](./SiHankor-Engineering-Mapping.sih.md#几层补全) 诊断了当前引擎的关键缺口：引擎从问题直接跳到工具——"文档需要验证？加一条规则。文档需要去重？写一个脚本。"——没有经过道→法→术→几的结构化推理。

在司衡六层脉络中，几层（三机流转的认知与决策节点）是"将动未动的微妙节点"——在形迹被修改之前，施加认知和判断。当前引擎直接操作形迹（parse/validate/index），跳过几。

**Mind 就是几层的工程实现**：一个对话式 MCP 服务器，按六层脉络对输入的问题、意图或提案进行结构化推导，产出包含完整推导链的分析报告。引擎从 Mind 接收决策 JSON 后执行文件操作。

### 1.2 核心定义

> Mind 对输入的问题、意图或提案，按司衡六层脉络（道→法→术→几→约→形迹）进行逐层分析，产出包含完整推导链的结构化分析报告。Mind 不操作文件，不代替人类做决定，不假装全知。

### 1.3 三个硬约束

沿用[《司衡工程映射》$八](./SiHankor-Engineering-Mapping.sih.md#八sihankor-mind-的哲学约束) 的三个硬约束：

- **只分析不写入**：Mind 不修改代码和规范——只提供分析结果。顺因之法：机器沿因果方向分析，但不代替人做决定。
- **只建议不决定**：Mind 的输出是建议而非指令——最终决定权在人类。知止之法：机器知道自己的边界，不越权。
- **只自知不假装**：Mind 必须声明分析不确定性和认知盲区——不假装全知。道四：治理工具自身也受道的约束。

## 二、道法约束

每条道和法在 Mind 中的工程约束。不重新定义道和法——引用 Tao 和 Canon 的对应条款，标注"因此 Mind 必须做到什么"。

| 道/法 | 依据                                                                               | 对 Mind 的约束                                                                                             |
| ----- | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| 道一  | [《道论》$2.1](../philosophy/On-SiHankor-Tao.sih.md#21-定义)：发散自-然，收敛必-为 | 文档体系必然发散——Mind 必须做跨文档关系照见，不能只做单文档检查                                            |
| 道二  | [《道论》$2.2](../philosophy/On-SiHankor-Tao.sih.md#22-道二)：意图先于代码         | 分析必须先理解意图元数据（type/stage/upstream）。对 propose 文档做 ratify 级检查是违道                     |
| 道三  | [《道论》$2.3](../philosophy/On-SiHankor-Tao.sih.md#23-道三)：代码自晦，意图必复   | 文档字面不揭示其关系——Mind 的立身之本是照见单文档看不到的跨文档关系（引用/重复/冲突/空白）                 |
| 道四  | [《道论》$2.4](../philosophy/On-SiHankor-Tao.sih.md#24-道四)：规约与实现必有间隙   | Mind 的分析报告也是规约——必含 `@limitations`、`self_question`、`confidence`。不输出"确定正确"的结论        |
| 顺因  | [《法论》$2.1](../philosophy/On-SiHankor-Canon.sih.md#21-顺因尊重因果方向)         | 三机流转不可逆序：iCL→iWW→iCT 严格顺序。分析未完成时，不生成建议                                           |
| 有度  | [《法论》$2.2](../philosophy/On-SiHankor-Canon.sih.md#22-有度收敛恰到好处)         | 分析力度按文档 stage 分级：propose 轻量，resolve 中量，ratify 全量                                         |
| 知止  | [《法论》$2.3](../philosophy/On-SiHankor-Canon.sih.md#23-知止知道不做什么)         | 三类不分析：非 .sih.md 不做结构分析、意图模糊的文档标记 human_review、哲学讨论不判对错                     |
| 损补  | [《法论》$2.4](../philosophy/On-SiHankor-Canon.sih.md#24-损补损有余补不足)         | 分析输出区分减损建议（合并重复/归档过时）和补充建议（补全引用链/补全 limitations），不输出笼统的"问题"标记 |
| 顺势  | [《法论》$2.5](../philosophy/On-SiHankor-Canon.sih.md#25-顺势力度适配场景)         | archive/ 中的文档不生成修改建议。propose 阶段建议以"可能"措辞，ratify 阶段以"应"措辞                       |


## 三、四步分析法（术层）

术层定义 Mind 的分析维度。四步对应四条道的追问，每一步的输出可触发前一步的迭代。

```text
输入 → ① 意图定位（道二·顺因）
         确定文档在治理体系中的位置
         追问：这份文档的 type/stage/upstream 是什么？给谁看的？确定性什么程度？
         输出：文档治理定位（type × stage × upstream 链）

     → ② 关系照见（道三·损补）
         发现跨文档关系
         追问：它的上游是谁？谁依赖它？有没有另一份文档说同样的事？
         输出：关系图谱（引用/重复/冲突/空白 四维度）

     → ③ 发散诊断（道一·有度）
         判定发散类型和严重程度，区分正常发散和有害发散
         追问：这是意图漂移？引用断裂？重复冗余？还是良性多角度讨论？
         输出：发散诊断列表（类型/严重度/confidence）

     → ④ 间隙声明（道四·知止）
         声明本分析的盲区和不确定性
         追问：有什么没看到？哪个判断最可能出错？什么需要人类确认？
         输出：@limitations + self_question + 待确认项
```

④ 的输出可以触发新一轮 ①——例如发现某个文档的意图定位需要人类确认后，用确认结果重新分析。


## 四、三机流转与输出 Schema（几层+约层）

### 4.1 三机流转模型

三机在 Mind 中不是三个独立的 MCP 工具，而是一次分析中三个依次流转的认知阶段。流转不可跳过、不可逆序。

```mermaid
flowchart LR
    Input["输入<br/>文档/问题/意图"] --> iCL
    iCL["iCL 明晰机<br/>认知：照见意图、发现关系、诊断发散"] --> iWW
    iWW["iWW 消息机<br/>驱动：从认知生成决策建议"] --> iCT
    iCT["iCT 方圆机<br/>验证：检验决策是否合道"] --> Output["输出<br/>完整分析报告"]

    iCT -.->|"fail: 退回 iWW<br/>重新生成建议"| iWW
```

| 阶段          | 职责                               | 输入                                            | 产出                                                         |
| ------------- | ---------------------------------- | ----------------------------------------------- | ------------------------------------------------------------ |
| iCL（明晰机） | 认知：照见意图、发现关系、诊断发散 | 文档/问题/意图                                  | `cognition`：治理定位 + 关系图谱 + 发散诊断                  |
| iWW（消息机） | 驱动：从认知生成决策建议           | iCL 的 `cognition`                              | `decision_proposal`：推荐行动 + 多方案对比 + 影响范围 + 理由 |
| iCT（方圆机） | 验证：检验决策是否合道             | iCL 的 `cognition` + iWW 的 `decision_proposal` | `verification`：五法逐条检验 + 道层可追溯性                  |

如果 iCT 返回 fail，流转退回到 iWW 重新生成建议。如果连续三次 fail，标记为 `human_review_required` 并输出失败原因。

### 4.2 输出 Schema（约层）

```rust
struct AnalysisResult {
    schema_version: String,       // "0.1.0"
    analysis_id: String,
    analysis_target: AnalysisTarget,

    // iCL 产出
    cognition: Cognition,

    // iWW 产出
    decision_proposal: Option<DecisionProposal>,

    // iCT 产出
    verification: Option<Verification>,

    // 道四：必填
    limitations: Vec<Limitation>,
    self_question: String,
    human_review_required: Vec<String>,
}

struct Cognition {
    governance_position: GovPosition,   // type × stage × upstream 链
    relation_graph: RelationGraph,      // 引用/重复/冲突/空白
    divergence_diagnosis: Vec<Divergence>, // 发散诊断列表（每项含 type/severity/confidence）
}

struct DecisionProposal {
    recommended_action: Action,         // merge | move | rename | archive | no_action | human_review
    rationale: Rationale,               // {dao_basis, fa_basis}
    alternatives: Vec<Alternative>,     // 多方案对比
    affected_documents: AffectedDocs,   // 下游影响范围
}

struct Verification {
    five_law_check: Vec<LawCheck>,      // 五法逐条：pass | fail | conditional
    overall: Verdict,                   // pass | fail | conditional
    dao_trace: Vec<DaoTrace>,           // 决策的道层可追溯性
}
```

关键约束：

- `cognition` 和 `limitations` + `self_question` 在所有分析中为必填——道四不因分析深度而放宽
- `decision_proposal` 和 `verification` 在 `full_analysis` 中必填，在 `analyze_document` 中可选
- 每个 `Divergence` 项必有 `confidence`（道四要求）
- `five_law_check` 的 5 条法必须逐条检验，不可省略


## 五、MCP 工具定义（形迹层）

工具按**三机流转阶段**划分，不按问题类型划分。4 个工具覆盖 3 个认知阶段：

| 工具               | 流转阶段    | 输入                           | 输出                              | 用途                           |
| ------------------ | ----------- | ------------------------------ | --------------------------------- | ------------------------------ |
| `analyze_document` | iCL         | 文档路径/ID                    | `cognition`                       | 单独查看某文档的治理定位和关系 |
| `propose_decision` | iCL→iWW     | 文档路径/ID 或已有 `cognition` | `cognition` + `decision_proposal` | 在认知基础上生成决策建议       |
| `verify_decision`  | iCT         | `decision_proposal` + 上下文   | `verification`                    | 单独验证已有决策是否合道       |
| `full_analysis`    | iCL→iWW→iCT | 文档路径/ID 或问题描述         | 完整 `AnalysisResult`             | 完整三机流转分析               |

### 安全约束

Mind 不执行写入操作——文件修改由下游引擎执行。决策 JSON 中传递的写入指令需遵守：

1. **dry-run 先于执行**：`affected_documents` 的每项需展示替换前后的具体文本行，待人类确认后由引擎执行
2. **最小影响面**：`affected_documents` 只列直接下游，不推测间接影响
3. **可回退**：每个写入建议附回退操作描述


## 六、与现有引擎的边界

```mermaid
flowchart LR
    subgraph Mind[\"sihankor-mind（思维核心）\"]
        M1[\"几层：三机流转\"]
        M2[\"术层：四步分析法\"]
        M3[\"约层：结构化输出\"]
    end
    subgraph Engine[\"sihankor 引擎（执行层）\"]
        E1[\"形迹层：文件操作\"]
        E2[\"parse → validate → modify\"]
    end
    Mind -->|\"JSON\"| Engine

    subgraph MindBound[\"Mind 不可跨边界\"]
        MB1[\"Mind 不写文件\"]
        MB2[\"Mind 不代替人类决策\"]
    end
    subgraph EngineBound[\"引擎不可跨边界\"]
        EB1[\"引擎不做推导\"]
        EB2[\"引擎不修改 Mind 的决策逻辑\"]
    end
```

Mind 的职责在输出 JSON 时结束。引擎接收 JSON 后执行 `affected_documents` 中描述的操作。引擎不自行判断是否执行——如果 `verification.overall` 为 fail，引擎应拒绝执行并标记为 human_review。


## 七、与约系的关系

Mind 的约层（结构化输出 Schema）与引擎约系（`.sih/index/` 中的 SQLite 索引）是不同层次的约，不重叠：

|          | Mind 约层            | 引擎约系                |
| -------- | -------------------- | ----------------------- |
| 约取对象 | 思维过程             | 代码/文档形迹           |
| 产物     | JSON 分析报告        | SQLite 索引表           |
| 存储     | 会话级（不持久化）   | 持久化（`.sih/index/`） |
| 目的     | 让决策可追溯、可验证 | 让形迹可搜索、可引用    |

Mind 依赖引擎约系获取分析数据，但 Mind 自身的约层服务于认知透明度——两者是消费关系，不是重叠关系。


## 八、自我质疑

以下三条质疑直击 Mind 设计的核心假设。不是形式化的"道四检查项"，而是设计规范必须面对的真实问题。

1. **三机分权在 MCP 层面是否过度工程化？** 单次 LLM 调用自然包含了三机功能——理解、建议、自我检查都在一次推理中完成。强行拆成 iCL→iWW→iCT 三个阶段增加了延迟和 token 成本。收益（推导链可追溯）是否真的需要三机形式化来实现？或者，三机作为输出结构中的逻辑分区（而非三个独立调用）更务实？

2. **约层 Schema 的详细度是否合理？** 当前 Schema 包含大量顶层字段。对简单的文档验证场景，填充全部字段是过度的。是否应该有"轻量模式"（仅 `cognition`）和"完整模式"（含 `decision_proposal` + `verification`）的区分？

3. **Mind 和引擎的边界是否会在实践中模糊？** 如果引擎扩展了跨文档分析能力（如引用完整性检查），Mind 的部分认知功能就会被形迹层吸收。Mind 的边界是否需要动态调整——随着引擎能力增长，Mind 退化为引擎的前置分析模块而非独立 MCP 服务器？

4. **Mind 的设计是否在"术"与"几"之间摇摆？** 四步分析法（术层）和三机流转（几层）的边界在当前设计中不够清晰。术层回答"分析什么维度"，几层回答"谁来执行判断"。如果术层已经包含了发散诊断的判定标准，几层的"验证"是否变成了冗余的对账？

> 注：本规范为工程设计文档（stage 2/3，在 specs/engineering/ 下，nature 为 spec）。Mind 尚未实现，本文描述的是设计蓝图而非已实现的功能。实现时以 `src/main.rs` 中的实际代码为权威来源。本文的非治理性底层设计（如具体 Rust struct 字段）在实现过程中可能调整。待 Mind 实现完成后推进 stage 至 3/3。
