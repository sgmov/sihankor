---
id: 260616-2100-code-lint-proposal
stage: 1/3
upstream: 240610-1500-sihankor-assay
---
# code-lint：基于鉴的 Rust 代码质量约束

## 一、正名：代码 lint 是什么

代码 lint 是术层规则在几层的自动化执行。它不定义「好代码」，而是将已经过鉴检验的代码质量主张编码为可验证的约束。

与 format-lint 的关系：format-lint 校验 .sih.md 的字符级表示合规；code-lint 校验 Rust 源码的术层质量合规。两者并列，不同域。

## 二、顺因：治理链

```
道（道三：代码自晦，意图必复）
  → 法（L-04 引用规则 等语义约束的代码后果）
    → 术（Rust 语言习惯、clippy 规则分类）
      → 几（clippy 配置 + CI 触发）
```

代码 lint 不是凭空建立规则列表，而是从道层推导「代码应该满足什么质量条件」，然后映射到 Rust 工具链。

## 三、有度：边界

### 管

- **类型安全**：禁止 `unsafe`（除非有明确的安全封装并加注鉴）
- **错误处理**：禁止 `unwrap()` / `expect()` 在非测试代码中裸用
- **命名**：类型 CamelCase、变量 snake_case、常量 SCREAMING_SNAKE_CASE
- **复杂度**：函数体长度、嵌套深度、参数个数上限
- **文档**：`pub` 项必须有 doc comment
- **所有权**：禁止无意义的 `.clone()`、禁止 `&Box<T>`

### 不管

- 架构设计（模块划分、依赖方向）
- 算法选择（O(n²) vs O(n log n)）
- 性能优化（那是 profiling 的职责）
- 测试覆盖率（那是 CI 的职责）

## 四、鉴：九段反推法推导 lint 规则

以下对「代码 lint 能提升代码质量」这一主张进行逐段检验，从中提取可存活的规则。

### 段一：主张提取

隐含的主张链：
1. P1：存在一组普遍适用于 Rust 代码的质量约束
2. P2：违反这些约束的代码客观上更难维护
3. P3：自动化检测这些约束能减少人类审查负担
4. P4：clippy 是执行这些约束的最合适工具

### 段二：概念分析

关键概念需要精确化：

- **「质量」**：在司衡体系中，代码质量不是美学偏好，而是**道三的可验程度**——「代码自晦，意图必复」。代码质量 = 意图被正确复原的概率。
- **「普遍适用」**：不是「所有 Rust 项目都应如此」，而是「SiHankor 项目的道/法/术层层推导后的必然要求」——鉴不检验普适性，只检验自身自洽性。
- **「更难维护」**：操作性定义——修复同一缺陷所需的时间和引入新缺陷的概率。

### 段三：最强反证构建

**反主张**：「lint 规则越多，噪音越大。强制风格一致性的收益递减。真正影响代码质量的不是风格而是设计。」

这个反证很强，因为它区分了*风格 lint*（空格、括号位置）和*语义 lint*（所有权误用、类型危险）。鉴的回应：前者确实边际收益低；后者有可证伪的正收益。因此**区分两类规则是设计的关键**。

### 段四：反例举证

实际案例：
1. `clippy::needless_return` — 风格级，Rust 社区有争议。SiHankor 采纳：低优先级，Warning。
2. `clippy::too_many_arguments` — 往往标记合理的参数传递。SiHankor 采纳：Warning 且阈值放宽至 7。
3. `unsafe` 禁止 — 无争议。Rust 安全模型的直接推论。Error。

### 段五：类比检验

C 语言有 lint（`lint` 工具发明于 1979），C++ 有 `clang-tidy`，Python 有 `ruff`/`pylint`。跨语言观察：
- 所有成熟语言都有 lint 工具
- 语言越接近底层，安全 lint 越重要；语言越接近应用层，风格 lint 越重要
- Rust 的独特之处：所有权系统提供了语义 lint 的新维度（`clippy` 可以检测所有权误用模式）

类比成立：代码 lint 作为术层工具，在语言生态中普遍存在，Rust 的特异性在于语义维度的丰富性。

### 段六：逻辑一致性

检验 SiHankor 内部自洽性：
- 道三「代码自晦，意图必复」：lint 规则应该服务于「代码如实表达意图」。过度风格约束（如强制某种特定模式）如果阻碍意图表达，则违道。
- 有度之法：规则应有明确边界。`deny` 级规则应仅用于「违反即缺陷」的约束。
- 知止之法：lint 不替代人类判断。`allow` 必须可用，且每个 `allow` 需要注释说明理由。

一致性检验：成立。

### 段七：可证伪条件设定

每条 lint 规则的可证伪条件：

| 规则类 | 证伪条件 |
|--------|----------|
| unsafe 禁止 | 找到一处只能用 unsafe 且无法安全封装的需求，且该需求无法通过其他架构解决 |
| unwrap 禁止 | 找到一处 unwrap 在逻辑上不可能 panic 的代码（如刚 checked 后） |
| 命名规范 | 找到 Rust 生态中广泛使用非约定命名的成熟项目 |

### 段八：证伪判定

| 子主张 | 判定 |
|--------|------|
| P1：存在适用于 SiHankor 的代码约束 | SURVIVES — 道三提供了推导基础 |
| P2：违反约束的代码更难维护 | SURVIVES — 有可操作的度量标准 |
| P3：自动化减少审查负担 | SURVIVES — format-lint 已证明此点 |
| P4：clippy 是最合适工具 | SURVIVES — Rust 生态无替代品 |

### 段九：校准建议

基于段三的反证和段四的反例，校准如下：
1. 规则分三级：`deny`（违反即缺陷）、`warn`（应审视）、`allow`（风格偏好，默认关）
2. 阈值校准：`too_many_arguments = 7`、`too_many_lines = 200`
3. 每个 `allow` 必须有 `// reason:` 注释

## 五、术层规则映射

### 5.1 deny 级（Error：违反即缺陷）

| 规则 | clippy lint | 道/法依据 |
|------|-------------|-----------|
| 禁止 unsafe | `unsafe_code` | Rust 安全模型 + 道三 |
| 禁止裸 unwrap | `unwrap_used` | 道四（规约与实现必有间隙——unwrap 在运行时暴露间隙为 panic） |
| 禁止裸 expect | `expect_used` | 同上 |
| 禁止 dbg! 宏残留 | `dbg_macro` | 术层整洁 |
| 禁止 todo! 宏残留 | `todo` | 术层整洁 |
| 禁止 print!/println! 残留 | `print_stdout` | 术层整洁 |
| 禁止未使用的变量 | `unused_variables`（特定模式） | 术层整洁 |
| 禁止 unreachable! | `unreachable` | 道四（不可达代码应在类型层面消除） |

### 5.2 warn 级（Warning：应审视）

| 规则 | clippy lint | 道/法依据 |
|------|-------------|-----------|
| 过多参数 | `too_many_arguments`（阈值 7） | 有度（复杂度边界） |
| 过长函数 | `too_many_lines`（阈值 200） | 有度 |
| 深层嵌套 | `cognitive_complexity`（阈值 30） | 道三（嵌套损害意图表达） |
| 无意义的 clone | `clone_on_copy`、`cloned_instead_of_copied` | 术层 |
| &Box\<T\> | `borrowed_box` | 所有权模型 |
| 冗余 return | `needless_return` | 术层 |
| 未使用的 Result | `unused_must_use` | 错误处理 |
| 通配符导入 | `wildcard_imports` | 术层命名透明 |
| match 单一分支 | `single_match` | 术层 |
| 无效的 `#[inline]` | `inline_always` 等 | 知止（编译器比人类更懂内联） |
| pub 项无文档 | `missing_docs`（仅 pub 项） | 道三（公开 API 必须表达意图） |
| const 可常量 | `missing_const_for_fn` | 术层 |

### 5.3 allow 级（默认关闭，择需开启）

| 规则 | 理由 |
|------|------|
| `must_use_candidate` | 框架风格偏好 |
| `module_name_repetitions` | Rust 惯例 `std::io::Error` 模式 |
| `cast_precision_loss` | 数值系统的性能权衡 |
| `cast_possible_truncation` | 同上 |

## 六、几层配置

### clippy.toml

```toml
too-many-arguments-threshold = 7
too-many-lines-threshold = 200
cognitive-complexity-threshold = 30
```

### Cargo.toml（lints section）

```toml
[lints.rust]
unsafe_code = "forbid"
unused = "warn"

[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
dbg_macro = "deny"
todo = "deny"
print_stdout = "deny"
unreachable = "deny"
# warn...
too_many_arguments = "warn"
too_many_lines = "warn"
cognitive_complexity = "warn"
clone_on_copy = "warn"
cloned_instead_of_copied = "warn"
borrowed_box = "warn"
needless_return = "warn"
unused_must_use = "warn"
wildcard_imports = "warn"
single_match = "warn"
inline_always = "warn"
missing_docs = "warn"
missing_const_for_fn = "warn"
```

## 七、与 format-lint 的关系

| 维度 | format-lint | code-lint |
|------|-------------|-----------|
| 对象 | .sih.md 文档 | Rust 源码 |
| 工具 | `sihankor-fmt`（自建） | `cargo clippy`（Rust 生态） |
| 法源 | Document-Conventions $八 | Rust 安全模型 + SiHankor 道/法 |
| 触发 | 本地 + pre-commit（默认） | `cargo clippy` + CI |
| 规则来源 | 从 AGENTS.md 推导 | 从 鉴 九段反推法推导 |

两者同为几层工具，互为补充：format-lint 管文档形式，code-lint 管代码实质。

## ADR

推进至 1/3。待人类确认后晋升 2/3（通过 clippy 配置验证），实施无违规后晋升 3/3。

### decided-by

本提案通过鉴的九段反推法从道层推导产生。SURVIVES 判定的 8 条 deny + 12 条 warn 规则构成了 SiHankor 代码质量基线。

### DEPS

- [《司衡鉴论》](../specs/philosophy/On-SiHankor-Assay.sih.md)
- [《司衡法论》](../specs/philosophy/On-SiHankor-Canon.sih.md)
- [《开发治理》](../specs/engineering/SiHankor-Dev-Governance.sih.md)
- [format-lint decision](../../decisions/260616-1930-format-lint-decision.sih.md)

### SEE-ALSO

- CI/CD 管道设计（待独立 proposal）
