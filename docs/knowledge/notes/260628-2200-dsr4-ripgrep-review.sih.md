---
id: 260628-2200-dsr4-ripgrep-review
stage: 3/3
upstream: 260628-1700-dsr-cycle
verified: 260628
---

# DSR-4 回顾：开源项目 ripgrep 外部治理基线采集

> DSR-3 的两份外部基线（bat / fd）都是 sharkdp 旗下 Rust CLI 工具。DSR-4 选取第三方作者 BurntSushi 的 ripgrep 跨血缘——**作者、语言、CLI 类别都不同**——以验证司衡治理运行时在"无血缘近似"项目上的零覆盖真相与预测触达规则数。

## 一、执行摘要

| 项目 | 内容 |
| ---- | ---- |
| 目标项目 | ripgrep (`https://github.com/BurntSushi/ripgrep`) |
| 执行时间 | 2026-06-28 |
| 工具 | 自写 `observe_project.py`（与 `src/observe/scanner.rs` 同构）+ MCP server + mcp_client.py |
| 基线 | 0 个 .sih.md 文档（21 个 .md 全部被跳过） |
| 治理迭代 | 不修复——保持零覆盖真相，验证"陌生项目"信号链 |
| 行迹 | 0 条（沿用 DSR-3 教训：不强行记录门内项目状态） |
| 度量管道 | VarianceMetric / SnapshotDiff / RuleDensity / RuleAudit / TradeoffCoverage / TrendAlignment |

## 二、与 DSR-3 bat / fd 的策略对比

### 2.1 项目元数据对比

| 项目 | 作者 | 仓库大小 |
| ---- | ---- | -------- |
| DSR-3 bat | sharkdp | ~90MB |
| DSR-3 fd | sharkdp | ~15MB（shallow） |
| DSR-4 ripgrep | BurntSushi | ~9.4MB（shallow） |

三项目都是 Rust CLI 工具，但 bat（语法高亮 / cat 替代）、fd（文件查找 / find 替代）、ripgrep（文本搜索 / grep 替代）三类不同。ripgrep 跨血缘（不同作者）、跨类别（搜索 vs 高亮/查找）。

### 2.2 结构差异对比

| 项目 | .sih.md 数 | doc/ 子目录 |
| ---- | ---------- | ----------- |
| DSR-3 bat | 2（DSR-3 创建） | 有（4 md + 2 .sih.md） |
| DSR-3 fd | 0 | 有（3 md + 1 manpage） |
| DSR-4 ripgrep | **0** | 无（README/GUIDE/FAQ 平铺在根） |

### 2.3 修复策略对比

| 项目 | 修复模式 | 触及文件 |
| ---- | -------- | -------- |
| DSR-3 bat | 主动升级（创建 .sih.md） | 2 |
| DSR-3 fd | 被动修复（仅 1 处） | 1 |
| DSR-4 ripgrep | **不修复**（零覆盖真相） | 0 |

| 项目 | snapshot_diff 增量 | 行迹数 |
| ---- | ------------------ | ------ |
| DSR-3 bat | Documents +2 | 2 |
| DSR-3 fd | Documents +0 | 2 |
| DSR-4 ripgrep | 预期 +0 | **0** |

DSR-4 的"不修复"是 DSR 策略空间的新象限——既不同于 bat 的"升级跨进门"，也不同于 fd 的"修复但留在门外"。

## 三、ripgrep 结构特征采集

### 3.1 文件规模

| 维度 | 数值 |
| ---- | ---- |
| Markdown 文件数 | 21 |
| 其中 .sih.md | **0** |
| 总字节数 | 214,613 |
| 总行数 | **5,113** |
| 平均行数 / 文件 | 243.5 |

### 3.2 目录深度分布

| 深度 | 文件数 |
| ---- | ------ |
| 0（根目录） | 10 |
| 1（crates/） | 9 |
| 2（benchsuite/runs/） | 2 |

ripgrep 的 Markdown 集中分布于根目录（README/GUIDE/FAQ/CHANGELOG 等 10 个）+ `crates/*/README.md`（9 个）+ `benchsuite/runs/*/README.md`（2 个），**没有专门的 doc/ 目录**。这是与 bat / fd 的关键结构差异——后两者都有 `doc/` 子目录承载详细文档。

### 3.3 表格列数分布

| 列数 | 表数 |
| ---- | ---- |
| 4 | **6**（全部在 README.md） |

含 ≥ 4 列表格的文件数 = **1**（README.md）。6 张 4 列表都是 benchmark 对比——工具名、命令行、行数、时间。这是 V-G-04 的"经典触发场景"——性能对比表天然需要 4 列。

### 3.4 README.md 4 列表位置

| 行号 | 内容摘要 |
| ---- | -------- |
| 51-60 | Tool / Command / Line count / Time |
| 67-71 | 同样 4 列（基准对比变体） |
| 77-83 | 同样 4 列 |
| 89-94 | 同样 4 列 |
| 98-105 | 同样 4 列 |
| 110-115 | 同样 4 列 |

### 3.5 代码块语言标签覆盖率

| 维度 | 数值 |
| ---- | ---- |
| 代码块总数 | 129 |
| 有 lang | 20 |
| 无 lang | **109** |
| 覆盖率 | **15.5%** |

GUIDE.md 是无 lang 代码块重灾区（45/109 = 41.3%）。该文档是 ripgrep 用户指南，大量命令行示例未标注语言标签。

### 3.6 无 lang 块最严重的文件

| 文件 | 无 lang 块数 |
| ---- | ------------ |
| GUIDE.md | **45** |
| README.md | 31 |
| FAQ.md | 28 |
| CHANGELOG.md | 2 |
| crates/globset/README.md | 2 |

### 3.7 实际使用的代码块语言

| 语言 | 块数 |
| ---- | ---- |
| toml | 9 |
| zsh | 3 |
| rust | 3 |
| powershell | 2 |
| rust,no_run | 2 |
| bash | 1 |

只有 6 种不同语言被实际标注，**bash 用得极少**——这与 bat（bash 占 262/290 = 90%）和 fd（bash 占 44/45 = 98%）形成鲜明对比。ripgrep 把绝大多数 shell 示例当作 plain text 写出。

### 3.8 Frontmatter 现状

| 维度 | 数值 |
| ---- | ---- |
| 含 frontmatter | 1 |
| frontmatter 覆盖率 | **4.8%** |
| 含 id 字段 | **0** |
| 含 stage 字段 | **0** |
| 含司衡合法 stage | **0** |

唯一的 frontmatter 文件是 `.github/ISSUE_TEMPLATE/feature_request.md`（GitHub issue 模板格式：`name` / `about` / `title` / `labels` / `assignees`），不是 SiHankor 风格 frontmatter。

### 3.9 水平线与 emoji

| 维度 | 数值 |
| ---- | ---- |
| body 中 --- 水平线 | 9 条（README.md 1 + crates/*/README.md 各 1） |
| 含 emoji 行 | **0** |

## 四、预测触达规则数（DSR-4 核心输出）

| 规则 | ripgrep 触达 |
| ---- | ------------ |
| V-F-01 | **0** 文件 |
| V-F-05 | **9** 条 / 9 文件 |
| V-G-04 | **1** 文件 / 6 张表 |
| V-G-05 | **109** 块 |
| V-G-06 | **0** 行 |

V-F-01 = 0 因为前置：先升级为 .sih.md 才会触发。

V-F-05 = 9 条 body 含 `---` 水平线（cratel 子目录 README.md 普遍有）。

V-G-04 = 1 文件（README.md）含 6 张 4 列 benchmark 对比表。

V-G-05 = 109 块缺 lang 的代码块（GUIDE.md 占 45）。

V-G-06 = 0 行无 emoji（ripgrep 文档遵守 ascii-only 约定）。

**DSR-4 的核心信号**：ripgrep 若被强制引入司衡治理，会触发 9 条 V-F-05 + 1 文件 V-G-04 + 109 块 V-G-05 = **共 119 条违规**。其中 V-G-05 是最大风险（109 块）——这是 ripgrep 用户向文档风格迁移的主要摩擦点。

## 五、跨项目结构特征对比

### 5.1 规模与代码块覆盖

| 项目 | .md 数 | 代码块数 |
| ---- | ------ | -------- |
| bat | 69 | 299 |
| fd | 8 | 71 |
| ripgrep | 21 | 129 |

| 项目 | lang 覆盖率 |
| ---- | ----------- |
| bat | **97.0%** |
| fd | 63.4% |
| ripgrep | **15.5%** |

### 5.2 表格与 frontmatter

| 项目 | ≥ 4 列表文件 |
| ---- | ------------ |
| bat | 1（10 列） |
| fd | 0 |
| ripgrep | 1（4 列） |

| 项目 | frontmatter 文件 |
| ---- | ---------------- |
| bat | 6 |
| fd | 2 |
| ripgrep | 1 |

| 项目 | id 字段 | stage 字段 |
| ---- | ------- | ---------- |
| bat | 2 | 2 |
| fd | 0 | 0 |
| ripgrep | 0 | 0 |

### 5.3 预测触达规则数对比

| 项目 | V-F-05 / V-G-04 / V-G-05 |
| ---- | ------------------------ |
| bat | 1 / 1 / 9 |
| fd | 0 / 0 / 26 |
| ripgrep | **9** / 1 / **109** |

V-F-01 = 0（三项目都没用 .sih.md stage 字段），V-G-06 仅 bat 触发（8 行 emoji）。

**关键观察**：ripgrep 的 V-G-05 触达数（109）是 bat（9）的 12 倍、fd（26）的 4 倍。这与 ripgrep 的项目定位（CLI 工具用户向文档）相符——CLI 用户文档天然需要大量未标注的 shell 命令示例。

### 5.4 度量管道隔离 DB 视图

| 度量 | bat (隔离 DB) | fd (隔离 DB) |
| ---- | ------------- | ------------ |
| Discovered | 2 | 0 |
| Indexed | 2 | 0 |
| Rule Density | 7.00（14/2） | 0.00 |
| ADR 覆盖率 | 0.0% | 0.0% |
| Trend Alignment | 1.00（1/1） | 0.00 |

| 度量 | ripgrep (隔离 DB) |
| ---- | ----------------- |
| Discovered | **0** |
| Indexed | **0** |
| Rule Density | **0.00** |
| ADR 覆盖率 | **0.0%** |
| Trend Alignment | **0.00** |

DSR-4 沿用了 DSR-3 bat 修复后的 `SIHANKOR_DB_PATH` 环境变量，三份数据来自隔离 DB（`/tmp/<project>/.sih/index.db`），**不污染司衡自身 DB**。所有数字真实反映各项目状态。

## 六、DSR-4 关键发现

### 6.1 跨血缘近似扩展验证

| 信号 | bat / fd / ripgrep |
| ---- | ------------------ |
| 零 .sih.md | 是 / 是 / **是** |
| `discover.rs:40` 硬筛选触发 | 是 / 是 / **是** |
| `infer_nature` 对非标准目录 | 触发（doc/）/ 未触发 / **未触发**（根平铺结构） |
| 普通 Markdown 修复对运行时影响 | 高（+2 indexed）/ 零（+0 indexed）/ **预期零（不修复）** |

DSR-3 的两份基线（bat / fd）都来自同一作者 sharkdp。DSR-4 跨到 BurntSushi 的 ripgrep——作者不同、项目维护规模更大（21 .md vs fd=8 / bat=69）、CLI 类别不同（搜索 vs 高亮/查找）。**零覆盖信号不依赖血缘相似**。

### 6.2 lang 覆盖率的项目特征相关性

| 项目 | lang 覆盖率 | CLI 类别 |
| ---- | ----------- | -------- |
| bat | 97.0% | 语法高亮（cat 替代） |
| fd | 63.4% | 文件查找（find 替代） |
| ripgrep | **15.5%** | 文本搜索（grep 替代） |

| 项目 | 文档风格 |
| ---- | -------- |
| bat | bash-heavy 示例，标注充分 |
| fd | 中等混合 |
| ripgrep | 未标注的 shell 示例密集 |

**假说**：CLI 工具的文档语言标注覆盖率与"命令数量 / 文档总量"成反比——ripgrep 有 109 个未标注命令示例散布在 GUIDE.md 和 FAQ.md，bat 集中在 README.md 的 bash 块上且已标注。

### 6.3 frontmatter 在外部项目的稀缺性

| 项目 | 含 frontmatter / 其中 GitHub issue template / 真实 frontmatter |
| ---- | -------------------------------------------------------------- |
| bat | 6 / 4 / 2（DSR-3 创建的 .sih.md） |
| fd | 2 / 2 / 0 |
| ripgrep | 1 / 1 / 0 |

三份基线共 98 个 .md 文件，**真实 frontmatter 覆盖率（不含 GitHub issue template）** = 2/98 = **2.0%**。这证实了 DSR-3 bat 的观察："扩展名硬筛选是最外层治理屏障"——前 99% 的外部项目没有 SiHankor 风格的 frontmatter。

### 6.4 README.md 是宽表与缺 lang 块的双重重灾区

| 项目 | README.md 是否有 4 列表 |
| ---- | ----------------------- |
| bat | 否（最大 2 列） |
| fd | 否 |
| ripgrep | **是（6 张 4 列）** |

| 项目 | README.md 是否含大量未标注块 |
| ---- | ---------------------------- |
| bat | 否 |
| fd | 否（README 全 lang 标注） |
| ripgrep | **是（31 块未标注）** |

README.md 是项目"门面文档"，承担最多性能对比 / 快速开始示例。ripgrep 的 README.md 在 V-G-04 和 V-G-05 上同时触发——这是治理迁移摩擦最大的位置。

## 七、为什么不修复

按用户对 DSR-4 的指引"沿用 DSR-3 方法论"，而 DSR-3 fd 已经建立了"门外"策略基线（Documents delta = +0 的零覆盖真相）。DSR-4 进一步推进这个实验：

1. **零覆盖真相是干净信号**——若 DSR-4 修复 1 处，snapshot_diff 显示 Documents +0 但 Rules 统计变化，无法区分"零覆盖"和"修复无效"
2. **跨血缘近似的样本价值**——不修复 = DSR-3 fd（sharkdp 兄弟项目）和 DSR-4（BurntSushi 跨血缘）的"门外"信号可对比，证明零覆盖不依赖血缘相似
3. **保持与 bat 的策略对照**——DSR-3 bat "升级跨进门"，DSR-4 ripgrep "不修留在门外"，构成 2x2 设计（sharkdp/BurntSushi × 升级/不修）的 2 个象限

## 八、与 DSR-3 fd 的"门外"对比

| 项目 | 修复动作 | snapshot_diff Documents delta |
| ---- | -------- | ----------------------------- |
| DSR-3 fd | CONTRIBUTING.md L42 加 `changelog` 标签 | 0 |
| DSR-4 ripgrep | **无修复动作** | 0 |

| 项目 | 验证了"修复普通 Markdown 不可见" |
| ---- | ------------------------------- |
| DSR-3 fd | **是** |
| DSR-4 ripgrep | 否（跳过此验证） |

| 项目 | 验证了"完全不修复的零覆盖" |
| ---- | -------------------------- |
| DSR-3 fd | 否 |
| DSR-4 ripgrep | **是** |

DSR-3 fd 与 DSR-4 ripgrep 是 DSR "门外策略" 的两个互补角度：
- **DSR-3 fd 验证了"修复普通 Markdown 不可见"** —— 即使主动修复 1 处，Documents delta = 0
- **DSR-4 ripgrep 验证了"完全不修复的零覆盖"** —— 不触碰也能保持零覆盖

两个 DSR 一起把"门外"策略的可能性空间（修复 / 不修）都覆盖了。

## 九、建议

### 9.1 观测窗工具（方向 A）优先级

DSR-4 数据进一步确认了"陌生项目结构特征采集"是观测窗的核心价值。当前自写 Python 脚本 `observe_project.py` 输出 JSON 已能完整覆盖：

- 5 维结构特征（文件统计、目录深度、表格列数、代码块语言、frontmatter 现状）
- 5 维预测触达规则数（V-F-01 / V-F-05 / V-G-04 / V-G-05 / V-G-06）

建议 `src/observe/scanner.rs` 的 Rust 实现对齐自写 Python 的 JSON schema，使 A 任务（observe-window-mvp）的 binary 与 DSR-4 脚本能产生等价输出。

### 9.2 跨血缘近似样本积累

DSR-3 + DSR-4 三个外部项目（bat / fd / ripgrep）已覆盖：

| 项目 | 作者组 | 策略 |
| ---- | ------ | ---- |
| bat | sharkdp | 升级跨进门（策略 A） |
| fd | sharkdp | 修复但不进门（策略 B） |
| ripgrep | BurntSushi | 不修复不进门（策略 C） |

下一个 DSR 建议采样"策略 D"——一个**主动升级**的跨血缘项目（BurntSushi 或其他作者），以补全 2x2 设计的第四象限。

### 9.3 治理迁移摩擦量化

| 规则 | ripgrep 触达数 |
| ---- | -------------- |
| V-F-05 | 9 条 |
| V-G-04 | 1 文件 / 6 张表 |
| V-G-05 | 109 块 |
| **合计** | **119 条** |

ripgrep 的 109 块 V-G-05 + 9 条 V-F-05 + 1 文件 V-G-04 = **共 119 条触达违规**。这量化了"陌生项目接受司衡治理"的迁移成本——若 ripgrep 决定引入 SiHankor，需要约 100 处代码块加 lang 标签 + 9 处删水平线 + 6 张表拆列。这是治理运行时"门槛值"的第一个量化锚点。

### 9.4 零 `.sih.md` 项目的元数据

DSR-3 bat 修复后**保持**了 2 个 .sih.md，DSR-4 ripgrep 完全保持 0 个。零 .sih.md 项目不是失败状态——它是司衡治理边界外的正常状态。建议在 `src/core/indexer.rs` 的"Discovered 0"输出中加入明确提示：

```
"项目未进入治理范围（0 个 .sih.md 文档）。
 如需评估治理迁移成本，请使用 observe 工具（独立 MCP group）。
 如需引入治理，请将关键 .md 升级为 .sih.md 后重新索引。"
```

## 十、DEPS

- `docs/proposals/260628-1700-dsr-cycle-initiation.sih.md`：DSR 提案，本回顾的上游。
- `docs/knowledge/notes/260628-2100-dsr3-review.sih.md`：DSR-3 bat 回顾（跨血缘近似对照组）。
- `docs/knowledge/notes/260628-2130-dsr3-fd-review.sih.md`：DSR-3 fd 回顾（同血缘"门外"对照组）。
- `docs/proposals/260628-1700-observation-window.sih.md`：观测窗设计文档（方向 A）。
- `src/core/indexer.rs:40`：`.sih.md` 硬筛选位置（DSR-4 同样触发）。
- `src/observe/scanner.rs`：Rust 实现的观测扫描器（与 DSR-4 自写脚本同构）。
- `src/observe/predictor.rs`：规则触达预测逻辑。
- `src/core/orchestrator.rs:18-25`：`PipelineConfig` 的 `SIHANKOR_DOCS_DIR` 与 `SIHANKOR_DB_PATH` 环境变量读取（DSR-4 使用隔离 DB）。
- `/tmp/dsr4-ripgrep/`：ripgrep shallow clone（HEAD: dfe4a81d "ignore/types: add Hurl"），DSR-4 整个生命周期未修改。
- `/tmp/dsr4-scripts/observe_project.py`：DSR-4 自写观测脚本（与 src/observe/ Rust 实现同构的 Python 版本）。
- `/tmp/dsr4-scripts/{ripgrep,bat,fd}_obs.json`：三份原始观测 JSON（含每个文件级 detail）。
- `/tmp/dsr4-scripts/{ripgrep,bat,fd}_metrics.txt`：三份隔离 DB 的 MCP metrics --all 原始输出。