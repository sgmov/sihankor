---
id: 260611-0000-sihankor-engine-design-summary
type: mapping
stage: 2/3
upstream: 240610-1030-on-sihankor-canon
---

# 司衡引擎设计摘要

> 供外部协作者审阅。归纳自项目源码、哲学文档、对话历史。
> 整理时间：2026-06-11。

---

## 一、引擎的整体定位

**司衡引擎是什么**：一个承认治理自身也不完备的代码工程收敛引擎。它不是外加于代码工程的统治者，而是代码工程自己长出来的收敛机制。

**解决什么问题**：代码工程中意图天然发散（道一），需要系统的、持续的治理力量介入才能收敛。引擎将"发散自然，收敛必为"这一哲学主张工程化，提供文档治理、认知分析、合规验证的能力。

**在 AI 辅助开发体系中的位置**：引擎通过 MCP（Model Context Protocol）暴露**治理能力接口**（非被动工具箱），供 AI IDE Agent（Cursor、Claude Code、Cline 等）调用。Agent 负责"何时调用"（when），引擎负责"怎么执行"（how）。

AI 成为治理能力的自然载体，源于道一的必然推论：代码工程的发散规模已超出纯人类治理的能力边界。AI 不是被选中的工具，而是从治理需求中涌现的必然选择。这一判断经过了 Decision #42 的校准：治理器官的角色归属于司衡引擎自身，AI 的角色从治理主体重新定位为治理能力的工程载体与调用者。

> 来源：对话中已讨论（MCP 治理能力接口设计）、哲学文档已定义

---

## 二、核心实体与数据模型

### 2.1 文档（核心实体）

文档是引擎治理的基本单元。纯 Markdown 格式，可选最小 frontmatter。

**必填字段**：

| 字段    | 类型   | 说明                                                                     |
| ------- | ------ | ------------------------------------------------------------------------ |
| `id`    | string | 格式 `YYMMDD-HHMM[-NNN]-语义短名`，一经分配永久不变                      |
| `type`  | enum   | 写作意图：`treatise`(论) / `compendium`(纲) / `mapping`(映) / `note`(记) |
| `stage` | string | 生命周期状态编码                                                         |

**可选字段**：`upstream`（治理授权源）、`domain`（领域归属）

### 2.2 文档状态机

```
1/3 (propose) → 2/3 (resolve) → 3/3 (ratify)
                                    ↓ 道四触发
                                  2/3 (reopen)
                                    ↓ 不可修
                                  0/successor-id (supersede)
1/3 → X (deprecate)
2/3 → X (deprecate)
note: 1/3 → 2/3(自动) → 3/3(晋升) → 0/decayed(衰减)
```

| 编码 | 含义                       | 下游引用权限               |
| ---- | -------------------------- | -------------------------- |
| 1/3  | 提案/草稿                  | 不可引用                   |
| 2/3  | 决议/成熟                  | 可引用但需注明阶段         |
| 3/3  | 定稿/晋升                  | 可引为可靠依据             |
| 0/   | 权威归零，后继 id 紧随 `/` | 保留 ratify 时的引用有效性 |
| X    | 终止                       | 禁止引用                   |

### 2.3 术语表（Glossary）

三层结构：

| 文件            | 职责                                                            | 语言相关性 |
| --------------- | --------------------------------------------------------------- | ---------- |
| `_concepts.yml` | 概念注册表，记录 derives-from 和 related                        | 语言无关   |
| `zh.yml`        | 中文权威定义（概念语义的唯一权威源）                            | 中文       |
| `en.yml`        | 英文映射（mapping）、被拒词（rejected）、消歧（disambiguation） | 英文       |

因果方向不可逆：`specs/` → `reference/` → `glossary/`。reference 变更必须传播到 glossary，glossary 变更不反向影响 reference。

### 2.4 存储方案

#### 2.4.1 概述

- **默认**：SQLite（文件 `.sih/index.db`），驱动库 rusqlite
- **可切换**：PostgreSQL、图数据库、向量数据库、Redis、MongoDB 等均通过 `SihDatabase` trait 独立实现，各后端自选最合适的驱动库
- **全文搜索**：SQLite FTS5 原生支持；初始使用 LIKE，为 FTS5 预留迁移路径
- **道四提醒**：LIKE 有固有召回率限制，搜索准确性不是 100%

#### 2.4.2 SihDatabase trait

```rust
#[async_trait]
pub trait SihDatabase: Send + Sync {
    async fn upsert_document(&self, doc: Document) -> Result<()>;
    async fn get_document(&self, id: &str) -> Result<Option<Document>>;
    async fn search_by_type(&self, doc_type: &str) -> Result<Vec<Document>>;
    async fn search_content(&self, query: &str) -> Result<Vec<SearchResult>>;
    async fn resolve_chain(&self, id: &str, depth: u32) -> Result<Vec<ChainNode>>;
}
```

**设计依据（顺因→有度→知止）**

- trait 标记 `async`：契约定义"做什么"，不泄漏"怎么做"。SQLite 的 `spawn_blocking` 是 `SqliteBackend` 的私有实现细节，不是契约的一部分。未来 Redis/MongoDB 原生 async 后端可直接实现，无需反向 `block_on`
- trait 方法稳定性顺序：`upsert_document`（写）→ `get_document`（单读）→ `search_*`（多读）→ `resolve_chain`（关系查询），按依赖和复杂度递增
- `resolve_chain` 的 `depth` 参数防止递归爆炸（知止：不假装可以无限追溯）

#### 2.4.3 Document：核心实体

```rust
pub struct Document {
    pub id: String,
    pub r#type: DocType,
    pub stage: Stage,
    pub title: String,
    pub upstream: Option<String>,
    pub frontmatter: Frontmatter,
    pub content: String,
    pub status: DocStatus,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
}
```

**字段设计依据**

| 字段          | 设计决策                                     | 道法依据                                               |
| ------------- | -------------------------------------------- | ------------------------------------------------------ |
| `content`     | 解析后正文，不含 frontmatter 和 `---` 分隔符 | 损补（从博返约）；知止（数据库是约层，Git 是形迹层）   |
| `frontmatter` | 展开为结构体，非 blob                        | 顺因（parser 产出被下游直接使用，不重复解析）          |
| `status`      | 带解析/验证状态                              | 道四（引擎承认不完备，解析失败的文档也需被记录和看见） |

#### 2.4.4 Frontmatter：结构化 + 兜底

```rust
pub struct Frontmatter {
    pub id: String,
    pub r#type: DocType,
    pub stage: Stage,
    pub upstream: Option<String>,
    pub decided_by: Option<String>,
    pub extra: serde_json::Value,
}
```

**设计依据（有度→知止→道四）**

- 只展开引擎需要查询、索引、验证的字段（有度：恰好够用）
- `extra: serde_json::Value` 兜底未定义字段，不丢失数据（道四：结构体定义不完备，未来可能新增字段；Document-Conventions 仍在 2/3）
- 不用 `serde_json::Value` 存全部 frontmatter（知止：那等于没定义 schema）

#### 2.4.5 DocStatus：三级解析/验证状态

```rust
pub enum DocStatus {
    Ok,      // 解析和验证均通过
    Warning, // 有违规但不阻断，可继续治理
    Error,   // 解析失败或严重违规，阻塞治理
}
```

**设计依据（道四→道三→损补）**

- 道四核心：引擎承认不完备。只存"验证通过"的文档等于假装引擎不会出错
- 道三：解析失败的文档是"代码自晦"的实例，应存下来标记状态，供未来改进的 parser 照亮
- 损补：记录失败状态是"补缺失"——让断裂的文档在治理体系中有一个位置
- 知止：三个状态够用，不建微状态机

#### 2.4.6 文档表 SQL Schema

```sql
CREATE TABLE documents (
    id          TEXT PRIMARY KEY,
    type        TEXT NOT NULL,
    stage       TEXT NOT NULL,
    title       TEXT NOT NULL,
    upstream    TEXT,
    frontmatter_json TEXT NOT NULL,  -- Frontmatter 序列化
    content     TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'Ok',
    indexed_at  TEXT NOT NULL
);

CREATE INDEX idx_documents_type ON documents(type);
CREATE INDEX idx_documents_stage ON documents(stage);
CREATE INDEX idx_documents_upstream ON documents(upstream);
CREATE INDEX idx_documents_status ON documents(status);
```

**设计要点**

- `frontmatter` 在 Rust 侧是强类型结构体，SQL 侧存 JSON：类型安全在应用层，存储弹性在数据库层
- `upstream` 可 NULL：note 类型无上游；根级文档用全大写域标识（如 `PHILOSOPHY`），不存为 NULL
- `stage` 存字符串（如 `1/3`、`2/3`、`3/3`、`0/`、`X`），非枚举：stage 编码是法层定义，可能扩展（有度：不过度规约）

> 来源：对话中已讨论（数据库选型、SihDatabase trait 设计）、设计文档已定义（文档约定、法论）

---

## 三、架构分层与模块职责

### 3.1 六层脉络架构

```
元（构成性条件）
  ↓ 使…得以成立
道（因果必然性）
  ↓ 自然生出
法（方法论原则：收敛五法）
  ↓ 御
术（工程化展开：Spec-Coding、三机、F/G/J）
  ↓ 因需生
几（执行层：iCL / iWW / iCT）
  ↓ 因博生
约（信息压缩：SymBrief + DocBrief）
  ↓ 因约取
形迹（可观测产物：文档、代码、索引）
```

### 3.2 引擎模块划分（Rust crate 规划）

当前 Rust 代码处于初始骨架阶段。基于对话和设计文档，规划的模块结构：

| 模块                | 职责                                                                         | 对应六层 |
| ------------------- | ---------------------------------------------------------------------------- | -------- |
| `core/parser`       | Markdown 解析：frontmatter 提取、正文区域识别                                | 术-约    |
| `core/validator`    | 六域验证规则（frontmatter/structure/content/reference/lifecycle/governance） | 术-几    |
| `core/indexer`      | 文档索引：discover → parse → SQLite                                          | 术-约    |
| `core/orchestrator` | 管道编排：parse → validate → index                                           | 几       |
| `mind/tools`        | 三机工具实现（iCL/iWW/iCT）                                                  | 几       |
| `mcp_server`        | MCP 协议层：暴露治理能力接口                                                 | 形迹     |
| `common`            | 通用 trait 和服务抽象                                                        | 跨层     |

### 3.3 当前代码状态

Rust 引擎已建立骨架：

- `Cargo.toml`：依赖 `rmcp 1.7.0`（MCP SDK）、`serde`、`tokio`
- `src/main.rs`：启动 MCP server（stdio 模式）
- `src/common/generic_service.rs`：通用 MCP 服务骨架，含 `DataService` trait 和 `GenericService` 泛型服务
- `GenericService` 已实现两个 MCP 工具：`get_data` / `set_data`（占位实现）

Python 旧版引擎（参考实现，将被 Rust 替代）包含完整的 parser、validator、indexer、orchestrator 和 mind 模块。

> 来源：源码已实现（Rust 骨架）、设计文档已定义（工程映射）、对话中已讨论（Rust 迁移）

---

## 四、核心接口/特质

### 4.1 已定义的 trait

**`DataService` trait**（源码已实现）：

```rust
pub trait DataService: Send + Sync + 'static {
    fn get_data(&self) -> String;
    fn set_data(&mut self, data: String);
}
```

当前实现：`MemoryDataService`（内存存储，占位用）。

### 4.2 已规划的 trait（对话中讨论，尚未实现）

**`SihDatabase` trait**（数据库抽象）：完整定义见 [$2.4.2、SihDatabase trait](#242-sihdatabase-trait)。

实现：`SqliteBackend`（默认，rusqlite）、`PostgresBackend`（可切换，sqlx 或 diesel）。未来可扩展图数据库（Neo4j）、向量数据库（Qdrant）、Redis、MongoDB 等后端——各后端独立实现 trait，自选最优驱动库，不借 SQL 统一抽象层。

> 来源：源码已实现（DataService）、对话中已讨论（SihDatabase）

---

## 五、与 MCP 的关系

### 5.1 定位：治理能力接口

引擎通过 MCP 暴露的是**治理能力**，不是原子工具零件。三机的编排逻辑（iCL→iWW→iCT）是道层决定的治理必然性，留在引擎内部，不外部化。

### 5.2 治理引擎 MCP（6 个工具）

| 工具             | 职责                   |
| ---------------- | ---------------------- |
| `validate_sihmd` | 验证文档合规性         |
| `search_docs`    | 搜索已索引文档         |
| `get_document`   | 获取文档元数据和结构   |
| `resolve_chain`  | 追溯授权链（递归 CTE） |
| `project_status` | 项目治理概览           |
| `index_rebuild`  | 触发全量索引重建       |

### 5.3 思维核心 MCP（4 个工具）

| 工具               | 三机        | 编排     | 职责                           |
| ------------------ | ----------- | -------- | ------------------------------ |
| `analyze_document` | iCL         | 无       | 六层定位 + 关系图谱 + 发散诊断 |
| `propose_decision` | iCL→iWW     | 引擎内部 | 认知分析 + 生成决策建议        |
| `verify_decision`  | iCT         | 无       | 五法检验 + 合道性验证          |
| `full_analysis`    | iCL→iWW→iCT | 引擎内部 | 全流程治理分析                 |

### 5.4 工具分级

- **原子工具**（analyze_document、verify_decision）：给需要精细控制的场景
- **编排工具**（propose_decision、full_analysis）：给需要完整治理的场景

所有思维核心 MCP 工具**只读**，不修改文档（符合识出接口定义）。

> 来源：对话中已讨论（MCP 治理能力接口设计）、设计文档已定义（mind 提案）

---

## 六、度量与自指

### 6.1 度量指标

对话中推演了三个动态收敛度量指标，尚未实现：

| 指标                    | 含义                               | 体现之法    |
| ----------------------- | ---------------------------------- | ----------- |
| GDT（间隙密度变化趋势） | 单位周期内未闭合间隙数量的变化方向 | 损补 + 顺势 |
| GDD（间隙债务深度）     | 未闭合间隙被下游引用的加权平均时间 | 损补 + 有度 |
| BF（回溯频率）          | ratify 文档被 Reopen 的频率        | 损补 + 顺势 |

### 6.2 元治理

引擎自身被纳入治理：

- 道四递归适用：引擎的规约（验证规则、解析规则）同样是有损编码，判断同样可能出错
- 三机输出都不是绝对正确的：iCT 的"通过"不等于没有问题，iWW 的"阻断"不等于必须修改
- 人工例外机制：@deviation 声明 + 强制理由记录 + 定期审查
- 思维核心的三个硬约束：只分析不写入、只建议不决定、只自知不假装

> 来源：对话中已推演（收敛度量框架）、哲学文档已定义（道四自指、mind 约束）

---

## 七、与司衡哲学的道法映射

### 7.1 引擎设计点到道/法的映射

| 设计点                                 | 依据                                          |
| -------------------------------------- | --------------------------------------------- |
| MCP 暴露治理能力而非原子工具           | 道一（编排是治理，不是组装）                  |
| 文档纯 Markdown + 最小 frontmatter     | 知止（智能在引擎，简单在文档）                |
| 三机编排留在引擎内部                   | 道一（iCL→iWW→iCT 由道决定）                  |
| MCP 工具只读外服                       | 道三（识出接口只读）                          |
| 不引入内嵌 Agent                       | 道四（Agent Loop 是 AI IDE 职责，非引擎职责） |
| SQLite 默认 + PostgreSQL 可切换        | 有度（恰到好处，不过度工程）                  |
| 初始不引入 FTS5                        | 知止（不需要的先不做）                        |
| 三阶生命周期（propose→resolve→ratify） | 顺势（力度递增）                              |
| Reopen / Supersede 修正机制            | 道四（间隙需持续修正）                        |
| Glossary 三层结构 + 因果方向不可逆     | 顺因（reference → glossary）                  |
| SihDatabase trait 抽象                 | 有度（不同规模不同选择）                      |
| @limitations + @deviation              | 道四（承认不完备）                            |
| F/G/J 三级力度体系                     | 有度（戒/规/矩）                              |

### 7.2 尚未映射的设计

- `GenericService` 的 `get_data` / `set_data` 占位工具：无道法映射，属于初始骨架代码，将在实际引擎实现中被替换

> 来源：对话中已讨论、哲学文档已定义、工程映射文档已定义

---

## 八、已识别的间隙

### 8.1 待后续收敛

| 间隙                              | 原因                                      | 未来收敛路径                                 |
| --------------------------------- | ----------------------------------------- | -------------------------------------------- |
| Rust 引擎模块划分尚未实现         | 当前只有 `common/generic_service.rs` 骨架 | 从 Python 旧版迁移，按规划的模块结构逐步实现 |
| SihDatabase trait 未实现          | 对话中已设计，代码未编写                  | 作为 Rust 迁移的核心模块优先实现             |
| 三机（iCL/iWW/iCT）Rust 实现为空  | Python 版有骨架实现，Rust 版未开始        | 依赖 parser + indexer 先完成                 |
| 收敛度量指标（GDT/GDD/BF）未实现  | 对话中推演了概念框架，属于术层设计        | 需要先有实际治理数据积累，再实现度量         |
| SymBrief / DocBrief（约系）未实现 | 属于"约"层的工程实现                      | 在 parser 和 indexer 之后实现                |
| 多 Agent 协作治理主权模型未实现   | 对话中推演了概念框架                      | 属于未来场景，当前单 Agent 场景不涉及        |

### 8.2 已声明的可证伪预设

- **道四递归适用于引擎自身**：引擎的验证规则可能有覆盖不到的边界情况。当前无法枚举所有边界，需在实践中发现
- **GDT/GDD/BF 阈值无法静态标定**：任何阈值设定都是有损编码，需在具体项目中通过实践校准
- **"语义等价性不可完备判定"**：本摘要中引用了这一形式化结论，如果未来出现可完备判定语义等价性的方法，相关论证需要修正

> 来源：对话中已讨论、哲学文档已声明（道四不完备、鉴论自指）
