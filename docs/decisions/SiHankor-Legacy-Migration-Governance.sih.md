---
id: 260613-2000-sihankor-legacy-migration-governance
stage: 3/3
upstream: 260613-1728-sihankor-philosophy-compendium
decided-by: ai-assist
---

# 司衡 Legacy 迁移治理记录

> 将 `legacy/` 中 21 个文件的有效信息对齐到 `docs/` 的完整治理决策与执行记录。
> 执行窗口：2026-06-12 ~ 2026-06-13

## 一、逐文件对齐决策

### A 类：已覆盖，可废弃

这些 legacy 文件的内容已被 docs 中的对应文件完全吸收并深化。

**L2** `siheng-meta-20260601-001.sih.md`：废弃。核心内容已拆分吸收进 docs 五论：`On-SiHankor.sih.md`、`On-SiHankor-Tao.sih.md`、`On-SiHankor-Canon.sih.md`、`Arche-The-One-Above-Being.sih.md`。AI 定位段落交叉检查后补入 `Engine-Design-Summary.sih.md` $一和 `Arguments.sih.md` $4.8。

**L21** `siheng-mind-20260601-001.md`：废弃。旧版"司衡心法"总览。内容与 L2 大量重叠，增量信息已降级为"spec-coding 术内部"概念。

**L4** `conceptual-framework-20260602-001.sih.md`：废弃。已被 `SiHankor-Philosophy-Compendium.sih.md` 和 `On-SiHankor.sih.md` 的 Mermaid 图取代。

**L3** `decisions-index-20260601-001.sih.md`：废弃。决策是道的发现轨迹，不是道本身。核心结论已吸收进五论。#41/#42 的"道的发现时刻"作为方法论注记补入 `Arguments.sih.md`。前 37 条决策原始时序已不可精确恢复，迁移本身就是有损编码（道四递归适用）。

**L11** `siheng-six-dimensions-philosophy.md`：废弃。已被鉴论反推九段式系统性证伪（21 条子主张，0 条幸存）。

**L12** `propose-six-dimensions-consistency-check-20260602-001.sih.md`：废弃。核心发现（五维不是道、冲突识别）已被鉴论吸收。

**L1** `sih-doc-readme.md`：废弃。新结构有自明的目录导航。

### B 类：核心论证已吸收，作为论证集案例补充

**L7** `propose-falsify-dao-diverge-converge-20260602-001.sih.md`：合并到 `Arguments`。[已完成]

道一校准论证。补入 $一.2（命题结构精确拆解）：D1-D3 子命题拆解、"天然"三种强度分析、因果必然性 vs 描述性命题区分。

**L8** `propose-falsify-dao-understand-20260602-001.sih.md`：合并到 `Arguments`。[已完成]

道三校准论证。九段式反推完整过程已记录于 `Arguments` $二。补入道层双轴框架（核心矛盾轴 + 因果方向轴）。

**L10** `propose-falsify-daowuwei-20260601-001.sih.md`：合并到 `Arguments`。

五维天道反推验证（FAL）。21 条子主张逐一检验记录，四种失效模式归纳。择要补入 `Arguments` $四。

### C 类：已吸收但可补充细节到现有 docs

**L5** `elaborate-meta-above-dao-20260602-001.sih.md`：已覆盖，无需迁移。核心内容已被 `Arche-The-One-Above-Being.sih.md` 全面吸收并深化。

**L6** `propose-formalize-meta-concept-20260602-001.sih.md`：已覆盖，无需迁移。五条件框架、统一定义、命名原则均已被 `Arche` 吸收。

**L13** `siheng-architecture-philosophy-20260602-001.sih.md`：部分补入 `Tao`。[已完成]

- 发散的四种形态：补入 `On-SiHankor-Tao.sih.md` $2.1
- 发散的条件调节表：补入 `On-SiHankor-Tao.sih.md` $2.5

**L9** `evaluate-dao-completeness-20260602-001.sih.md`：已覆盖，无需迁移。D1-D6 六维评估框架的结果已体现在五论中。

**L14** `propose-philosophy-foundation-20260601-001.sih.md`：已覆盖，少量补入 `Arguments`。跨智库贡献注记写入 `Arguments`。[D10]

**L15** `propose-daoharmony-20260601-001.sih.md`：部分补入 `Engineering-Mapping`。[D11]

- 不确定性元数据：补入 $7.5（什么是 confidence/impact、对应哪条道/法、工程体现为什么）
- 软阈值分级：补入 $7.6（pass/soft-pass/soft-fail/fail 的定义和触发条件）

### D 类：已迁移到 reference 或 engineering

**L16** `naming-philosophy-20260531-001.sih.md`：已迁移到 `reference/SiHankor-Onomastic-Philosophy.sih.md`。Onomastic-Philosophy 已完全吸收并超越 L16。

**L17** `governance-plan-20260602-001.sih.md`：已迁移到 `engineering/SiHankor-Document-Conventions.sih.md`。Document-Conventions 已系统化吸收 L17 核心内容。

**L18** `review-mechanism-20260602-001.sih.md`：废弃。[已完成]提取一条原则补入 `On-SiHankor-Canon.sih.md` $2.1：归纳合成必须保持可追溯的 upstream 路径。

### E 类：保留待工程化

**L19** `propose-sihankor-mind-20260604-001.sih.md` + **L20** `sihankor-mind-meta-20260604-001.sih.md`：保留在 legacy，待 Rust 引擎实现 Mind MCP server 时将其设计决策提炼到 `Engineering-Mapping` 或 `specs/engineering/SiHankor-Mind-Design.sih.md`。

## 二、已决议事项

### 基础 12 项决策（2026-06-13 批量决议）

| #   | 决策项                  | 决议                                                           | 理由                         |
| --- | ----------------------- | -------------------------------------------------------------- | ---------------------------- |
| D1  | Legacy 删除方式         | 直接 rm（Git 可恢复）；删前运行遗漏检测                        | 心理阻力非实际               |
| D2  | FAL 逐维检验粒度        | 摘要+代表性案例，约 6 行                                       | 已决 7，无需再议             |
| D3  | Compendium 历史注记时机 | 等 Compendium 对齐完成后处理                                   | 1/3 文档上不做增量写入       |
| D4  | #41/#42 发现时刻格式    | 嵌入 Arguments $四 末尾，分隔行引导                            | 元叙事不独立成节             |
| D5  | Assay 三个方法论偏见    | 补入 Assay $五 末尾，6-8 行                                    | 已决 3 维持                  |
| D6  | L19/L20 当前动作        | 保留在 legacy，待工程实现后迁移                                | 空白占位违反知止             |
| D7  | Mind 远期归档位置       | 哲学约束：Engineering-Mapping；运作细节：Engine-Design-Summary | 边界清晰，不混类             |
| D8  | 人类可读性规范去向      | Document-Conventions $八 新增子节（3-5 行）                    | 不独立成文                   |
| D9  | L5/L6/L9 确认删除       | 直接删除，无需再扫                                             | 草稿纸无独立引用价值         |
| D10 | L14 补入范围            | 跨智库贡献注记写入 Arguments，等 Compendium 对齐后交叉引用     | 解除"等 Compendium"阻塞      |
| D11 | L15 补入范围            | 两个概念条目各 3-4 行；不展开配置值                            | 映射 vs 设计规范边界不可跨越 |
| D12 | 翻译流水线空文件        | po/ 下新增 README（5 行）                                      | .gitkeep 有可见性无含义      |

### cc-v2 与 glm-v2 增量决策（2026-06-13 批量决议）

| #   | 决策项                   | 来源    | 决议            | 理由                             |
| --- | ------------------------ | ------- | --------------- | -------------------------------- |
| D13 | 1/3 文档写入             | cc-Q1   | 接受写入        | Arguments 是叙事型文档           |
| D14 | L3 决策索引筛查          | cc-Q2   | 不筛查          | 决策!=道；结论已在五论中         |
| D15 | Tao $2.1 发散四形态覆盖  | cc-Q3   | 已确认          | 子节完整存在                     |
| D16 | FAL 粒度                 | cc-Q4   | 10 行摘要       | 已执行并验证                     |
| D17 | "道法自然，人为成事"口号 | cc-Q5   | 保持现状        | 不让标语喧宾夺主                 |
| D18 | F/G/J 35 条法则归属      | glm-D1  | 暂不处理        | 三道到四道兼容性审查前置         |
| D19 | 三机工程规范             | glm-D2  | 暂不处理        | 引擎骨架阶段建规范是空转         |
| D20 | Mind 定位补充确认        | glm-D3  | 保留 D6/D7 立场 | 不在当前阶段决策                 |
| D21 | 术语分级与引用标签       | glm-D4  | 暂不处理        | Conventions 已功能完整           |
| D22 | 三域边界模型             | glm-D5  | 暂不处理        | Engineering-Mapping 已有概念定义 |
| D23 | 约系                     | glm-D6  | 暂不处理        | iCL 引擎实现细节                 |
| D24 | 概念关系可视化           | glm-D7  | 暂不处理        | 纯可读性，不影响完整性           |
| D25 | 治理构成性条件论证       | glm-D8  | 不补入          | 反驳回应非正面定义               |
| D26 | 速度参考卡格式           | glm-D9  | 不引入          | AGENTS.md 风格约束               |
| D27 | legacy stage 标记更新    | glm-D10 | 不更新          | 历史快照改标记是形式主义         |

## 三、执行记录

| 日期       | 项目                          | 操作                                              | 涉及文件                                        |
| ---------- | ----------------------------- | ------------------------------------------------- | ----------------------------------------------- |
| 2026-06-12 | L2 AI 定位交叉检查            | 补入哲学溯源 + 历史注记                           | `Engine-Design-Summary.sih.md` $一              |
| 2026-06-12 | L13 发散形态+条件调节         | 补入四形态子节 + 条件依赖段                       | `On-SiHankor-Tao.sih.md` $2.1, $2.5             |
| 2026-06-12 | L18 内容审核机制              | 废弃；提取原则补入 Canon 顺因段                   | `On-SiHankor-Canon.sih.md` $2.1                 |
| 2026-06-12 | L7/C1 道一检验                | 补入命题拆解+天然三强度+因果区分                  | `Arguments.sih.md` $一                          |
| 2026-06-12 | L8/C2 道三检验                | 补入道层双轴框架论述                              | `Arguments.sih.md` $二                          |
| 2026-06-13 | L3 决策索引                   | 废弃；提取#41/#42 发现时刻为 Arguments 方法论注记 | `Arguments.sih.md` $四末尾                      |
| 2026-06-13 | P1-A: FAL 代表性案例          | 五维各一例，5 行分号格式                          | `Arguments.sih.md` $四.2                        |
| 2026-06-13 | P1-B: 跨智库注记+#41/#42 注记 | D10 注记 + 方法论注记                             | `Arguments.sih.md` $四.8 末尾                   |
| 2026-06-13 | P3: Engineering-Mapping       | 补充不确定性元数据 + 软阈值分级                   | `Engineering-Mapping.sih.md` $七.5, $七.6       |
| 2026-06-13 | P4: Assay 三种偏见            | 补入反证偏见/精确性偏见/片断化偏见                | `On-SiHankor-Assay.sih.md` $五末尾              |
| 2026-06-13 | P5: Document-Conventions      | 补入可读性约定子节                                | `Document-Conventions.sih.md` $八.10            |
| 2026-06-13 | P6: po/README                 | 创建翻译流水线 README（注：po/ 目录已在后续重构中废除） | `docs/glossary/po/README.md`                    |
| 2026-06-13 | 12 项决策批量决议             | 全部确认                                          | 见 $二 已决议事项表                             |
| 2026-06-13 | cc-v2 + glm-v2 决策决议       | 15 项全部决议                                     | 见 $二 增量决策表                               |
| 2026-06-13 | P7: 迁移完成审计              | 19 个待删文件全部通过                             | legacy/ 目录                                    |
| 2026-06-13 | P8: 删除 legacy 文件          | rm 19 个文件（A/B/C/D 类）                        | legacy/ 目录                                    |
| 2026-06-13 | P9: Mind 设计规范             | 创建 SiHankor-Mind-Design.sih.md                  | `specs/engineering/SiHankor-Mind-Design.sih.md` |

## 附录：独立审阅记录

本方案的决策过程受益于两份独立的 legacy 到 docs 对齐审阅：

- **cc-plan.md**（2026-06-12，DeepSeek）：对 21 个 legacy 文件的逐文件处置建议。增量决策点（Q1-Q5）已决议后纳入 $二。
- **glm-plan.md**（2026-06-12，GLM）：对 docs 工程层缺口的系统性识别。增量决策点（D1-D10）已决议后纳入 $二。

两份审阅均以 docs/ 为单一权威源独立产出。原始版本保留在 `legacy/plan/` 中作为历史记录。[^import-message]

[^import-message]: 作为历史遗存，在司衡引擎文档重构阶段，`legacy`目录下的所有文件已完成其历史使命，未纳入本项目的`git logs`，并在本文档中永久移除。保留本文并非为了考古研究，旨在阐明司衡引擎的起源历经多轮推导与决策。
