---
id: 260615-1500-docs-restructure-v2-decision
stage: 3/3
upstream: 260615-1430-docs-restructure-v2
decided-by: ai-assist
---

# 司衡文档体系重构 v2 决策记录

> 采纳 260615-1430-docs-restructure-v2 提案。记录决策理由与后果。

## 背景

司衡文档体系存在四个结构性问题：(1) type 字段与目录构成双信号系统，违反道三；(2) stage 同时承载成熟度、流程位置、引用权限三层语义；(3) type 定义在 Canon 与 Document-Conventions 中不一致；(4) 治理元层与被治理内容混放在同一浏览流中。

提案从道层重新推导，提出"目录即身份"替代 type 字段、"stage 语义按 nature 分化"替代三层语义混装、引入 knowledge/ 和 archive/ 分离构思碎片与废弃文档、将 glossary 定位为纯跨人类语言翻译、新增 semantic.yml 作为意图↔代码语义映射。

## 决策

采纳提案全部内容，已执行：

1. 废除 `type` 字段 — 目录即身份，引擎从路径第一层推断 nature
2. `stage` 语义按 nature 分化 — spec/proposal/decision/reference 表达可信度，note 表达生命周期
3. 目录结构重整 — `specs/` `proposals/` `decisions/` `reference/` `knowledge/`（含 drafts/ notes/）`archive/`
4. frontmatter 字段精简 — id, stage, upstream, successor（删除 type, domain）
5. glossary 重定位 — zh.yml 充实 derives-from，删除 _concepts.yml 和 po/
6. semantic.yml 新建 — 排放于 .sih/，引擎意图↔代码映射
7. 17 份文档 frontmatter 批量更新，5 份文件物理移动
8. Canon Reopen → 修改 $3.2 $6 → 定稿 3/3
9. Document-Conventions 同步更新

## 后果

- 正向：消除双信号系统，一个目录一个 nature，人类浏览路径单一无歧义
- 正向：stage 语义精确化，可信度与生命周期不再混用同一编码
- 正向：knowledge/ 和 archive/ 分离了构思、洞察、废弃三种不同的知识形态
- 正向：glossary 与 semantic 的问题域分离，维护节奏分离，消费者分离
- 风险：交叉引用更新可能存在遗漏，后续使用中发现断裂需补充修正
- 风险：semantic.yml 的完整 schema 和 engine 索引实现尚未定义，此为道四声明的已知间隙

