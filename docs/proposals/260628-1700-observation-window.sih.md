---
id: 260628-1700-observation-window
stage: 3/3
upstream: 260628-2300-sih-entry-model
---

# 观测窗：让司衡看见陌生项目

> DSR-3 暴露的核心盲区是司衡只被验证于血缘近似项目（SiHankor-A）。本提案引入"观测窗"作为治理门的前置工具：让司衡能分析任何 markdown 项目，不预设对方有 .sih.md / frontmatter / stage。

## 背景

DSR-1（commit `01dc922`）和 DSR-2（commit `8f84730`）都在 SiHankor-A（Python 版司衡近亲）上跑通治理迭代，DSR-3（commit `11d5c1b`）扩展到 bat（2 个 .sih.md）和 fd（0 个 .sih.md）两个真实开源项目。三轮 DSR 都假设目标项目能被司衡 runtime 索引、解析、治理。

盲区：如果目标项目**完全没有 .sih.md**，司衡当前**什么都看不见**——index_rebuild 返回 0/0、ProjectSnapshot 归零、所有度量退化为退化值。这是 DSR-3 在 fd 项目上发现的事实。

观测窗要解的不是"治理陌生项目"，而是"在治理前，先看见陌生项目长什么样"。

## 设计

### 入口

`observe` MCP group 第一个工具：`observe_project`

CLI 等价命令：`sihankor observe <path>`（path 可以是任意目录）

### 输出结构

观测窗输出两个层：描述层（结构特征）+ 预测层（规则触达预测）。

#### 描述层（5 维结构特征，纯 Markdown 分析，不消费司衡概念）

| 维度 | 测量方法 |
| --- | --- |
| 文件统计 | 扫描 `**/*.md`，统计总数 / 总字节 / 平均行数 |
| 目录结构 | 子目录深度分布（`docs/a/` vs `docs/a/b/c/`） |
| 表格列数 | 统计每个 markdown 表格的列数，分布直方图 |
| 代码块语言标签 | 统计 ```` ``` ```` 围栏代码块，标签覆盖率 |
| frontmatter 现状 | 统计含 YAML 围栏的文件，**只统计**不解析（不消费司衡 schema） |

这五个维度对应**纯 Markdown 结构特征**——不假设文件想成为 .sih.md，不假设 frontmatter 字段名是 `id`/`stage`/`upstream`。哪怕 frontmatter 是 Jekyll/Hugo/Pandoc 风格，也只统计"有 frontmatter"和"有 X 个字段"。

#### 预测层（基于描述层 → 规则触达预测）

| 司衡规则 | 预测触发条件 | 输出 |
| --- | --- | --- |
| V-F-01（id 必填且格式合法）| 含 frontmatter 但缺 id 字段的文件数 | N_f01_predicted |
| V-G-04（表格列数 ≤ 3）| 含 ≥ 4 列表格的文件数 × 表格数 | N_g04_predicted |
| V-G-05（代码块必须声明语言）| 缺 lang 的代码块数 | N_g05_predicted |
| V-G-06（禁止 emoji）| 含 emoji 的行数 | N_g06_predicted |
| V-F-05（禁止 --- 水平线）| 包含 `---` 围栏外的纯水平线 | N_f05_predicted |

**核心交付**：`{ "if_introduce_sihankor": { "F_predicted": N, "G_predicted": N, "J_predicted": N, "rule_breakdown": {...} } }`

这回答用户的真问题："**引入司衡治理这里，会触发多少规则？**"——是治理前的尽职调查。

### 知止边界

观测窗**不**做以下事（这些是治理门的工作，不是观测门）：

- 不消费司衡的 `id` / `stage` / `upstream` / `decided-by` schema
- 不做合规判断（"这文档是否合法"）
- 不修改任何文件
- 不依赖司衡的 metrics / violations 表
- 不假设项目已经被索引

观测窗**只**做"看见"。任何"治理"动作属于 `govern` group。

### 校准数据

DSR-3 已经提供了两个校准样本：

| 项目 | .sih.md 数 | 观测窗预测 F | 实际治理 F |
| --- | --- | --- | --- |
| SiHankor 自身 | 65+ | 53（v1 实测）| 53（修复前）|
| bat | 2 | 待观测 | 待观测 |
| fd | 0 | 待观测 | N/A（无文档可治理）|

观测窗 MVP 完成后，跑这三个项目，**对比预测 vs 实际**，建立校准曲线。校准数据让预测从"启发式"变成"基于证据的统计"。

## 与 violation-distribution 的关系

| 维度 | 观测窗（本提案）| violation-distribution（后续提案）|
| --- | --- | --- |
| MCP group | observe 入口 1 | observe 入口 2 |
| 输入 | 任意 markdown 项目 | 已治理项目（消费 metrics）|
| 假设 | 项目**不**需要 .sih.md | 项目**已经**有 .sih.md |
| 回答 | "如果治理会触达什么" | "治理后违规怎么分布" |
| 时序 | 治理前 | 治理中 / 治理后 |

两者互补：观测窗预测 → 用户决定是否引入司衡 → 引入后用 violation-distribution 监测治理效果。这是**先看 → 再治 → 后测**的完整生命周期。

## 交付物（MVP）

| 任务 | 内容 |
| --- | --- |
| 1 | 新建 `src/observe/` 模块：项目扫描器（纯 markdown 解析，无司衡概念）|
| 2 | 新建 MCP 工具 `observe_project`：扫描路径 + 输出描述层 + 预测层 |
| 3 | 新建 CLI 命令 `sihankor observe <path>`：MCP 工具的 CLI 包装 |
| 4 | 校准：跑 bat / fd / SiHankor 自身，对比预测 vs 实际 |
| 5 | 测试：纯函数（给定 mock markdown 目录，输出预期报告）|
| 6 | 文档：观察窗 + violation-distribution 的 MCP group 架构说明 |

预估工作量：1 个 dev branch + 1 个 verifier，约 1-2 天有效编码。

## 风险与缓解

| 风险 | 缓解 |
| --- | --- |
| 误把观察窗当治理工具 | 知止边界写在模块顶部 + 严格的命名（`observe_*` 而非 `validate_*`）|
| 预测启发式不准 | 校准数据（bat / SiHankor）建立 baseline，不准就 warn 用户 |
| Markdown 解析器有边角情况 | pulldown-cmark 已是项目依赖，参照现有 parser.rs 风格复用 |
| 与现有司衡体系耦合 | 物理隔离：`src/observe/` 不引用 `src/core/` 的任何 types |

## DEPS

- 260628-2300-sih-entry-model
  - 司衡双入口模型提案：govern + observe 双 MCP group 架构，本观测窗是 observe group 第 1 入口
- 260628-1100-engineering-mapping
  - 司衡工程映射，本文参考其 MCP 工具架构
- 260628-1700-dsr-cycle
  - DSR 周期提案，本提案源于 DSR-3 的 fd 项目盲区发现
- 260628-1500-trail-mechanism
  - 行迹机制，本提案的"先看后治"哲学与行迹的"先观测后决策"一致
