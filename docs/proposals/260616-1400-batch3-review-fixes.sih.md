---
id: 260616-1400-batch3-review-fixes
stage: 3/3
upstream: 240610-1030-on-sihankor-canon
---

# 第三批审阅修复：正文/代码层旧体系残余清理

> 第三批审阅（工程文档 + proposals/decisions/knowledge + 源代码）发现：重构决策（废除 type、目录即身份、_concepts.yml 废除、upstream 替代 resolve_ref、root docs 自指向）在 frontmatter 层已执行，但正文内容和代码模型仍大量残留旧体系。本提案逐项列明变更。

## 前置条件

B-02 id 连字符决策已漂移回有连字符格式 `YYMMDD-HHMM`。本提案中所有 id 格式引用以漂移后格式为准。B-02 回退执行（全项目 frontmatter id 加连字符）不在本提案范围内，需单独提案。本提案 upstream 引用 `240610-1030-on-sihankor-canon` 在当前仓库中尚不可解析（Canon 实际 id 为 `2406101030-on-sihankor-canon`，无连字符），待 B-02 执行后自动解除。

## 变更清单

### 文件 1：Document-Conventions（specs/engineering/，stage 2/3）

| 编号     | 变更                                                                                                                                    | 位置      |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| **B-14** | 删除 `_concepts.yml` 引用："拆分后 `_concepts.yml` 不变，引擎 扫描目录树按概念键匹配" -> "拆分后引擎扫描目录树按概念键匹配"              | 行384     |
| **B-15** | 引擎校验规则表中删除三项 `_concepts.yml` 依赖规则（unregistered/untranslated/dangling），保留 orphan/stale/duplicate 三项               | 行392-394 |
| **S-11** | 合并两个 $5.4：将 $5.4 拆分机制（行382-384）内容并入前一个 $5.4 治理模型（行364-380），删除重复的 $5.4 编号。后续 $5.5 引擎校验规则不变 | 行364-384 |

### 文件 2：Engine-Design-Summary（specs/engineering/，stage 2/3）

| 编号     | 变更                                                                                                              | 位置       |
| -------- | ----------------------------------------------------------------------------------------------------------------- | ---------- |
| **B-20** | 三层 glossary 表中删除 `_concepts.yml` 行，改为两层（zh.yml + en.yml）。因果方向描述保留，删除 `_concepts.yml` 行 | 行68-71    |
| **B-16** | `SihDatabase` trait：`search_by_type(&self, doc_type: &str)` -> `search_by_nature(&self, nature: &str)`            | 行90       |
| **B-17** | `Document` 结构体：删除 `pub r#type: DocType` 行（行107）                                                         | 行107      |
| **B-17** | `Frontmatter` 结构体：删除 `pub r#type: DocType` 行（行131）。删除 `pub decided_by: Option<String>` 行（行134）   | 行131, 134 |
| **B-18** | SQL schema：删除 `type TEXT NOT NULL` 列（行167）和 `CREATE INDEX idx_documents_type`（行177）                    | 行167, 177 |
| **B-19** | 存储设计要点："根级文档用全大写域标识（如 `PHILOSOPHY`），不存为 NULL" -> "根级文档自指向自身 id"                  | 行186      |

### 文件 3：Engineering-Mapping（specs/engineering/，stage 2/3）

| 编号     | 变更                                                                                                                                            | 位置      |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| **B-21** | 两处 `resolve_ref 溯源` -> `upstream 溯源`                                                                                                       | 行15, 249 |
| **B-22** | 删除整个 $5.2 `scope.yaml 的哲学含义`（行149-151）。scope.yaml 已废除                                                                           | 行149-151 |
| **B-23** | $6.3 标题 `"单向不可逆"与顺因之法` -> `propose->resolve->ratify 与顺因之法`。正文中修正模型：propose->resolve->ratify 单向流动，修正通过叠加而非覆盖 | 行169-171 |

### 文件 4：Mind-Design（specs/engineering/，stage 2/3）

| 编号     | 变更                                                                                      | 位置  |
| -------- | ----------------------------------------------------------------------------------------- | ----- |
| **B-24** | (1) 意图定位追问："type/stage/upstream" -> "nature/stage/upstream"                           | 行57  |
| **B-24** | (1) 输出描述："type x stage x upstream 链" -> "nature x stage x upstream 链"                 | 行58  |
| **B-25** | `governance_position` 注释："type x stage x upstream 链" -> "nature x stage x upstream 链" | 行127 |

### 文件 5：Engine-Roadmap（proposals/，stage 1/3）

| 编号     | 变更                                                                                                                                                                              | 位置      |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| **B-26** | reference 域验证："resolve_ref 指向有效文档" -> "upstream 指向有效文档"                                                                                                            | 行227     |
| **B-27** | `project_status` 输出："按 type/stage 的分布统计" -> "按 nature/stage 的分布统计"                                                                                                  | 行279     |
| **B-28** | 意图定位："读取 type/stage/upstream" -> "读取 nature/stage/upstream"                                                                                                               | 行307     |
| **D-19** | Mermaid 成熟度矩阵（行39-40）：Engineering-Mapping 和 Mind-Design 从 `1/3` 更新为 `2/3`                                                                                           | 行39-40   |
| **D-19** | 类别二推进表（行99-100）：Engineering-Mapping 和 Mind-Design 已从 1/3 推进到 2/3。当前行表示"当前1/3->目标2/3"，需更新为当前2/3->目标3/3；或删除此表（与 Mermaid 已更新的状态矛盾） | 行97-101  |
| **S-12** | 合并重复的第五条（行502-504）：行502-503 是有效状态记录"Mind-Design 1/3->2/3：已完成"，行504-505 是旧版任务描述。删除行504-505，保留行502-503                                      | 行502-505 |

### 文件 6：Type-Extension（decisions/，stage 2/3）

| 编号     | 变更                                                                                                                                                   | 位置        |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------- |
| **B-29** | 整个决策主题是"将 type 从 4 种扩展为 7 种"，type 已废除。不修改正文（历史记录），仅将 stage 从 `2/3` 改为 `0/260615-1500-docs-restructure-v2-decision` | frontmatter |

### 文件 7：External-Validation（decisions/，stage 3/3）

| 编号     | 变更                                                                                         | 位置        |
| -------- | -------------------------------------------------------------------------------------------- | ----------- |
| **B-30** | 删除非标准字段 `participants`（不在 id/stage/upstream 四字段设计内）。参与者信息移入正文首段 | frontmatter |
| **D-18** | "Note 类型" -> "Note"（"类型"是旧体系自然语言残留）                                           | 行85        |

### 文件 8：Legacy-Migration-Governance（decisions/，stage 3/3）

| 编号     | 变更                                 | 位置 |
| -------- | ------------------------------------ | ---- |
| **D-17** | `resolve_ref 路径` -> `upstream 路径` | 行73 |

### 文件 9：models.rs（src/core/）

| 编号     | 变更                                                                                         | 位置    |
| -------- | -------------------------------------------------------------------------------------------- | ------- |
| **B-32** | 删除 `DocType` 枚举定义（7 种变体：Treatise/Compendium/Mapping/Note/Plan/Decision/Proposal） | 行7-15  |
| **B-32** | 删除 `impl DocType` 块（`as_str()` + `from_str()` 方法）                                     | 行17-42 |
| **B-32** | `Frontmatter` 结构体：删除 `pub r#type: DocType` 字段                                        | 行112   |
| **B-32** | `Frontmatter` 结构体：删除 `pub decided_by: Option<String>` 字段                             | 行115   |
| **B-32** | `Document` 结构体：删除 `pub r#type: DocType` 字段                                           | 行124   |
| **B-32** | `SearchResult` 结构体：删除 `pub r#type: DocType` 字段                                       | 行138   |
| **B-32** | `ChainNode` 结构体：删除 `pub r#type: DocType` 字段                                          | 行149   |

### 文件 10：validator.rs（src/core/）

| 编号     | 变更                                                                                                                                                                                                                                  | 位置      |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| ：        | 删除 `use super::models::DocType` 导入                                                                                                                                                                                                | 行3       |
| **B-33** | 删除 F-02 规则（验证 type 值是否为合法 DocType 枚举变体）                                                                                                                                                                             | 行118-130 |
| **B-34** | F-04 规则：将 `!matches!(doc.r#type, DocType::Note \| DocType::Plan)` 改为基于路径（nature）判断 upstream 必填性：note 类型（knowledge/notes/）无 upstream，proposal 在 proposals/ 目录下有 upstream。需从 `doc.id` 或路径推断 nature | 行143     |
| **B-35** | G-02 规则：删除 `DocType->期望目录` 的匹配表。改为从文档所在路径推断 nature，反向校验目录正确性                                                                                                                                        | 行173-179 |
| **B-36** | G-01 规则：删除 `PHILOSOPHY` 等大写域标识作为合法根级 upstream 的许可（行152-157 区域）。root docs 应自指向自身 id                                                                                                                    | 行152-157 |
| **B-37** | G-09 规则：decided-by 校验从 `DocType::Treatise \| Compendium \| Mapping \| Decision` 改为仅对 `decisions/` 目录文档执行                                                                                                              | 行324-335 |
| **B-37** | 删除 `ai-auto` 禁止值检查（decided-by 领域不在 validator 通用规则中）                                                                                                                                                                 | 行342-348 |
| ：        | 测试辅助函数 `make_test_doc` 删除 `doc_type: DocType` 参数                                                                                                                                                                            | 行391     |

### 文件 11：database.rs（src/core/）

| 编号     | 变更                                                                                                       | 位置            |
| -------- | ---------------------------------------------------------------------------------------------------------- | --------------- |
| ：        | 删除 `use super::models::DocType` 导入                                                                     | 行6             |
| **B-33** | `SihDatabase` trait：`search_by_type(&self, doc_type: &DocType)` -> `search_by_nature(&self, nature: &str)` | 行13            |
| **B-27** | `SihDatabase` trait：`count_by_type` -> `count_by_nature`，返回按 nature 的分布统计                         | 行19            |
| **B-18** | SQL schema：删除 `type TEXT NOT NULL` 列和 `idx_documents_type` 索引                                       | 行70, 80        |
| ：        | `upsert_document`：删除 `doc.r#type.as_str()` 引用和 SQL INSERT 的 `type` 列                               | 行95, 101-105   |
| ：        | `get_document` SELECT 语句删除 `type` 列                                                                   | 行121           |
| ：        | `search_by_type` -> `search_by_nature`，按 nature（从路径推断）而非 DocType 过滤                            | 行137           |
| ：        | `count_by_type` -> `count_by_nature`                                                                        | 行258           |
| ：        | `row_to_document`：删除 `DocType::from_str` 调用和 `r#type` 赋值                                           | 行175, 216, 310 |
| ：        | `SearchResult` 构造：删除 `r#type: DocType::from_str(...)`                                                 | 行175           |
| ：        | `ChainNode` 构造：删除 `r#type: DocType::from_str(...)`                                                    | 行216           |

### 文件 12：parser.rs（src/core/）

| 编号 | 变更                                                                                                                                                                              | 位置       |
| ---- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- |
| ：    | 删除 `use super::models::DocType` 导入                                                                                                                                            | 行5        |
| ：    | 删除 `InvalidType` 错误变体                                                                                                                                                       | 行16-17    |
| ：    | `parse_content`：删除 `r#type: frontmatter.r#type.clone()` 赋值                                                                                                                   | 行42       |
| ：    | `parse_frontmatter`：删除 `type` 字段提取逻辑（行91-97）。type 字段在 YAML frontmatter 中已不存在，但 YAML 解析不应因缺失字段而报错：：`get("type")` 结果为空时跳过，不作为必填字段 | 行91-97    |
| ：    | `parse_frontmatter`：删除 `decided-by` 提取逻辑，decided-by 仅限 decisions/ 目录文档的 YAML frontmatter                                                                           | 行114-117  |
| ：    | `Frontmatter` 构造：删除 `r#type: doc_type` 和 `decided_by` 赋值                                                                                                                  | 行127, 130 |

### 文件 13：governance.rs（src/mcp_server/）

| 编号 | 变更                                                                                                                 | 位置                      |
| ---- | -------------------------------------------------------------------------------------------------------------------- | ------------------------- |
| ：    | `r.r#type.as_str()` 引用（4 处）：nature 从路径推断或显示空字符串。搜索结果显示 nature（从文档路径获取）而非 DocType | 行104, 131, 165, 208, 218 |

## 影响范围

- 工程文档 5 份
- 决策文档 3 份
- Rust 源代码 5 份（models.rs, validator.rs, database.rs, parser.rs, governance.rs）
- 变更总量约 50+ 处

## 非本提案范围

- B-02 id 格式回退（全项目 frontmatter + Conventions 正则 + zh.yml derives-from）
- D-14 `.sih/config.yml` 检查
- D-20 archive/Restructure-Plan（文件已不存在）
- S-15 `.sih/` 目录创建（连锁于 Engine-Roadmap）

## 已修复项（无需操作）

以下第三批审阅项在前两批修复中已解决，或审阅时基于已过时的假设：

| 编号 | 原因                                                                            |
| ---- | ------------------------------------------------------------------------------- |
| B-11 | `type` 已改为 `nature`                                                          |
| B-12 | stage 语义表列标题已改为 `nature`                                               |
| B-13 | glossary 路径已改为项目根级描述                                                 |
| B-31 | zh.yml derives-from 当前与实际文档 id 匹配（均为无连字符），B-02 回退时一并修正 |
| D-13 | id 格式正则一致性 -> B-02 回退时一并处理                                         |
| D-15 | note 状态机已写明"人类确认"                                                     |
| D-20 | archive/Restructure-Plan 文件不存在                                             |
| S-13 | glossary 已物理移至项目根级                                                     |
