---
id: 260616-1600-docs-system-review
stage: 3/3
upstream: 260615-1500-docs-restructure-v2-decision
---

# 司衡文档体系完整审阅报告（合并修正版）

> 审阅时间：2026-06-16 16:00
> 审阅范围：docs/ 全部 .sih.md、glossary/、.sih/、src/core/、AGENTS.md
> 审阅基准：13 条已确认决策
> 说明：本报告合并自两份独立审阅报告，已剔除经代码/文件核实不成立的发现项，仅保留验证通过的条目。

---

## 一、审阅方法

以 13 项已确认决策为审计基线，逐一检查所有审计范围内文件，按三级分类：

| 级别          | 含义               | 判定标准                                         |
| ------------- | ------------------ | ------------------------------------------------ |
| **B（阻塞）** | 直接违背已确认决策 | 文件内容与决策规则矛盾，必须修正                 |
| **D（偏离）** | 不一致但非直接冲突 | 风格漂移、术语不统一、遗漏更新、代码与规约不一致 |
| **S（建议）** | 可做可不做的改进   | 增强可读性、降低歧义、补充缺失支持               |

**豁免规则**：

- 历史性提及已废除术语（如讨论废除过程本身）不视为违规
- 已归档（stage X）或已替代（stage 0/）文档可保留旧术语
- decisions/ 目录的 `decided-by` 字段为合法字段

---

## 二、13 项已确认决策

| #   | 决策                             | 核心内容                                                                                                                       | 来源                                       |
| --- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------ |
| 1   | 目录即身份                       | 文档 nature 由目录唯一确定，`type` 字段废除                                                                                    | restructure-v2 decision                    |
| 2   | type 字段废除                    | frontmatter 不得包含 `type`                                                                                                    | restructure-v2 decision                    |
| 3   | stage 语义按 nature 分化         | spec/proposal/decision/reference：stage = 治理可信度；**note：stage = 生命周期**（1/3, 2/3, 3/3, 0, X）                        | restructure-v2 decision                    |
| 4   | frontmatter 字段精简             | 必填：id、stage；spec/proposal/decision/reference 必填 upstream（note 可选）；decided-by 仅 decisions/；stage 0 时含 successor | restructure-v2 decision                    |
| 5   | upstream 自指向                  | 根文档 upstream 指向自身 id，不用大写域标识（如 PHILOSOPHY）                                                                   | restructure-v2 decision                    |
| 6   | decided-by 限于 decisions/       | 其他目录 frontmatter 不得有 decided-by                                                                                         | decided-by-placement decision              |
| 7   | _concepts.yml 废除               | glossary 不再需要此文件                                                                                                        | restructure-v2 decision                    |
| 8   | glossary 置于项目根              | 与 docs/ 同级，不在 docs/ 下                                                                                                   | restructure-v2 decision                    |
| 9   | 通过 upstream 链溯源             | 无 resolve_ref                                                                                                                 | restructure-v2 decision                    |
| 10  | scope.yaml / sih-docs / po/ 废除 | 这些目录/文件不再存在                                                                                                          | restructure-v2 decision                    |
| 11  | "单向不可逆"替换为修正模型       | 不使用"单向不可逆"作为现行术语（历史分析语境除外）                                                                             | Canon $3.1                                 |
| 12  | knowledge/ 拆分                  | drafts/（非 .sih.md）和 notes/（.sih.md，nature=note）                                                                         | restructure-v2 decision                    |
| 13  | note 2/3 推进需人类确认          | 非自动晋升                                                                                                                     | External-Validation decision               |
| —   | id 格式                          | **`YYMMDD-HHMM[-NNN]-语义短名`**（强制连字符）。此为决策漂移结果，以 validator.rs 正则为准                                     | format-hyphen-drift note + batch1 proposal |

> **关键更新**：决策 #3 确认 note 有 stage，语义为生命周期（与 spec/proposal/decision/reference 的"可信度"语义不同，但编码复用 `1/3, 2/3, 3/3, 0, X`）。此定义来自 `260615-1500-docs-restructure-v2-decision`（stage 3/3），覆盖 Canon $5.3 的旧表述。AGENTS.md 中"Notes have no stage"是历次审阅修复的遗漏，属过时定义。
> **id 格式漂移**：batch1 提案曾统一为无连字符格式 `YYMMDDHHMM`，但后续决策漂移回强制连字符格式 `YYMMDD-HHMM`。当前 validator.rs 正则 `^\d{6}-\d{4}(-\d{3})?-.+$` 为权威标准。Document-Conventions $2.1 定义和 AGENTS.md 的 id 格式说明需同步更新。

---

## 三、B 级发现（阻塞）

### B-01 | specs/ 文档 ADR 尾部残留 `decided-by: ai-assist`

**文件**（6 个）：

- `docs/specs/philosophy/On-SiHankor.sih.md`（行410）
- `docs/specs/philosophy/On-SiHankor-Canon.sih.md`（行680）
- `docs/specs/philosophy/On-SiHankor-Tao.sih.md`（行490）
- `docs/specs/philosophy/Arche-The-One-Above-Being.sih.md`（行400）
- `docs/specs/philosophy/On-SiHankor-Assay.sih.md`（行809）
- `docs/specs/engineering/SiHankor-Document-Conventions.sih.md`（行572）

**断言**：6 个 specs/ 文档的 ADR 附录尾部包含 `decided-by: ai-assist`。按决策 #6（decided-by 仅限 decisions/ 目录），此类字段不应以 frontmatter 字段格式存在于正文中。Document-Conventions 自身 $4.4 定义了"非 decisions/ 不声明 decided-by"却自相矛盾。

**修复**：删除 `decided-by: ai-assist` 行，替换为 prose 叙事表述（如 `> 本附录的设计决策由 AI 辅助生成，人类审核确认。`）。详见 S-03。

---

### B-02 | proposals/ 文档正文末尾残留 `decided-by: ai-assist`

**文件**（2 个）：

- `docs/proposals/260616-1330-batch1-review-fixes.sih.md`（行192）
- `docs/proposals/260615-1430-docs-restructure-v2.sih.md`（行183）

**断言**：2 个 proposals/ 文档在 frontmatter `---` 闭合标记之外的正文末尾包含 `decided-by: ai-assist`。按决策 #6，decided-by 仅限 decisions/ 目录。

**修复**：删除正文末尾的 `decided-by: ai-assist` 行。

---

### B-03 | `260616-1400-batch3-review-fixes.sih.md` upstream 引用断裂

**文件**：`docs/proposals/260616-1400-batch3-review-fixes.sih.md`

**断言**：frontmatter `upstream: 240610-1030-on-sihankor-canon`。此引用使用了带连字符的 id 格式，但 Canon 文档实际 id 为 `240610-1030-on-sihankor-canon`（旧格式无连字符）。两种格式不匹配，引用无法解析。此问题是全局 id 格式漂移（见决策 #id 格式）的个案表现：Canon 的 id 需随全局统一更新为 `240610-1030-on-sihankor-canon`，届时此 upstream 引用自然修复。

**修复**：在全局 id 格式统一修正中，将 Canon id 更新为 `240610-1030-on-sihankor-canon`（含连字符），同步更新所有引用 Canon id 的 upstream 字段。

---

### B-04 | AGENTS.md note stage 定义与决策链矛盾

**文件**：`AGENTS.md`（行40 Field Definitions 表、行50 Directory Structure 表）

**断言**：AGENTS.md 定义 note 的 stage 为"none (verified only)"，但决策 #3（来自 `260615-1500-docs-restructure-v2-decision`，stage 3/3）明确定义 note 的 stage 表达生命周期（编码复用 `1/3, 2/3, 3/3, 0, X`）。AGENTS.md 作为项目规则入口文件，其定义与 3/3 决策矛盾，将导致后续文档和代码实现产生歧义。此为历次审阅修复的遗漏。

**修复**：将 AGENTS.md 中 note stage 的定义统一为"note 有 stage，语义为生命周期"，与决策 #3 一致。

---

### B-05 | `SiHankor-Engine-Roadmap.sih.md` 引用已废除的 po/ 目录

**文件**：`docs/proposals/SiHankor-Engine-Roadmap.sih.md`（行403-404）

**断言**：Roadmap 提到 `glossary/po/messages.pot` 和 `glossary/po/en.po` 作为未来计划。按决策 #10，po/ 已废除。

**修复**：从 roadmap 中移除 po/ 相关计划。

---

### B-06 | `SiHankor-README.sih.md` glossary 位置错误

**文件**：`docs/reference/SiHankor-README.sih.md`（行51-67 目录树）

**断言**：README 的目录概览树中 `glossary/` 显示在 `docs/` 下。按决策 #8，glossary 应位于项目根（与 docs/ 同级）。README 作为新用户入口文档，此错误影响最大。

**修复**：将 `glossary/` 移出 docs/ 子树，与 `.sih/` 同级置于项目根。

---

### B-07 | `src/core/models.rs` Frontmatter 包含 `decided_by` 通用字段

**文件**：`src/core/models.rs`（行74 `pub decided_by: Option<String>`）

**断言**：Frontmatter 作为所有文档共用的结构体，不应包含 `decided_by` 字段。按决策 #6，decided-by 仅在 decisions/ 文档中合法。此字段应在 Document 或 validator 层按 nature 条件处理，而非作为通用字段。

**修复**：从 Frontmatter 结构体中移除 `decided_by`，改为在 Document 结构体或独立的 DecisionMetadata 中按 nature 条件携带。

---

### B-08 | `src/core/validator.rs` 缺少 decided-by 位置反向校验

**文件**：`src/core/validator.rs`

**断言**：validator 仅检查 decisions/ 文档在 2/3 和 3/3 时应有 decided-by（G-09），以及 ai-auto 禁止（F-06），但缺少反向校验：非 decisions/ 目录文档的 frontmatter 中出现 decided-by 应标记为违规。当前 models.rs 中 Frontmatter 包含通用 `decided_by` 字段（见 B-07），validator 也未对此进行约束，形成双重漏洞。

**修复**：添加校验规则：当文档 nature 非 decision 时，frontmatter 含 `decided_by` 或 `decided-by` 字段应产出 Fatal 级违规。

---

## 四、D 级发现（偏离）

### D-01 | id 格式连字符漂移 — 全局性不一致

**涉及文件**：约 20 个 .sih.md 文件

**断言**：batch1 提案曾统一为无连字符 `YYMMDDHHMM`，但后续新创建的提案/决策文档漂移回有连字符 `YYMMDD-HHMM`。当前决策标准为强制连字符（以 validator.rs 正则为准），但大量旧文档（所有 philosophy 文档、batch1 提案等）仍为无连字符格式。Document-Conventions $2.1 和 AGENTS.md 的 id 格式定义也未更新。

**涉及文件分类**：

- 无连字符（需添加）：`260616-1330-batch1-review-fixes`、`260615-1430-docs-restructure-v2`、`260613-1728-sihankor-engineering-mapping`、所有 philosophy specs 文档、`240610-1030-on-sihankor-canon`、所有 decisions/ 早期文档、4 个 knowledge/notes/ 文件
- 有连字符（已正确）：`260616-1400-batch3-review-fixes`、`260615-1530-decided-by-placement`、`260616-1200-engine-dev-governance-chain`、`260616-1210-post-restructure-doc-cleanup`、`260616-1214-engine-mvp-parser`、`260616-1214-gap-entity-definition`

**修复**：

1. 在 Document-Conventions 中正式确认强制连字符格式
2. 更新 AGENTS.md 的 id 格式定义
3. 全局批量修正所有无连字符 id（添加连字符）
4. 同步更新所有 upstream 引用

---

### D-02 | 多个文档称"note 无 stage"，与决策 #3 矛盾

**文件**（3 个）：

- `docs/specs/philosophy/On-SiHankor-Canon.sih.md`（行191, 539, 543）
- `docs/specs/engineering/SiHankor-Document-Conventions.sih.md`（行170）
- `docs/reference/SiHankor-Philosophy-Compendium.sih.md`（行537）

**断言**：上述文档明确写"note 无 stage"，与决策 #3（note 有 stage，语义为生命周期）矛盾。此问题与 B-04 同源：AGENTS.md 的过时定义导致文档层表述混乱。Canon 作为 stage 3/3 权威文档，$5.3 的"note 无 stage"表述已被 `260615-1500-docs-restructure-v2-decision` 覆盖，但尚未 Reopen 修正。

**修复**：待 B-04（AGENTS.md）统一后，对上述 3 个文档启动 Reopen 流程修正。Canon 的修正优先级最高。

---

### D-03 | "单向不可逆"术语在多个文档中残留

**文件**（3 个）：

- `docs/specs/philosophy/On-SiHankor.sih.md`（行312）："因果单向不可逆"
- `docs/specs/philosophy/SiHankor-Philosophy-Arguments.sih.md`（行289, 298, 306）：衍生原则表中仍列"单向不可逆"
- `docs/specs/engineering/SiHankor-Engineering-Mapping.sih.md`（行167-168）："修正通过叠加而非覆盖"与 Canon 已修正模型矛盾

**断言**：决策 #11 明确"单向不可逆"已被修正模型取代。上述文档中以不同形式残留：Philosophy-Arguments 将"单向不可逆"列为衍生原则名称（规范性使用）；On-SiHankor 用"因果单向不可逆"描述 glossary 治理（当前描述）；Engineering-Mapping $6.3 描述与 Canon $3.1 修正模型不一致。Canon $3.1 自身使用"单向不可逆"仅为历史分析语境，属豁免范围。

**修复**：

- Philosophy-Arguments：将"单向不可逆"标注为已整合入修正模型，更新衍生原则表
- On-SiHankor：将"因果单向不可逆"改为"因果方向不可逆"或直接引用修正模型
- Engineering-Mapping：更新 $6.3 描述，与 Canon 修正模型一致

---

### D-04 | `SiHankor-Philosophy-Compendium.sih.md` Supersede 编码缺少斜杠

**文件**：`docs/reference/SiHankor-Philosophy-Compendium.sih.md`（行388-389）

**断言**：状态编码表中 Supersede 行编码列为 `"0"` 而非 `"0/"`。其他文档（Canon、Conventions）均使用 `0/` 格式（如 `0/new-id`）。

**修复**：将编码列改为 `"0/"`。

---

### D-05 | `SiHankor-Legacy-Migration-Governance.sih.md` 引用已废除的 po/ 路径

**文件**：`docs/decisions/SiHankor-Legacy-Migration-Governance.sih.md`（行133）

**断言**：执行记录中提到 `docs/glossary/po/README.md`。按决策 #10，po/ 已废除。虽为历史记录，但作为 3/3 决策文档，保留对已废除路径的引用可能误导读者。

**修复**：添加注释说明 po/ 已在后续重构中废除。

---

### D-06 | `SiHankor-External-Validation.sih.md` 应迁移为 note

**文件**：`docs/decisions/SiHankor-External-Validation.sih.md`

**断言**：当前 stage 为 2/3，位于 decisions/ 目录。但其内容本质是实践洞察——记录外部验证过程、发现的间隙、与构建者确认的结果——而非治理决策。其核心结论（note 晋升机制等）已被后续 3/3 决策吸收。作为 decision 滞留在此目录中，会给读者"此决策仍未定稿"的误导。

**修复**：迁移至 `docs/knowledge/notes/`，nature 改为 note，stage 设为 `3/3`（洞察已确认有效），添加 verified 字段。

---

---

### D-08 | 4 个 note 文件缺少 stage 字段

**文件**：

- `docs/knowledge/notes/260616-1350-id-format-hyphen-drift.sih.md`
- `docs/knowledge/notes/Format-Violations.sih.md`
- `docs/knowledge/notes/260616-1216-id-timestamp-automation.sih.md`
- `docs/knowledge/notes/SiHankor-Doc-Migration-Patterns.sih.md`

**断言**：4 个 note 文件仅有 id 和 verified，无 stage 字段。按决策 #3（note 有 stage，语义为生命周期），全部缺少 stage。

**修复**：为上述 4 个文件添加 stage 字段（建议初始值为 `1/3`）。

---

### D-09 | 3 个 note 文件 id 格式缺少连字符

**文件**（3 个）：

- `docs/knowledge/notes/Format-Violations.sih.md`：id `240611-0115-format-violations`
- `docs/knowledge/notes/260616-1216-id-timestamp-automation.sih.md`：id `260616-1216-id-timestamp-automation`
- `docs/knowledge/notes/SiHankor-Doc-Migration-Patterns.sih.md`：id `260613-1728-sihankor-doc-migration-patterns`

**断言**：按 id 格式强制连字符标准（见决策 #id 格式），日期与时间之间应有连字符。上述 3 个 id 均缺少连字符。

**修复**：修正 id 格式，添加连字符。与 D-01 全局修正同步执行。

---

### D-10 | Rust 代码模型与 successor 编码方式一致（确认无偏离）

**文件**：

- `src/core/models.rs`：Frontmatter 无 successor 字段
- `src/core/parser.rs`：parse_frontmatter() 未提取 successor 字段

**断言**：Canon 行 299 明确"0/ 文档的 stage 值直接编码了后继文档 ID（如 0/240610-1030-new-doc），无需单独的 successor 字段"。代码模型当前实现与此一致：successor 编码在 stage 值中，Frontmatter 无需独立 successor 字段。此非偏离，而是正确实现。

**建议**：validator 可增加校验：当 stage 值为 `0/xxx` 格式时，验证 `/` 之后的 successor id 是否可解析（格式合法 + 对应文档存在）。此为增强项，非修复项。

---

### D-11 | validator.rs 未实现根文档 upstream 自指向检查

**文件**：`src/core/validator.rs`（行148）

**断言**：决策 #5 要求"根文档 upstream 指向自身 id"。validator.rs 中仅有注释"root docs 应自指向自身 id"，无实际检查代码。

**修复**：在 validate_governance() 中添加根文档自指向验证逻辑。

---

### D-12 | AGENTS.md Field Definitions 缺少 decided-by 和 successor 说明

**文件**：`AGENTS.md`

**断言**：AGENTS.md 的 Field Definitions 表缺少：

- `decided-by`：决策 #6 定义此字段仅限 decisions/ 目录
- `successor`：Canon 行 299 明确 successor 编码在 stage 值中（格式 `0/<successor-id>`），非独立 frontmatter 字段。AGENTS.md 的 stage 字段描述中 `0/<successor-id>` 已暗示此编码方式，但未在 Field Definitions 中显式说明

**修复**：在 Field Definitions 表中补充 decided-by 定义；在 stage 字段的 Description 中显式说明"stage 值为 `0/<successor-id>` 时，successor id 直接编码在 stage 值中，无需独立 successor 字段"。

---

### D-13 | `SiHankor-Mind-Design.sih.md` 道二约束表中 `type` 残留

**文件**：`docs/specs/engineering/SiHankor-Mind-Design.sih.md`（行41）

**断言**：道二约束表中写"分析必须先理解意图元数据（type/stage/upstream）"。按决策 #1 和 #2，type 已废除。

**修复**：改为 `nature/stage/upstream`，并附带定义：nature 由引擎从文档所在目录推断（specs/ → spec，proposals/ → proposal，decisions/ → decision，reference/ → reference，knowledge/notes/ → note）。读者理解 nature 的含义无需查询 type 字段——目录即身份。

---

### D-14 | validator.rs id 格式正则与 Document-Conventions 矛盾

**文件**：`src/core/validator.rs`（行362）、`docs/specs/engineering/SiHankor-Document-Conventions.sih.md`（$2.1）

**断言**：validator 正则 `^\d{6}-\d{4}` 要求连字符，但 Document-Conventions $2.1 定义 id 为 `YYMMDDHHMM`（无连字符）。按当前决策漂移结果，强制连字符为正确标准。validator 正则正确；Document-Conventions 需更新。

**修复**：更新 Document-Conventions $2.1 的 id 格式定义为 `YYMMDD-HHMM[-NNN]-语义短名`（含连字符）。与 D-01 全局修正同步执行。

---

### D-15 | upstream 数组形式与代码模型不一致

**文件**：`docs/proposals/260616-1200-engine-dev-governance-chain.sih.md`（行4-6）

**断言**：frontmatter 中 `upstream:` 后跟 YAML 数组：

```yaml
upstream:
  - 260611-0000-sihankor-engine-design-summary
  - 240610-1030-on-sihankor-canon
```

当前 parser 和 models.rs 中 upstream 为 `Option<String>`（单值），数组形式无法被正确解析。

**根源分析**：upstream 的语义是治理授权链——回答"谁授权了这份文档的存在"，而非"这份文档参考了什么"。依赖关系另有承载：`DEPS`（前置依赖，可多值）和 `SEE-ALSO`（同级关联）。此处将 Canon（治理授权源）和 Engine-Design-Summary（工程依赖）混放在 upstream 中，混淆了两种关系。

**修复**：

1. 保持 upstream 单值——代码模型 `Option<String>` 正确，无需改动
2. 修正 governance-chain 文档的 upstream 为单一授权源：`upstream: 240610-1030-on-sihankor-canon`
3. 将 Engine-Design-Summary 的依赖关系移入 `DEPS` 或 `SEE-ALSO`

---

### D-16 | `SiHankor-Dev-Governance.sih.md` id 无连字符，与同日文档不一致

**文件**：`docs/specs/engineering/SiHankor-Dev-Governance.sih.md`

**断言**：id 为 `260616-1200-sihankor-dev-governance`（无连字符），但同日的其他文档如 `260616-1200-engine-dev-governance-chain` 使用了含连字符格式。

**修复**：随 D-01 全局修正统一添加连字符：`260616-1200-sihankor-dev-governance`。

---

### D-17 | `src/core/database.rs` search_by_nature 未实现 nature 索引

**文件**：`src/core/database.rs`（行133-143）

**断言**：search_by_nature 方法当前通过全表扫描 + 路径推断实现。按决策 #1（目录即身份），nature 应从路径推断并作为索引列存储。

**修复**：在 database schema 中增加 nature 列，在索引时从路径推断并写入，search_by_nature 使用索引列查询。

---

## 五、S 级发现（建议）

### S-01 | `SiHankor-Engine-Design-Summary.sih.md` Rust 模型描述含 `decided_by`

**文件**：`docs/specs/engineering/SiHankor-Engine-Design-Summary.sih.md`

**建议**：$2.4.4 Frontmatter 结构体描述中包含 `decided_by` 字段。Spec 文档描述代码时引用已废除字段可能误导开发者。在 B-07 修正后同步更新此描述。

---

### S-02 | `SiHankor-Document-Conventions.sih.md` 对 `_concepts.yml` 的否定性提及

**文件**：`docs/specs/engineering/SiHankor-Document-Conventions.sih.md`（行340）

**建议**：文档写"不需要独立的 `_concepts.yml`"，虽语义正确（支持决策 #7），但提及已废除文件名可能造成读者困惑。建议改为直接说明 glossary 通过 derives-from 链 join reference/，不出现 `_concepts.yml`。

---

### S-03 | specs/ 文档 ADR 正文中的 `decided-by` 转换为 prose 表述

**文件**：`On-SiHankor.sih.md`、`On-SiHankor-Tao.sih.md`、`Arche-The-One-Above-Being.sih.md`、`On-SiHankor-Assay.sih.md`、`On-SiHankor-Canon.sih.md`、`SiHankor-Document-Conventions.sih.md`

**建议**：上述文件的附录 ADR 块（正文中，非 frontmatter）包含 `decided-by: ai-assist`。此处的语义是叙事性签认记录，不是引擎治理字段——但因使用了与 frontmatter 相同的 `key: value` 格式，造成人机歧义。

**处理方式**：保留记录价值，转换为 prose 格式。

- **当前**：ADR 末尾裸露 `decided-by: ai-assist`
- **改为**：`> 本附录的设计决策由 AI 辅助生成，人类审核确认。`（或等效的自然语言表述）

**理由**：

1. **记录有价值**：这些哲学文档的形成方式信息对读者理解可信度边界有意义，不应删除
2. **格式消歧**：prose 表述明确区分了叙事签名与 frontmatter 元数据，消除歧义
3. **与 B-01 一致**：B-01 移除 ADR 尾部的 `decided-by: ai-assist` 行；本建议为其提供替代方案——删后补 prose

---

### S-04 | `On-SiHankor.sih.md` $6.2 顺因之法中 `resolve_ref` 历史提及

**文件**：`docs/specs/philosophy/On-SiHankor.sih.md`

**建议**：$6.2 顺因之法的工程体现中列有 `resolve_ref 溯源`。此为讨论废除过程的历史性提及（属豁免范围），但建议添加注释说明"resolve_ref 已由 upstream 链溯源替代"，避免新读者误解。

---

### S-05 | `SiHankor-Philosophy-Compendium.sih.md` 交叉引用可增强

**文件**：`docs/reference/SiHankor-Philosophy-Compendium.sih.md`

**建议**：Compendium 作为权威定义源，在术语速查表中可增加"工程映射"列，指向 Engineering-Mapping 中对应条目。

---

### S-06 | `.sih/semantic.yml` 仅为占位文件

**文件**：`.sih/semantic.yml`

**建议**：当前仅含注释说明待 engine 实现后填充。建议在文件头部增加预期的 schema 示例（注释形式），使开发者能理解未来数据结构。

---

### S-07 | `Format-Violations.sih.md` 内容已合并，可考虑归档

**文件**：`docs/knowledge/notes/Format-Violations.sih.md`

**建议**：该 note 自身声明内容已合并入 Document-Conventions $八。已合并内容的 note 可考虑标记终止（stage: X）并归档。

---

### S-08 | `parser.rs` 未对非 decisions/ 含 decided-by 产出警告

**文件**：`src/core/parser.rs`

**建议**：parser 从 YAML 中提取 decided-by 但不检查其合法性。建议在 parser 层增加 warning：当非 decisions/ 路径的文档包含 decided-by 时产出警告。与 B-08（validator 反向校验）互补。

---

### S-09 | `Stage` 结构体缺少 note 生命周期支持

**文件**：`src/core/models.rs`

**建议**：Stage 结构体有 `propose()`, `resolve()`, `ratify()`, `deprecated()` 方法，但缺少 note 的 `confirm()`（人类确认 1/3→2/3）和 `decay()` 方法。建议在 Document 或 Stage 中增加 note 生命周期支持。

---

### S-10 | 确立文件名规范：PascalCase 驼峰、无连字符、无时间戳

**文件**：全部 .sih.md 文件（文件名层面）

**建议**：当前文件名存在三种模式混用——纯 PascalCase 含连字符（如 `On-SiHankor-Canon.sih.md`）、语义 kebab-case（如 `Format-Violations.sih.md`）、时间戳前缀（如 `260616-1600-docs-system-review.sih.md`）。建议确立统一的文件名规范：

- **PascalCase 驼峰命名**，不含连字符。如 `OnSiHankorCanon.sih.md`、`SiHankorDocumentConventions.sih.md`
- **不含时间戳**。时间戳归 id（引擎排序/唯一性），文件名归人类（浏览/定位）
- **`.sih.md` 后缀**

**设计理由**：

1. **道二（意图先于代码）**：文件名承载人类扫描意图——人在 `ls` 或目录树中应一眼认出文档主题。时间戳挡在语义名前面是引擎格式侵占人类浏览空间。
2. **文件名归人类，id 归引擎**：Document-Conventions 行 67 明确"id 的唯一职责是引擎内无歧义标识，不承担人类阅读功能。人类通过文件名和 title 定位文档"。行 461 明确"文件名与 id 的语义短名不强制一致"。两者各自服务不同受众，不应互相约束。
3. **顺因之法**：PascalCase 驼峰是人类扫描英文复合词的自然方式（大写字母充当词边界信号），不需要连字符辅助分割。时间戳只出现在 id 中，引擎用它排序；文件名保持纯语义。

**修复**：在 Document-Conventions 中纳入此规范（更新 $2.1 或新增文件名专节）。现有文件的文件名修正随 D-01 全局 id 格式统一时批量执行。

---

## 六、统计

### 按级别

| 级别      | 数量 |
| --------- | ---- |
| B（阻塞） | 8    |
| D（偏离） | 16   |
| S（建议） | 10   |

### 按决策维度

| 决策                        | B 级 | D 级 | 说明                                                                                                |
| --------------------------- | ---- | ---- | --------------------------------------------------------------------------------------------------- |
| #1 目录即身份/type 废除     | 0    | 1    | D-13 Mind-Design type 残留                                                                          |
| #3 note 有 stage            | 1    | 2    | B-04 AGENTS.md 矛盾；D-02 3 文档称"note 无 stage"；D-08 note 缺 stage                               |
| #4 frontmatter 精简         | 0    | 1    | D-12 AGENTS.md 缺 decided-by/successor 定义                                                         |
| #5 upstream 自指向          | 0    | 1    | D-11 validator 缺自指向检查                                                                         |
| #6 decided-by 限 decisions/ | 3    | 0    | B-01 specs/ 清理；B-02 proposals/ 清理；B-07/B-08 代码模型                                          |
| #8 glossary 项目根          | 1    | 0    | B-06 README 位置错误                                                                                |
| #10 po/ 废除                | 1    | 1    | B-05 Roadmap；D-05 Legacy-Migration 历史引用                                                        |
| #11 "单向不可逆"替换        | 0    | 1    | D-03 3 文件术语残留                                                                                 |
| #id 格式强制连字符          | 0    | 5    | B-03 upstream 断裂；D-01 全局漂移；D-09 note id；D-14 validator vs Conventions；D-16 Dev-Governance |
| 文件名规范（无时间戳/驼峰） | 0    | 0    | S-10 建议确立文件名规范                                                                             |
| — 其他                      | 1    | 2    | B-03（同 id 格式）；D-06 External-Validation 迁移 note；D-15 upstream 数组；D-17 search_by_nature   |

---

## 七、总体评价

### 7.1 核心矛盾

本次审阅识别出两个系统性根源问题：

**根源一：AGENTS.md 与决策链脱节**。AGENTS.md 中 note stage 定义（"none (verified only)"）已被 `260615-1500-docs-restructure-v2-decision` 覆盖，但 AGENTS.md 在历次审阅修复中未被更新。此问题导致 B-04、D-02、D-08 连锁偏差。

**根源二：id 格式决策漂移**。从 `YYMMDD-HHMM` → `YYMMDDHHMM` → `YYMMDD-HHMM` 的往返漂移未伴随 Document-Conventions 和 AGENTS.md 的同步更新，导致 validator.rs 的正则与文档规范矛盾，约 20 个文件存在新旧格式混合。

### 7.2 优先修正建议

**P0 — 立即修复（阻断级）**：

1. B-04：修正 AGENTS.md note stage 定义
2. B-01/B-02：批量清理 decided-by（8 个文件）
3. B-07/B-08：修正 Rust 代码模型 decided_by + validator 反向校验
4. B-06：修正 README glossary 位置
5. B-05：清理 Roadmap po/ 引用
6. B-03：修正 upstream 断裂项（随 D-01 全局 id 统一修正）

**P1 — 短期修复（偏离级）**：

1. D-01：启动全局 id 格式统一修正（~20 文件 + Conventions + AGENTS.md）
2. D-02：对 Canon 等 3 个文档启动 Reopen 修正"note 无 stage"表述
3. D-08：为 4 个 note 文件添加 stage 字段
4. D-11/D-17：补全代码模型自指向检查、nature 索引
5. D-03："单向不可逆"术语清理（3 文件）

**P2 — 中期改进**：

- D-04 至 D-06、D-12 至 D-16：编码格式、External-Validation 迁移、AGENTS.md 字段补充
- S-01 至 S-10：建议级改进项

### 7.3 无问题文件

以下文件审阅无发现问题：

- `docs/specs/engineering/SiHankor-Dev-Governance.sih.md`（除 id 格式外）
- `docs/reference/SiHankor-Onomastic-Philosophy.sih.md`
- `docs/proposals/260616-1210-post-restructure-doc-cleanup.sih.md`
- `docs/proposals/260616-1214-gap-entity-definition.sih.md`
- `docs/proposals/260615-1530-decided-by-placement.sih.md`
- `docs/decisions/260615-1500-docs-restructure-v2-decision.sih.md`
- `docs/decisions/260616-1210-post-restructure-doc-cleanup-decision.sih.md`
- `docs/decisions/260615-1530-decided-by-placement-decision.sih.md`
- `glossary/zh.yml`
- `glossary/en.yml`
- `src/core/indexer.rs`
- `src/core/orchestrator.rs`
