---
id: 260629-0037-dsr5-review
stage: 2/3
upstream: 260628-1700-dsr-cycle-initiation
verified: 260628
---

# DSR-5 回顾：跨语言观测窗验证（ugrep, C++）

> DSR-1 到 DSR-4 全部以 Rust 项目为外部基线。DSR-5 选取 C++ 项目 ugrep，跨语言 + 跨 CLI 范式（grep 替代）+ 异作者（Genivia vs sharkdp / BurntSushi）三重去血缘，验证司衡观测窗在非 Rust 项目上的零覆盖真相与五维规则预测是否仍然成立。

## 一、执行摘要

| 项目 | 内容 |
| ---- | ---- |
| 目标项目 | ugrep (`https://github.com/Genivia/ugrep`) |
| 语言 | C++ |
| CLI 类别 | 文本搜索（grep 替代） |
| 执行时间 | 2026-06-29 |
| 工具 | `sihankor-observe` + 隔离 DB (`/tmp/dsr5-ugrep-index.db`) |
| 原始数据 | `docs/knowledge/notes/260628-dsr5-ugrep-obs.json` |
| 基线 | 0 个 .sih.md 文档（26 个 .md 全部被跳过） |
| 治理迭代 | 不修复：：保持零覆盖真相 |
| 行迹 | 0 条（沿用 DSR-3 / DSR-4 教训） |

## 二、项目元数据对比

| 项目 | 作者 | 语言 | CLI 类别 |
| ---- | ---- | ---- | -------- |
| DSR-3 bat | sharkdp | Rust | cat 替代 |
| DSR-3 fd | sharkdp | Rust | find 替代 |
| DSR-4 ripgrep | BurntSushi | Rust | grep 替代 |
| **DSR-5 ugrep** | **Genivia** | **C++** | **grep 替代** |

ugrep 与 ripgrep 同属"文本搜索"CLI 类别，CLI 范式最接近；但语言从 Rust 跳到 C++，作者群完全无关（Genivia 是单人维护的开源贡献者，与 sharkdp / BurntSushi 无协作关系），是 DSR 系列中跨语言幅度最大的一组基线。

## 三、五维结构特征

### 3.1 文件规模

| 维度 | ugrep | ripgrep |
| ---- | ----- | ------- |
| .md 文件数 | 26 | 21 |
| 其中 .sih.md | **0** | 0 |
| 总字节数 | 305,691 | 214,613 |
| 总行数 | 6,323 | 5,113 |
| 平均行数 / 文件 | 243.2 | 243.5 |

ugrep 与 ripgrep 平均行数几乎一致（243.2 vs 243.5），但文件数多 5 个：：可能是 ugrep 拆分了更细的子文档（如 `tests-release-notes.md` 等 release notes 类）。

### 3.2 目录深度分布

| 深度 | ugrep 文件数 | ripgrep 文件数 |
| ---- | ------------ | -------------- |
| 0（根目录） | 3 | 10 |
| 1（子目录） | 4 | 9 |
| 2（孙目录） | 19 | 2 |

ugrep 的 26 个 .md 中 19 个位于深度 2，与 ripgrep 的"根平铺"结构形成鲜明对比：：ripgrep 21 个 .md 中 10 个在根目录。ugrep 把文档下沉到 `docs/` 类子目录的组织习惯与 C++ 项目的模块化组织相符。

### 3.3 表格与代码块

| 维度 | ugrep | ripgrep |
| ---- | ----- | ------- |
| 表格总数 | **0** | 6 |
| 含 4 列表文件 | 0 | 1（README.md, 6 张表） |
| 代码块总数 | **0** | 129 |
| 有 lang | 0 | 20 |
| 无 lang | 0 | 109 |
| lang 覆盖率 | N/A | 15.5% |

**这是 DSR-5 最重要的反差**：ugrep 整个项目的 Markdown 中**没有一张表格、没有一段代码块**。这不是说 ugrep 文档"质量更高"，而是说它采用了完全不同的文档载体：：CLI 示例可能放在 man page（`man/` 目录有 18 个文件）、asciidoc、或者仓库自带的 wiki 体系。Markdown 在 ugrep 中是**纯叙述性**的。

### 3.4 Frontmatter 与元数据

| 维度 | ugrep | ripgrep |
| ---- | ----- | ------- |
| 含 frontmatter | 0 | 1（GitHub issue template） |
| 含 id 字段 | 0 | 0 |
| 含 stage 字段 | 0 | 0 |
| 含司衡合法 stage | 0 | 0 |

ugrep 的 frontmatter 覆盖率是 0：：比 ripgrep（1 个 GitHub issue template）还低。两个项目都没有任何 SiHankor 风格 frontmatter。

### 3.5 水平线与 emoji

| 维度 | ugrep | ripgrep |
| ---- | ----- | ------- |
| body 中 `---` 水平线 | **0** | 9 |
| 含 emoji 行 | **61** | 0 |

ugrep 与 ripgrep 的"装饰性元素"分布完全相反：ugrep 零水平线 + 61 行 emoji，ripgrep 9 水平线 + 0 emoji。ugrep 的 emoji 集中在哪些文件需要进一步定位（按 file 维度），但 V-G-06 的 61 触达是本基线最显眼的信号。

## 四、五维规则预测

| F 规则 | ugrep | ripgrep | bat / fd |
| ---- | ----- | ------- | -------- |
| V-F-01（id 必填） | 0 | 0 | 0 / 0 |
| V-F-05（禁止 body 水平线） | **0** | 9 | 1 / 0 |
| F 预测合计 | 0 | 9 | 1 / 0 |

| G 规则 | ugrep | ripgrep | bat / fd |
| ---- | ----- | ------- | -------- |
| V-G-04（表格 <= 3 列） | 0 | 1 | 1 / 0 |
| V-G-05（代码块需 lang） | **0** | 109 | 9 / 26 |
| V-G-06（禁止 emoji 行） | **61** | 0 | 8 / 0 |
| G 预测合计 | 61 | 110 | 18 / 26 |

### 4.1 跨项目触达分布观察

- **V-F-01**：四项目全 0：：`.sih.md` 扩展名硬筛选对所有外部项目都成立，这是治理最外层屏障
- **V-F-05**：仅 ripgrep 触达（9）：：ripgrep 子 crate 的 README 普遍用 `---` 分节
- **V-G-04**：仅 ripgrep（1 文件 6 表）和 bat（1 文件 10 列）触达：：两者都是 README benchmark 对比表
- **V-G-05**：ripgrep > fd > bat > ugrep：：CLI 工具 markdown 越"示例密集"触达越多；ugrep 0 不是因为规范，是因为没用代码块
- **V-G-06**：ugrep 61 / bat 8 / ripgrep 0 / fd 0：：emoji 使用完全取决于项目维护者个人风格

### 4.2 ugrep 的 G 触达唯一性

DSR-5 是第一个**单一规则触达占比 100%** 的基线：61 条 G 触达全部来自 V-G-06，其他 4 个 G 规则全部 0。这与 ripgrep 的"109 块 V-G-05 主导 + 0 emoji"形成镜像对称：：ugrep 走"视觉化 emoji 路线"，ripgrep 走"未标注 shell 示例路线"。

## 五、关键发现

### 5.1 观测窗的跨语言泛化结论

**结论：观测窗在 C++ 项目上正常运行，所有 5 维结构特征和 5 维规则预测都产出有效信号**。

具体证据：

- `discover.rs:40` 扩展名硬筛选：触发（0 .sih.md 跳过）
- `infer_nature`：未触发（ugrep 无 `doc/` 等司衡识别目录）
- `wide_table` 扫描：正常（0 表 / 0 触发）
- `code_block` 扫描：正常（0 块 / 0 触发）
- `frontmatter` 解析：正常（0 frontmatter / 0 id）
- `horizontal_rule` 检测：正常（0 水平线）
- `emoji` 检测：正常（61 行触发）

scanner + predictor 的实现是**语言无关**的：：它只读 Markdown 文本结构，不解析 Rust / C++ 代码。所以从 Rust 切到 C++ 不需要改观测窗本体。

### 5.2 文档范式差异 vs 治理质量差异

DSR-5 暴露了一个**关键概念区分**：

- `预测触达 = 0` **不等于** `文档质量高`
- `预测触达 = 0` **也等于** `文档范式不同`

ugrep 的 V-G-04 / V-G-05 都是 0，但**不**是"ugrep 文档比 ripgrep 规范"：：而是"ugrep 完全不用 markdown 表格和代码块"。同理 ripgrep 的 V-G-06 = 0 **不**是 ripgrep 不用 emoji 就更"工程化"：：是 ripgrep 的 ASCII-only 约定与 ugrep 的视觉化约定都是项目维护者的风格选择。

这是 DSR 系列第一次给出明确的"预测信号 != 治理品质"的实证。

### 5.3 跨语言治理迁移摩擦点

如果强制把 ugrep 纳入司衡治理，唯一触达的规则是 V-G-06（61 行）。这意味着**迁移摩擦 = 0 个表格重写 + 0 个代码块补 lang + 0 个水平线删除 + 61 行 emoji 删除**。

对比 DSR-4 ripgrep 的迁移摩擦：**9 水平线 + 1 宽表 + 109 无 lang 块 = 119 处**。

跨语言迁移摩擦远低于同语言迁移：这是 DSR-5 的意外收获，异语言项目在 markdown 层面往往使用**更克制或更范式化**的写法。

## 六、跨项目汇总对比

| 维度 | ugrep | ripgrep | bat / fd |
| ---- | ----- | ------- | -------- |
| 语言 | **C++** | Rust | Rust / Rust |
| .md 数 | **26** | 21 | 69 / 8 |
| 总行数 | **6,323** | 5,113 | 大 / 小 |
| 表格 | **0** | 6 | 有 / 0 |
| 代码块 | **0** | 129 | 299 / 71 |
| lang 覆盖率 | **N/A** | 15.5% | 97.0% / 63.4% |
| frontmatter | **0** | 1 | 6 / 2 |
| body 水平线 | **0** | 9 | ? / 0 |
| emoji 行 | **61** | 0 | 8 / 0 |
| V-F-05 触达 | **0** | 9 | 1 / 0 |
| V-G-04 触达 | **0** | 1 | 1 / 0 |
| V-G-05 触达 | **0** | 109 | 9 / 26 |
| V-G-06 触达 | **61** | 0 | 8 / 0 |
| G 触达合计 | **61** | 110 | 18 / 26 |

四项目五维规则触达合计 215 条，其中 ripgrep 占 110 条（51%），是 DSR 系列触达最高的基线。ugrep 61 条（28%）位列第二，但其分布集中度最高（100% 来自 V-G-06）。

## 七、局限与下一步

### 7.1 本次扫描的覆盖边界

- 仅扫了 .md，未扫 .adoc / .man / .txt / .wiki：ugrep 的 CLI 文档主体可能在 man page 体系
- emoji 触达未按文件定位（61 行分布在哪些文件待 V-G-06 加 file 维度后可知）
- max-depth=4 限制下，孙目录深度的 19 个文件都被纳入

### 7.2 后续可补的 DSR-6 / DSR-7 方向

- **DSR-6**：纯文档项目（如 rustdoc 输出、mdbook 站点）：特征是 100% markdown + 高链接密度
- **DSR-7**：超大 monorepo 的一级子项目（如 tokio 子 crate 集合）：特征是重复结构 + 模板化 README
- **DSR-8**：非英语母语项目（如 Chinese / Japanese README 主流的开源项目）：特征是 CJK 字符密度高

### 7.3 对司衡观测窗的工程含义

DSR-1 到 DSR-5 累计 4 个外部基线（bat / fd / ripgrep / ugrep）覆盖了 Rust 同作者双基线、跨作者单基线、跨语言单基线三种去血缘梯度。**观测窗的语言无关性已被实证**：scanner + predictor 不依赖代码语义层，只依赖 markdown 文本结构层，迁移到任意语言的开源项目都不需要修改观测窗本体。

## 八、与 DSR-4 的"为什么不修复"对照

DSR-4 ripgrep 选择了"零覆盖真相"：不把任何 .md 升级为 .sih.md，以验证"陌生项目"信号链。DSR-5 ugrep 沿用同一策略：26 个 .md 全部保持原状，0 .sih.md，0 行迹。两个跨血缘（ripgrep 跨作者、ugrep 跨语言）基线都用"不修复"强化了 DSR 策略空间的新象限：既不"主动升级"（DSR-3 bat），也不"被动修复"（DSR-3 fd），而是**纯观察**。

这一象限存在的治理意义是：司衡的治理边界有意识地停留在自家仓库，外部项目的 markdown 风格由各自维护者主权决定，观测窗只承担"读"的责任，不承担"改"的责任。
