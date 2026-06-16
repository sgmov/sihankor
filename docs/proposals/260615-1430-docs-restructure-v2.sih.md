---
id: 2606151430-docs-restructure-v2
stage: 3/3
upstream: 2406101030-on-sihankor-canon
---

# 司衡文档体系重构 v2

> 基于道层重新推导 docs/ 目录结构、frontmatter 字段与治理流程的完整方案。替代 2406110100-sihankor-philosophy-restructure（旧重构计划，stage 1/3）。

## 一、背景与间隙证据

### 1.1 当前体系的四个结构性问题

**问题一：双信号系统。** type 字段（frontmatter）和目录（文件系统）同时声明文档身份。两份信号可以不一致——Canon $6.3 明确允许"文档可以不在 type 对应的目录中"。人对目录形成第一印象，打开文件后读到 type，两个信号冲突时需自行判断。道三：信号自晦，恢复意图的成本转嫁给人类。

**问题二：stage 承载三层语义。** 当前 stage 同时表达文档成熟度、治理流程位置、下游引用权限。三层语义混在一个字段中，导致 `1/3 → 2/3 → 3/3` 既表达信心增长又表达流程推进又表达可用性提升。对 note 而言，`1/3` 复用 `n/3` 编码但语义完全不同（生命周期 vs 可信度）。

**问题三：type 定义不一致。** Canon $3.2 定义 4 种 type，Document-Conventions $4.3 扩展为 7 种。SiHankor-Type-Extension 决策（2606140000）记录了这是实现先于规约暴露的间隙，但修复方向是"扩展 type"——在根因上叠加而非修正。

**问题四：治理元层与被治理内容混放。** brainstorming 产物、notes、specs 共享同一套目录浏览流。人在 `notes/` 或 `specs/philosophy/` 中无法快速区分"这是构思碎片""这是实践洞察""这是系统规范"。

### 1.2 道层推导

**道二（意图先于代码）**：文档的身份是意图——它要做什么（规范、提案、决策、参照、沉淀）。目录是文档意图的第一信号。人类打开文件之前先看到目录和文件名。

**道三（代码自晦，意图必复）**：不应有两个信号说同一件事。目录即身份，打开文件只是确认，不是发现。

**有度**：五目录恰好覆盖五种文档意图，type 字段是冗余。冗余的字段增加维护成本，不增加区分力。

**知止**：不穷举目录内的子分类。不在文件名中强制编码身份信息。引擎只读路径第一层语义。

## 二、决策

### 2.1 目录即身份——废除 type 字段

| 目录               | nature（引擎推断） | 中文     | 定义                                                       |
| ------------------ | ------------------ | -------- | ---------------------------------------------------------- |
| `specs/`           | spec               | 系统定义 | 回答"系统是什么"。意图草案（1/3）和正式规约（3/3）同在此处 |
| `proposals/`       | proposal           | 变更提议 | 回答"我们提议改变什么"。指向被变更的 spec 或 decision      |
| `decisions/`       | decision           | 决策记录 | 回答"为什么这样选"。源于 proposal 的决议                   |
| `reference/`       | reference          | 参照标准 | 回答"术语/概念精确指什么"。供查阅，不求证                  |
| `knowledge/notes/` | note               | 实践洞察 | 回答"我们学到了什么"。从工程实践中提炼                     |

引擎从路径推断 nature：`docs/{第一层目录}/**` → nature = 目录名。`knowledge/notes/` 是特例——nature 为 note，由第二层目录确定。

### 2.2 stage 语义按 nature 分化

`stage` 字段保留，编码形式统一（1/3, 2/3, 3/3, 0, X），语义由 nature 决定：

| nature    | 1/3              | 2/3                | 3/3          | 0        | X    |
| --------- | ---------------- | ------------------ | ------------ | -------- | ---- |
| spec      | 起草中，不可引用 | 审查中，可谨慎引用 | 定稿，可依赖 | 已被替代 | 废弃 |
| proposal  | 提案中           | 决议中             | 已决议       | 已被替代 | 废弃 |
| decision  | 草拟             | 审查中             | 定稿         | 已被替代 | 废弃 |
| reference | 起草中           | 审查中             | 定稿         | 已被替代 | 废弃 |
| note      | 草稿             | 活跃               | 已晋升       | 已衰减   | 废弃 |

- spec/proposal/decision/reference 的 stage = **治理可信度**。回答"我能信这份文档吗？"
- note 的 stage = **生命周期**。回答"这份洞察处于什么阶段？"
- 编码复用 `n/3`，但语义由 nature 确定，无歧义

### 2.3 目录结构

```tree
docs/
  specs/                              # 被治理侧：系统定义
    philosophy/                       # 司衡的系统恰好是哲学
      On-SiHankor.sih.md
      On-SiHankor-Tao.sih.md
      On-SiHankor-Assay.sih.md
      On-SiHankor-Canon.sih.md
      Arche-The-One-Above-Being.sih.md
      SiHankor-Philosophy-Arguments.sih.md
    engineering/
      SiHankor-Engineering-Mapping.sih.md
      SiHankor-Mind-Design.sih.md
      SiHankor-Engine-Design-Summary.sih.md
      SiHankor-Document-Conventions.sih.md
  proposals/                          # 治理侧：变更提议
    2606151430-docs-restructure-v2.sih.md
  decisions/                          # 治理侧：决策记录
    SiHankor-External-Validation.sih.md
    SiHankor-Legacy-Migration-Governance.sih.md
    SiHankor-Type-Extension.sih.md
  reference/                          # 被治理侧：参照标准
    SiHankor-Philosophy-Compendium.sih.md
    SiHankor-Onomastic-Philosophy.sih.md
  knowledge/                          # 未规约化的集体知识
    drafts/                           # 构思碎片（非 .sih.md）
    notes/                            # 实践洞察（.sih.md）
      Format-Violations.sih.md
      SiHankor-Doc-Migration-Patterns.sih.md
  archive/                            # 废弃文档
  glossary/                           # 可选：跨人类语言翻译
    zh.yml
    en.yml
```

### 2.4 frontmatter 精简

删除 `type` 字段。必填字段：

```yaml
id: 2406020900-on-sihankor
stage: 3/3
upstream: <文档id>       # note 可选，其余必填
successor: <文档id>      # 仅 stage 为 0 时出现
```

### 2.5 glossary 定位修正

**glossary/**：跨人类语言的语义精确传递。中文（源语言）→ 英文（工程通用语）。可选——无多语言需求的项目不创建此目录。

`zh.yml` 中每个条目声明 `derives-from`，指向 reference/ 中的文档 id。engine 通过 `zh.yml → derives-from → reference/` 做 join，不需要 `_concepts.yml`。

**semantic.yml**：意图与代码之间的语义翻译引擎。排放于 `.sih/semantic.yml`。每个项目都需要（道四不可逃）。engine 维护——iCL 用语义映射做意图恢复，iCT 用 semantic 做间隙检测。

### 2.6 删除项

| 项目                                                         | 理由                                         |
| ------------------------------------------------------------ | -------------------------------------------- |
| `docs/plan/` 目录                                            | 不在五目录模型中。plan 型文档放入 proposals/ |
| `docs/notes/` 目录                                           | 迁移到 knowledge/notes/                      |
| `docs/glossary/_concepts.yml`                                | 概念注册是 reference/ 的事                   |
| `docs/glossary/po/` 全部文件                                 | 无 CLI 国际化需求                            |
| `docs/proposals/SiHankor-Philosophy-Restructure-Plan.sih.md` | 被本计划替代，推进到 X                       |

### 2.7 文件移动

| 文件                                     | 从                       | 到                      |
| ---------------------------------------- | ------------------------ | ----------------------- |
| `SiHankor-Philosophy-Compendium.sih.md`  | `docs/specs/philosophy/` | `docs/reference/`       |
| `SiHankor-Onomastic-Philosophy.sih.md`   | `docs/specs/philosophy/` | `docs/reference/`       |
| `SiHankor-Doc-Migration-Patterns.sih.md` | `docs/reference/`        | `docs/knowledge/notes/` |
| `SiHankor-Engine-Roadmap.sih.md`         | `docs/plan/`             | `docs/proposals/`       |
| `Format-Violations.sih.md`               | `docs/notes/`            | `docs/knowledge/notes/` |

### 2.8 新建文档

| 文档            | 位置                                    | 内容                                  |
| --------------- | --------------------------------------- | ------------------------------------- |
| SiHankor-README | `docs/reference/SiHankor-README.sih.md` | 新用户入口：一句话定义 + 三条阅读路径 |

## 三、执行顺序

按顺因之法（法层→术层→形迹层）：

1. Canon Reopen：3/3 → 2/3，附间隙证据
2. 修改 Canon：废除 type 定义（$3.2），目录即身份（$6），stage 语义表（新增），glossary/semantic 定位修正
3. 更新 Document-Conventions：同步 frontmatter 字段、目录结构
4. 文件物理移动
5. 批量更新交叉引用路径
6. 新建 reference/SiHankor-README.sih.md
7. 旧 restructure plan → X
8. Canon → 3/3，ADR

## 四、道四声明

本计划当前认识边界：

- semantic.yml 的完整 schema 和 engine 索引实现不在本次重构范围内
- knowledge/drafts/ 中的 brainstorming 产物格式和与 .sih.md 的转化流程在后续细化
- 交叉引用更新可能遗漏部分非标准格式的引用（如纯英文书名引用）

## 五、ADR

### 背景

基于道层系统性重新推导 docs/ 目录结构与 frontmatter 设计，消除双信号系统（type+目录）、stage 语义混淆、治理元层与被治理内容混放等结构性问题。

### 决策

废除 type 字段，目录即身份。stage 语义按 nature 分化。引入 knowledge/ 和 archive/。glossary 定位为纯跨人类语言翻译。semantic.yml 为引擎意图↔代码映射。删除 _concepts.yml、po/、plan/、notes/（迁移后）。

### 后果

- 正向：消除双信号，前端 frontmatter 字段从 5 个减为 4 个，人类浏览路径单一无歧义
- 正向：为后续引擎实现 semantic 索引和间隙检测提供明确的规约基础
- 负向：Canon $3.2 和 $6 需实质性修改，需 Reopen 流程
- 负向：交叉引用批量更新存在遗漏风险

decided-by: ai-assist
