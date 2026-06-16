---
id: 260616-1330-batch1-review-fixes
stage: 3/3
upstream: 240610-1030-on-sihankor-canon
---

# 第一批审阅修复：核心文档同步已确认决策

> 第一批审阅（On-SiHankor、On-SiHankor-Canon、AGENTS.md）发现多处旧术语与旧概念未更新。根本原因：重构过程中确认的决策（废除 type、note 无 stage、id 格式变更、glossary 独立、upstream 替代符号标签）未完整回写到核心文档。本提案逐项列明变更。

## 变更清单

### B-02：id 格式统一（全项目）

**当前**：所有文档 id 使用 `YYMMDD-HHMM-`（日期与时间之间有连字符）。例：`240602-0900-on-sihankor`（修正后）。

**已确认格式**：`YYMMDDHHMM[-NNN]-语义短名`，日期与时间之间无连字符。例：`240602-0900-on-sihankor`。

**变更**：

- 所有 `.sih.md` 文件的 frontmatter `id` 字段：删除日期与时间之间的连字符
- 所有文档正文中引用的旧格式 id：同步替换为新格式
- 文件名不变（文件名中的连字符是 `-语义短名` 的分隔符，与 id 格式一致）

**影响文件**：所有 `.sih.md` 文件（约 31 个），每文件 1-5 处变更。纯机械操作，不改变语义。

### B-01：upstream 替换 PHILOSOPHY（5 文件）

**当前**：5 个哲学文档的 `upstream: PHILOSOPHY`。`PHILOSOPHY` 是符号标签，不是文档 id，engine 无法解析。

**变更**：

| 文件                                                             | 当前 upstream | 修正后 upstream         | 理由                   |
| ---------------------------------------------------------------- | ------------- | ----------------------- | ---------------------- |
| On-SiHankor.sih.md                                               | PHILOSOPHY    | 240602-0900-on-sihankor | 哲学体系根文档，自指向 |
| On-SiHankor-Tao.sih.md                                           | PHILOSOPHY    | 240602-0900-on-sihankor | 道论从总纲出           |
| On-SiHankor-Canon.sih.md                                         | PHILOSOPHY    | 240602-0900-on-sihankor | 法论从总纲出           |
| On-SiHankor-Assay.sih.md                                         | PHILOSOPHY    | 240602-0900-on-sihankor | 鉴论从总纲出           |
| docs/archive/proposals/SiHankor-Philosophy-Restructure-Plan.X.md | PHILOSOPHY    | 240602-0900-on-sihankor | archive 文档不例外     |

**额外**：同样检查 `upstream:` 为空的文档（proposals/ 中有 3 处空 upstream），为空与 PHILOSOPHY 不同，属于尚未填写，不在此次修改范围。

### B-04：AGENTS.md 删除 type 字段

**当前**：AGENTS.md §Frontmatter 写 "Mandatory fields: id, type, stage"。

**变更**：改为 "Mandatory fields: id, stage"。type 已废除，目录即身份。

### B-03：Canon 移除 note stage 定义（内容改动最大）

**当前**：Canon §3.2 状态定义表、§6.2 knowledge/notes/ 中 note 有完整 stage 生命周期：1/3→2/3→3/3→0/decayed→X。另有 note 晋升机制（引用计数≥3 且跨≥2 目录建议晋升）。

**已确认决策**：notes 无 stage，有 id 和 verified 衰变。给 notes 加 stage 是把沉淀变成了治理对象。

**变更**：

1. §3.2 状态定义表：删除 note 行（原文 nature 映射表中 note 行）
2. §3.2 正文 "spec/proposal/decision/reference 的 stage 回答……note 的 stage 回答……" 段落：删除 note 子句
3. §6.2 knowledge/notes/ 行：`stage` 行改为 "-（无 stage）"，或整行删除
4. §6.2 note 的晋升机制（1/3→2/3→3/3 晋升、引用计数触发、人类确认等）：重写为身份变更模型
5. §6.2 衰减机制（`0/decayed`）：重新定义为 verified 过期（`verified: YYMMDD` 超过阈值自动标记衰退）

**note 的新模型**：

- note 有 id，无 stage
- note 有 `verified` 字段：记录洞察被确认的日期
- 超过 `review_after_days`（在 config.yml 中定义）未更新 verified：engine 标记衰退警告
- note 的洞察成熟时：人创建新的 spec/proposal/decision 文档（有 stage），note 本身不变——不是晋升，是身份变更

### D-01：Canon §2.1 区分 upstream 与 resolve_ref

**当前**：Canon §2.1 "顺因的具象"下并列出现 "upstream 溯源"（frontmatter 字段，治理授权链）和 "resolve_ref 路径"（compendium 内容追溯，术层实现细节）。两者概念不同但放在同一列表项中，容易混淆。

**变更**：将 "文档归纳合成" 从顺因具象列表中拆出，独立为一段。在其上下文中明确 resolve_ref 是术层实现，不是独立的法层概念。

### D-02：On-SiHankor §7 重写过时内容

**当前**：§7.1 提到 `.po 翻译流水线`、`i18n/translation.en.yml`、`LLM 自动化`；§7.2 提到 `sih-docs/`、`scope.yaml`、`单向不可逆`。

**已确认**：po/ 已删除，i18n/ 已废除，sih-docs 不是当前目录名，scope.yaml 是旧概念，单向不可逆已被 Reopen/Supersede 修正，翻译模型已改为 glossary 三层模型。

**变更**：§7 全节重写。内容对齐当前工程状态：

- §7.1 文档治理：glossary 三层模型（zh.yml→en.yml 跨语言映射）
- §7.2 生命周期：Reopen/Supersede 修正模型
- §7.3 F/G/J 法则：保持（无过时内容）

### D-04：glossary 移至项目根级

**当前**：`docs/glossary/zh.yml` + `docs/glossary/en.yml`。

**已确认**：glossary 是与 docs/ 同级的一等模块，不是 docs/ 的子目录。

**变更**：

- 创建 `glossary/` 于项目根
- 移动 `docs/glossary/zh.yml` → `glossary/zh.yml`
- 移动 `docs/glossary/en.yml` → `glossary/en.yml`
- 删除 `docs/glossary/`
- Canon §6.4 路径描述同步更新
- Canon §6.3 config.yml 示例中 `glossary: docs/glossary/` → `glossary: glossary/`

### D-06：Canon §6.4 显式声明 zh→en 权威方向

**当前**：§6.4 已有方向声明（"glossary 变更不反向影响 reference"），但 zh.yml→en.yml 的跨语言权威方向未显式声明。

**变更**：在 §6.4 "因果方向"段落后追加一句："zh.yml 的 definition 是跨语言映射的语义权威源，en.yml 不重新定义概念——en.yml 只提供翻译映射与歧义消解。"

## S 级变更

### S-01：On-SiHankor §1.3 "metadata 文件夹" → 当前目录引用

### S-02：On-SiHankor §2.1 Mermaid 图文本加连字符（"发散自然，收敛必为" → "发散自-然，收敛必-为"）

### S-03：AGENTS.md 补全内容

当前 AGENTS.md 只有格式规则，缺少：

- 身份约束（AGENTS.md 对 agent 的约束性质）
- 判断链（agent 如何判断一个操作是否合道）
- 边界声明（agent 能做什么、不能做什么）
- 目录结构定义（docs/ 下各目录的语义）
- frontmatter 字段定义（id/stage/upstream 的含义和约束）
- 生命周期规则摘要

此项依赖之前设计的项目级提示词框架。在框架未恢复前，先补最小集：frontmatter 字段定义（更正 type→删除）、目录结构一览。

### S-04：archive 命名约定明确化

Canon §6.2 当前写 "stage X 的文档迁移至 `docs/archive/{原目录}/{name}.X.md`"。但实际 archive/ 下只有一个文件，不带原目录子分类。且 `.X.md` 后缀是文件名变更——id 不可变原则下，文件名变更与 id 不变之间的关系需要更明确的约定。

**变更**：在 Canon §6.2 archive 段补一句："归档时文件名追加 `.X` 后缀仅用于人类浏览识别废弃状态；文档 id 不变，engine 按 stage 字段判断，不依赖文件名。"

### S-05：note 晋升重新定义为身份变更

依存于 B-03。B-03 执行后自然消除此问题。

## 执行顺序

1. **B-02**（id 格式，机械操作但波及最广）→ 先做，后续所有引用新 id
2. **B-01**（upstream 替换，依赖 B-02 后的新 id）
3. **B-04**（AGENTS.md 删 type，单文件微调）
4. **B-03**（note stage，Canon 最大文本改动）
5. **D 级 + S 级**（内容修正，在核心结构稳定后执行）

## 影响范围

- **波及文件数**：约 31 个 .sih.md + AGENTS.md + 2 个 yml
- **最大单文件改动**：On-SiHankor-Canon.sih.md（§3.2 + §6.2 note 相关内容重写）
- **最大机械操作**：B-02 id 连字符删除（每文件 1-5 处，全项目约 80-120 处）
- **风险**：B-02 与 B-01 的 id 引用替换需精确匹配，遗漏一处会导致引用断裂

## 执行记录

### 执行验证（2026-06-16）

所有变更已执行并验证通过：

| 编号 | 状态 | 验证方式                                                                                     |
| ---- | ---- | -------------------------------------------------------------------------------------------- |
| B-02 | 完成 | 全项目 grep：零旧格式 id 残留                                                                |
| B-01 | 完成 | 全项目 grep：零 PHILOSOPHY upstream 残留，5 文件已修正                                       |
| B-04 | 完成 | AGENTS.md frontmatter 字段表已删除 type                                                      |
| B-03 | 完成 | Canon §3.2 + §6.2 重写，Document-Conventions §4.6 同步，3 个 note frontmatter stage→verified |
| D-01 | 完成 | Canon §2.1 upstream/resolve_ref 已拆分                                                       |
| D-02 | 完成 | On-SiHankor §7 已重写为 glossary 三层模型 + Reopen/Supersede                                 |
| D-04 | 完成 | glossary/ 已移至项目根级，所有路径引用已更新                                                 |
| D-06 | 完成 | Canon §6.4 已追加 zh→en 权威方向声明                                                         |
| S-01 | 完成 | "metadata 文件夹" → `docs/specs/philosophy/`                                                 |
| S-02 | 完成 | 全部 6 处 "发散自然，收敛必为" → "发散自-然，收敛必-为"                                      |
| S-03 | 完成 | AGENTS.md 已追加字段定义和目录结构一览                                                       |
| S-04 | 完成 | Canon §6.2 archive 段已追加 id 不变声明                                                      |
| S-05 | 完成 | 随 B-03 自然消除                                                                             |

变更量：约 33 个文件，约 150+ 处修改。

## ADR

### 背景

第一批审阅发现 On-SiHankor、On-SiHankor-Canon、AGENTS.md 中存在大量与已确认决策不一致的内容。根本原因是重构过程中的决策（废除 type、note 无 stage、id 格式、upstream 机制、glossary 独立）未完整回写到核心文档。

### 决策

按上述变更清单逐项修正。执行顺序按 id 格式→upstream→AGENTS.md→note stage→D/S 级。

### 后果

- 正向：核心文档与已确认决策一致，后续审阅基于正确基线
- 风险：B-02 id 格式变更为全项目机械替换，遗漏一处会导致引用断裂。必须逐文件验证
