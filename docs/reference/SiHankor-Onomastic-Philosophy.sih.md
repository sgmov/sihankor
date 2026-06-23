---
id: 260613-1728-sihankor-onomastic-philosophy
stage: 3/3
upstream: 240602-0900-on-sihankor
---

# 司衡的命名哲思

> 司衡的每一个命名都不只是标签，而是哲学概念的精确投射。

本文定义司衡全部核心术语的中文对、英文对与代号的三重对齐体系，并阐述其命名哲学：为何这样命名、如何构造新名、哪些不可变更。

## 一、命名哲学

### 1.1 三重对齐原则

司衡的命名体系遵循一个核心模式：中文对 + 英文对 + 代号。三者不是翻译关系，而是同一哲学概念在三个符号系统中的平行表达。

- 中文侧提供拆字见意的深度：每个字有独立含义，合在一起生成新的意义。
- 英文侧提供词源根基：英文对从拉丁、希腊或古英语中择取，承载概念的西方哲学谱系。
- 代号侧提供工程的简洁性：从英文对中提取首字母，在代码、命令行、配置文件中零歧义。

### 1.2 拆字见意

司衡命名哲学的精髓。一个好的命名，拆开来看每个字都有独立含义，合起来又生成超越字面之和的新意义。

以方圆机 (iCT) 为例：

- 方：刚性、判定、pass/fail：矩的功能
- 圆：弹性、标准、should comply：规的功能
- 方圆合：规矩之治：完整地描述了 iCT 掌管 Schema 定义和合规验证的职能

以明晰机 (iCL) 为例：

- 明：日月并照：光照的前提，照亮结构
- 晰：日 + 析：日光下剖析，分辨细节
- 明晰合：认知之治：先照亮前提（明），再分辨细节（晰），从明到晰是递进

### 1.3 代号提取规则

代号从英文对中提取，遵循以下规则：

- 两词英文对：取两词首字母，大写。例：Celestial & Terrestrial -> CT
- 代号前缀：i- 标记 Incipient（几层），表示该代号处于"将动未动"的执行层
- 代号必须可发音或可识读。不接受无意义的字母组合
- 代号一经 ratify，不可变更。在代码库中散布，变更等于全局重构

## 二、系统命名

### 2.1 司衡 (SiHankor)

- 中文对：司-衡
- 英文对：SiHankor
- 代号：sihankor (小写，用于文件路径与代码标识符)

司 (Si)：司职：履行职能。不是主宰，而是代码工程自身演化出治理需求时自然涌现的机能。

衡 (Hankor)：称量、持平。从度（量）到衡（平）到鉴（照）的完整治理过程。Hankor 为司衡体系的构词：Hank（英语本义：一束、一卷：线绳之束，隐喻代码工程中发散纠缠与治理束整的张力）+ or（引擎，驱动执行）。

合论：SiHankor：司职称量、驱动持平之治理引擎。

### 2.2 论 (On-) 系列命名

司衡哲学体系中的专题论述，以 "On-SiHankor-" 为前缀：

- On-SiHankor：司衡论（总纲）
- On-SiHankor-Tao：司衡道论
- On-SiHankor-Assay：司衡鉴论
- On-SiHankor-Canon：司衡法论

元不是论：：元不在论域内。关于元的梳理，见 `Arche-The-One-Above-Being`。

本文不属于 On- 系列：它是命名参照标准（reference canon），而非哲学论述。

文件命名规则：大驼峰，SiHankor 保留大写 S 和大写 H。

## 三、六层脉络命名

六层脉络从根本原理到可观测世界。层的命名遵循"日常词汇 + 哲学负载"原则：字面可读，深究有意。

- **元 (Arche)**：希腊语 arkhe，本原、开端、第一原理。使道得以可能的构成性条件。Arche 是 archetype 与 architect 的词根：既是原型也是建构者。
- **道 (Tao)**：中文 Wade-Giles 拼音。代码工程的因果必然性。Tao 在国际学界已为通用术语（Taoism, Tao Te Ching）。
- **法 (Canon)**：希腊语 kanon，规矩、标准、法则。从道自然生出的方法论原则。Canon 承载"被确立的权威准则"之意：非任意规定，而是被验证为合道。
- **术 (Techne)**：希腊语 tekhne，技艺、工艺、实践智慧。法的工程化展开。Techne 是 technology 的词根：正是"将原理转化为实践"的本义。
- **几 (Incipient)**：拉丁语 incipere，开始、将发未发。事物将动未动的微妙节点。三机在此层执行。Incipient 精确捕捉了"已启动但未完成"的状态。
- **约 (Brief)**：拉丁语 brevis，短、压缩。从博返约的信息压缩。Brief 既是"摘要"（名词）也是"简短"（形容词）：双重贴合。
- **形迹 (Manifest)**：拉丁语 manifestus，显然可见的。可观测产物。Manifest 同时是"清单"（名词，如 cargo manifest）和"显现"（动词）：形迹正是道的显化清单。

### 3.1 几层的特殊地位

`Incipient` 是六层中唯一获得代号前缀的层：所有处于几层的实体，其代号均以 i- 为前缀。三机之"机"即"几"（Incipient）：机微，事物将动未动之节点。工程实践中借"机器"之意（引擎运转），但其哲学本体是"机微"而非"机械"。

- iCT (Incipient Celestial & Terrestrial)：方圆机
- iWW (Incipient Waning & Waxing)：消息机
- iCL (Incipient Clear & Clarify)：明晰机

这意味着：任何非 i- 前缀的代号不在几层；任何在几层的实体必须带有 i- 前缀。

## 四、三机体系命名

三机之"机"，哲学本体为"几"（Incipient）：机微，事物将动未动之微妙节点。工程实践中借"机器"之喻（引擎驱动、规则判定、认知解析），但"方圆机"不是"方圆机器"，而是"方圆之机微"：天圆地方之道的将动未动处。

### 4.1 方圆机 (iCT)

- 中文对：方-圆
- 英文对：Celestial & Terrestrial
- 代号：iCT
- 词源：Celestial (天, 拉丁 caelestis) + Terrestrial (地, 拉丁 terrestris)
- 命名理据：天圆地方：中国古代宇宙观的治理隐喻。天为规（弹性标准），地为矩（刚性判定）。iCT 掌管 Schema 定义与合规验证：Schema 如天，覆盖万物；验证如地，寸步不移
- 角色名：司规 (Canon Keeper)

### 4.2 消息机 (iWW)

- 中文对：消-息
- 英文对：Waning & Waxing
- 代号：iWW
- 词源：Wane (消, 古英语 wanian：衰减) + Wax (息, 古英语 weaxan：增长)
- 命名理据：消息盈虚：消与息交替，如月之盈亏。iWW 管理收敛策略：消（收敛、衰减发散）与息（驱动、推进流程）交替进行，推动文档从 propose 走向 ratify
- 角色名：司驱 (Drive Keeper)

### 4.3 明晰机 (iCL)

- 中文对：明-晰
- 英文对：Clear & Clarify
- 代号：iCL
- 词源：Clear (明, 拉丁 clarus：清晰、明亮) + Clarify (晰, 拉丁 clarificare：使清晰)
- 命名理据：明->晰递进：先是光照（Clear，照亮结构），再是剖析（Clarify，分辨细节）。iCL 的认知过程：先照见意图与关系（明），再拆解读写修、约取形迹（晰）。Clear 是状态，Clarify 是过程：明是晰的前提，晰是明的深化
- 角色名：司判 (Judgment Keeper)

### 4.4 三机角色对照

角色详见各机分述，此处仅列对照：

| 三机         | 角色                   | 主权     |
| ------------ | ---------------------- | -------- |
| 方圆机 (iCT) | 司规 (Canon Keeper)    | 验证主权 |
| 消息机 (iWW) | 司驱 (Drive Keeper)    | 策略主权 |
| 明晰机 (iCL) | 司判 (Judgment Keeper) | 认知主权 |

## 五、诸法命名

### 5.1 收敛五法

法是从道自然生出的方法论原则。遵循则合道，违逆则违道。五法的命名遵循"动词优先"原则：法是行动原则，命名应直接传达行动的指向。

- **顺因 (Conform)**：拉丁 conformare，依从、顺应。尊重因果方向。Conform 的本义是"与形式一致"（con- + forma）：代码的形式应与其意图的形式一致。
- **有度 (Measure)**：拉丁 mensura，度量、分寸。收敛恰到好处。Measure 同时是"量度"（名词）和"把握分寸"（动词）：规约不多不少。
- **知止 (Restrain)**：拉丁 restringere，约束、勒住。知道不做什么。Restrain 不是禁止一切，而是有边界地约束：知道何时停手比知道何时动手更难。
- **损补 (Prune-Mend)**：Prune (古法语 proignier，修剪) + Mend (拉丁 emendare，修补)。损有余补不足。Prune 去掉冗余（过度规约），Mend 补全缺失（规约空白）。定向调节，不笼统。
- **顺势 (Adapt)**：拉丁 adaptare，适配、调整。力度适配场景。Adapt 的核心是"与环境匹配"：不同阶段、不同场景，收敛的力度自然适配。

### 5.2 F/G/J 力度体系

F/G/J 的命名原则是力度法（按违反后果分类），非分类法（按规则内容分类）。

| 代号 | 英文        | 含义                   |
| ---- | ----------- | ---------------------- |
| F    | Forbid (戒) | 硬约束，违反则拒绝     |
| G    | Guide (规)  | 软规范，违反则鉴行标记 |
| J    | Judge (矩)  | 精确判定，pass/fail    |

## 六、诸用命名

### 6.1 三阶生命周期

| 中文 | 英文      | 编码           | 含义                                   |
| ---- | --------- | -------------- | -------------------------------------- |
| 议   | Propose   | "1/3"          | 提案：开放、发散、可推翻               |
| 决   | Resolve   | "2/3"          | 决议：结构化、绑定、有理由             |
| 定   | Ratify    | "3/3"          | 定稿：确定、可验证、可引用             |
| 修   | Reopen    | "3/3" -> "2/3" | 重开：定稿退回决议，修补间隙后再次归一 |
| 替   | Supersede | "0"            | 归零：已被新版本取代，标注后继         |
| 弃   | Deprecate | "X"            | 废弃：概念明确弃用，无后继             |

主流程 "1/3" -> "2/3" -> "3/3" 为正向流，3/3 = 1，定稿归一。

分数设计使序数关系自明（1/3 < 2/3 < 3/3），对任何语言母语者完全平等。"0" 与 "X" 使用全球通用的零值与终止记号，不依赖自然语言命名。"0" 是轮回：旧文档权威归零，意图以新 "1/3" 重生（枯木逢春）。"X" 是终结：概念被放弃，不产生后继。

> 生命周期的完整治理规则（Reopen 条件、Supersede 取消、转换判据），见[《司衡法论》$三、生命周期治理](../specs/philosophy/On-SiHankor-Canon.sih.md#三生命周期治理)。本文仅定义命名的中英对照与编码设计。

### 6.2 三域边界

| 域       | 英文             | 权限                            |
| -------- | ---------------- | ------------------------------- |
| 治理域   | Governed Domain  | 完全管理：docs/ 内部            |
| 观察域   | Observed Domain  | 只读访问：config 声明的外部路径 |
| 不可见域 | Invisible Domain | 完全不可见：未声明的路径        |

### 6.3 约系 (Brief System)

Brief 是六层中"约"的英文，SymBrief 和 DocBrief 继承此词根。

| 约系 | 英文     | 全称                                               |
| ---- | -------- | -------------------------------------------------- |
| 符约 | SymBrief | Symbol Brief：符号层压缩（术语表、符号索引）       |
| 文约 | DocBrief | Document Brief：文档层压缩（文档摘要、治理链索引） |

SymBrief = Symbol + Brief（符号之约），DocBrief = Document + Brief（文档之约）。

## 七、体系内其他专名

### 7.1 鉴 (Assay)

- 中文：鉴
- 英文：Assay
- 词源：古法语 assai，试验、检验
- 命名理由：Assay 在冶金学中意为"测定纯度"：精确对应鉴在司衡体系中的职能：检验道层主张的真伪。Assay 不是"审计"（audit：检查合规），不是"分析"（analysis：拆解理解），而是"测定真值"

### 7.2 Spec-Coding

- 中文：规约编码
- 英文：Spec-Coding
- 定位：术层核心：将意图显式化为可验证的规范，代码从规范生成

Spec-Coding 保留连字符写法。大写 S 和大写 C 标记其作为专有术名的地位，区别于一般的 "specification-driven coding"。

## 八、命名代数学

### 8.1 代号构造公式

司衡体系中所有代号的构造遵循统一规则：

```text
代号 = [层前缀] + [英文对首字母1] + [英文对首字母2]

其中：
  层前缀：i- = Incipient (几层)
  英文对首字母：两词英文对的首字母，均大写
```

示例：

- iCT = i (Incipient) + C (Celestial) + T (Terrestrial)
- iWW = i (Incipient) + W (Waning) + W (Waxing)
- iCL = i (Incipient) + C (Clear) + L (Clarify 的第二字母，避免与 iCT 冲突)

### 8.2 消歧规则

当两词英文对的首字母相同且取两首字母导致视觉歧义时，第二词取第二个字母：

- Clear & Clarify -> iCL (而非 iCC，后者看起来像罗马数字或 typo)

首字母相同但不产生视觉歧义时，保留两首字母：Waning & Waxing -> iWW（WW 自然重复，强调消息的对称交替，无歧义）。

当首字母组合与已有代号冲突时，取最具区分力的字母组合。

### 8.3 层前缀表

| 前缀     | 层             | 含义                                                   |
| -------- | -------------- | ------------------------------------------------------ |
| i-       | Incipient (几) | 将动未动的执行层                                       |
| (无前缀) | 其他所有层     | Tao, Canon, Techne, Brief, Manifest 层实体不使用层前缀 |

## 九、不可变性规则

### 9.1 三层变更约束

| 命名层 | 变更难度 | 理由                                                       |
| ------ | -------- | ---------------------------------------------------------- |
| 代号   | 极难变更 | 在代码库、命令行、配置文件中散布，变更意味着全局重构       |
| 英文对 | 中等变更 | 在文档和国际化学术讨论中使用，变更需要同步更新翻译表和引用 |
| 中文对 | 中等变更 | 在哲学推导和文档撰写中固化，变更意味着语义断裂             |

### 9.2 不可变命名清单

以下代号已经 ratify，不可变更。ratify 在各源文档的生命周期中完成，本文件仅做汇总。

| 代号                                                   | ratify 源                       |
| ------------------------------------------------------ | ------------------------------- |
| SiHankor (sihankor)                                    | On-SiHankor ： 司衡论 (3/3)     |
| Arche (元)                                             | Arche-The-One-Above-Being (3/3) |
| Tao (道)                                               | On-SiHankor-Tao (3/3)           |
| Canon (法)                                             | On-SiHankor-Canon (3/3)         |
| Techne (术)                                            | On-SiHankor-Canon $四 (3/3)     |
| Incipient (几)                                         | On-SiHankor-Canon (3/3)         |
| Brief (约)                                             | On-SiHankor-Canon (3/3)         |
| Manifest (形迹)                                        | On-SiHankor (3/3)               |
| Assay (鉴)                                             | On-SiHankor-Assay (3/3)         |
| iCT, iWW, iCL                                          | On-SiHankor-Canon (3/3)         |
| F, G, J (Forbid, Guide, Judge)                         | On-SiHankor-Canon (3/3)         |
| Propose, Resolve, Ratify, Reopen, Supersede, Deprecate | On-SiHankor-Canon (3/3)         |
| Conform, Measure, Restrain, Prune-Mend, Adapt          | On-SiHankor-Canon $二 (3/3)     |
| Canon Keeper, Drive Keeper, Judgment Keeper            | On-SiHankor $六 (3/3)           |
| SymBrief, DocBrief                                     | On-SiHankor-Canon (3/3)         |

### 9.3 约定俗成

以下命名尚未 ratify，变更仍需充分论证：

| 命名        | 源文档                       | 当前 stage |
| ----------- | ---------------------------- | ---------- |
| Spec-Coding | SiHankor-Engineering-Mapping | 1/3        |

## 十、命名速查表

| 中文     | 英文                    | 代号                |
| -------- | ----------------------- | ------------------- |
| 司衡     | SiHankor                | sihankor            |
| 元       | Arche                   | --                  |
| 道       | Tao                     | --                  |
| 法       | Canon                   | --                  |
| 术       | Techne                  | --                  |
| 几       | Incipient               | i- (前缀)           |
| 约       | Brief                   | --                  |
| 形迹     | Manifest                | --                  |
| 鉴       | Assay                   | --                  |
| 方圆机   | Celestial & Terrestrial | iCT                 |
| 消息机   | Waning & Waxing         | iWW                 |
| 明晰机   | Clear & Clarify         | iCL                 |
| 司规     | Canon Keeper            | --                  |
| 司驱     | Drive Keeper            | --                  |
| 司判     | Judgment Keeper         | --                  |
| 顺因     | Conform                 | --                  |
| 有度     | Measure                 | --                  |
| 知止     | Restrain                | --                  |
| 损补     | Prune-Mend              | --                  |
| 顺势     | Adapt                   | --                  |
| 戒       | Forbid                  | F                   |
| 规       | Guide                   | G                   |
| 矩       | Judge                   | J                   |
| 议       | Propose                 | "1/3"               |
| 决       | Resolve                 | "2/3"               |
| 定       | Ratify                  | "3/3"               |
| 修       | Reopen                  | "2/3" (从 3/3 退回) |
| 替       | Supersede               | "0"                 |
| 弃       | Deprecate               | "X"                 |
| 符约     | SymBrief                | --                  |
| 文约     | DocBrief                | --                  |
| 治理域   | Governed Domain         | --                  |
| 观察域   | Observed Domain         | --                  |
| 不可见域 | Invisible Domain        | --                  |

## 附录

### DEPS

- 240602-0900-on-sihankor
  - 总纲：司衡全貌，六层脉络、命名体系在 $二中有系统定义
  - [司衡论](../specs/philosophy/On-SiHankor.sih.md)

### SEE-ALSO

- 240610-1030-on-sihankor-canon
  - 法论：F/G/J 命名规则、目录治理、生命周期术语定义
  - [司衡法论](../specs/philosophy/On-SiHankor-Canon.sih.md)
- 240610-1500-sihankor-document-conventions
  - 术层文档：id 格式、stage 编码、目录结构约定
  - [司衡文档约定](../specs/engineering/SiHankor-Document-Conventions.sih)
- 260613-1728-sihankor-philosophy-compendium
  - 哲学纲要：全部核心概念的权威定义与术语速查表
  - [司衡哲学纲要](../../reference/SiHankor-Philosophy-Compendium.sih.md)
