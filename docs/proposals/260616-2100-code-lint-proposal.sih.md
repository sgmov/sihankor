---
id: 260616-2100-code-lint-proposal
stage: 2/3
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

### 段四：反例举证（全部 26 条规则逐条检验）

#### deny 级（5 条）

| 规则 | 最强反例 | 裁定 |
|------|----------|------|
| `unsafe_code` | FFI 交互需要 unsafe 块封装（如调用 C 库） | deny（非 forbid），allow 须加 `// SAFETY: <reason>` |
| `unwrap_used` | `"42".parse::<u32>().unwrap()` — 常量解析不可能失败 | deny，`expect("by construction")` 允许 |
| `expect_used` | 同上 | deny，带构造证明的 expect 允许 |
| `dbg_macro` | 开发期临时调试——dbg! 在 1/3 阶段是合理的 | deny — ratify 代码不应残留调试痕迹（L-17） |
| `print_stdout` | 二进制程序的用户可见输出（如 CLI 的 `println!("Done.")`） | deny，二进制 main 中的操作 I/O 豁免 |

#### warn 级（17 条）

| 规则 | 最强反例 | 裁定 |
|------|----------|------|
| `non_camel_case_types` | FFI 绑定中 C 库的类型名（如 `DWORD`） | warn，FFI 模块豁免 |
| `non_snake_case` | 同上，C 常量名 | warn |
| `todo` | 1/3 阶段未完成功能——`todo!("implement X")` 是合法的意图占位 | warn — 开发期标记，ratify 前清除 |
| `unreachable` | exhaustive match 中的 safety net——当 enum 未来添加 variant 时编译器报错而非静默 panic | warn，穷举 match 中允许 |
| `unused_variables` | 模式匹配中 `_` 前缀变量——`let _unused = expr` 明确表示有意忽略 | warn |
| `too_many_arguments` | Config 结构体的构造参数、事件处理函数的多个上下文参数 | warn，阈值 7 |
| `too_many_lines` | 自动生成的代码（如 protobuf 编译产物）、穷举 match 的分支展开 | warn，阈值 200 |
| `cognitive_complexity` | 编译器生成的 match 分支、DSL 解释器的主循环 | warn，阈值 30 |
| `clone_on_copy` | 泛型上下文中 `T: Clone` 的 `x.clone()`——Copy 是 Clone 的子 trait | warn |
| `cloned_instead_of_copied` | 同上——泛型代码中 `.cloned()` 是惯用法 | warn |
| `borrowed_box` | trait object 的动态分发场景——`&Box<dyn Trait>` 在特定模式下有意义 | warn |
| `needless_return` | Rust 社区有争议——有些开发者认为显式 return 更清晰 | warn，风格偏好 |
| `unused_must_use` | fire-and-forget 场景中的 `let _ = result` 已表达有意忽略 | warn |
| `wildcard_imports` | prelude 模块的通配符导入是 Rust 惯例（如 `use super::*` in tests） | warn |
| `single_match` | match 比 if let 更清晰表达穷举意图——`if let` 隐式忽略其他分支 | warn |
| `inline_always` | 某类场景（如 `#[inline(always)]` on trivial accessors）有可测量的收益 | warn |
| `missing_docs` | trait impl 中 trait 已声明契约——重复文档是冗余 | warn，trait impl 豁免（L-15） |
| `missing_const_for_fn` | const fn 有编译器限制——不是所有逻辑上 const 的函数都能标记为 const fn | warn |

#### allow 级（4 条——默认关闭，择需开启）

| 规则 | 反例说明 | 裁定 |
|------|----------|------|
| `must_use_candidate` | 过度使用 `#[must_use]` 产生编译器噪音——有些函数的结果忽略是合法的 | allow，框架风格偏好 |
| `module_name_repetitions` | Rust 惯例 `std::io::Error` 模式——类型名重复模块名是惯用法 | allow |
| `cast_precision_loss` | 数值系统的性能权衡——`f64 as f32` 在某些计算场景中是必要的 | allow |
| `cast_possible_truncation` | 同上——协议解析中 `u64 as usize` 在 64 位平台上安全 | allow |

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

### 段七：可证伪条件设定（全部 26 条规则）

| 规则 | 证伪条件 |
|------|----------|
| `unsafe_code` | 找到一处只能用 unsafe 且无法安全封装的需求，且该需求无法通过其他架构解决 |
| `unwrap_used` | 找到一处 unwrap，其中 None/Err 在类型层面被证明不可能（而非人类声称不可能） |
| `expect_used` | 同上——expect 的证伪条件与 unwrap 相同 |
| `dbg_macro` | 找到一处 dbg! 用于正式的运行时诊断（而非临时调试），且无法用 log/tracing 替代 |
| `print_stdout` | 找到一处 println! 用于二进制程序的结构化输出（如 JSON 行），且无法用其他方式替代 |
| `non_camel_case_types` | 找到 Rust 生态中广泛使用非 CamelCase 类型的成熟项目 |
| `non_snake_case` | 同上——找到广泛使用非 snake_case 的成熟项目 |
| `todo` | 证明 todo! 在 ratify 代码中有合法残留的理由（如标记「已知限制但设计如此」） |
| `unreachable` | 证明 exhaustive match 中的 safety net unreachable! 比类型消除更优 |
| `unused_variables` | 找到 Rust 生态中 `_prefix` 约定被广泛弃用的证据 |
| `too_many_arguments` | 证明参数数量与缺陷密度无正相关（阈值 7 的实证检验） |
| `too_many_lines` | 证明函数长度与缺陷密度无正相关（阈值 200 的实证检验） |
| `cognitive_complexity` | 证明认知复杂度与缺陷密度无正相关（阈值 30 的实证检验） |
| `clone_on_copy` | 找到 Rust 生态中 Copy 类型的显式 clone 被广泛接受的证据 |
| `cloned_instead_of_copied` | 同上 |
| `borrowed_box` | 找到 &Box\<T\> 在某个常用模式中不可替代的证据 |
| `needless_return` | 找到 Rust 生态中显式 return 被广泛接受的证据 |
| `unused_must_use` | 证明忽略 Result 不是缺陷的来源（实证检验） |
| `wildcard_imports` | 找到 Rust 生态中 `use foo::*` 被广泛接受的证据 |
| `single_match` | 证明 match 单分支模式比 if let 产生更少的缺陷 |
| `inline_always` | 证明 `#[inline(always)]` 显著影响编译时间而收益可忽略 |
| `missing_docs` | 证明 doc comment 的存在与 API 的正确使用率无正相关 |
| `missing_const_for_fn` | 证明 const fn 不被广泛使用或不改善代码质量 |
| `must_use_candidate` | 证明 `#[must_use]` 的过度使用导致编译器警告疲劳 |
| `module_name_repetitions` | 证明 `std::io::Error` 模式不被 Rust 生态广泛接受 |
| `cast_precision_loss` | 证明精度损失强制检查在性能敏感场景中不可替代 |
| `cast_possible_truncation` | 同上——证明截断强制检查在协议解析中不可替代 |

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
| 禁止 unsafe（可 allow 加注） | `unsafe_code = "deny"` | L-16：unsafe 是间隙的入口——每个 unsafe 须追溯到规约中明确声明的安全边界 |
| 禁止裸 unwrap | `unwrap_used = "deny"` | L-16：间隙不得隐藏为裸 panic——unwrap 将 Option/Result 的间隙暴露为无上下文崩溃 |
| 禁止裸 expect | `expect_used = "deny"` | L-16：同上；`expect("by construction")` 允许 |
| 禁止 dbg! 残留 | `dbg_macro = "deny"` | L-17：dbg! 是开发期临时桥梁——道四：临时标记非永久建筑，ratify 代码不应残留 |
| 禁止 println! 残留 | `print_stdout = "deny"` | L-17：裸 println! 是调试痕迹或退化日志——二进制程序的用户 I/O 豁免 |

### 5.2 warn 级（Warning：应审视）

| 规则 | clippy lint | 道/法依据 |
|------|-------------|-----------|
| 非标准命名 | `non_camel_case_types = "warn"`、`non_snake_case = "warn"` | Rust 生态约定——类型/变量命名偏离惯例损害意图表达（道三） |
| todo! 开发残留 | `todo = "warn"` | L-18：开发期标记——ratify 前须清除或转为正式 issue |
| unreachable! | `unreachable = "warn"` | L-18：不可达应从类型层面保证——exhaustive match 中的安全网允许 |
| 未使用变量 | `unused_variables = "warn"` | 道三：未使用变量是意图未收敛的噪音——声明了但未使用说明意图不完全 |

| 规则 | clippy lint | 道/法依据 |
|------|-------------|-----------|
| 过多参数 | `too_many_arguments`（阈值 7） | 有度（复杂度边界） |
| 过长函数 | `too_many_lines`（阈值 200） | 有度 |
| 深层嵌套 | `cognitive_complexity`（阈值 30） | 道三（嵌套损害意图表达） |
| 无意义的 clone | `clone_on_copy`、`cloned_instead_of_copied` | 所有权模型——Copy 类型的显式 clone 是无操作噪音 |
| | &Box\<T\> | `borrowed_box` | 所有权模型 |Box\<T\> | `borrowed_box` | 所有权模型——双重间接引用无意义 |
| 冗余 return | `needless_return` | 风格偏好——Rust 以表达式结尾为惯例 |
| 未使用的 Result | `unused_must_use` | 错误处理——忽略 Result 等于隐藏间隙（道四） |
| 通配符导入 | `wildcard_imports` | 命名透明——`use foo::*` 使意图来源不可追溯（道三） |
| match 单一分支 | `single_match` | 有度——单分支 match 可用 if let 替代，减少嵌套 |
| 无效的 `#[inline]` | `inline_always` 等 | 知止（编译器比人类更懂内联） |
| pub 项无文档 | `missing_docs`（仅 pub 项） | L-15：公开 API 须声明意图——trait impl 豁免 |
| const 可常量 | `missing_const_for_fn` | 知止——编译器比人类更清楚什么可以 const；提示作为建议 |

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
unsafe_code = "deny"
unused = "warn"

[lints.clippy]
# deny — 违反即缺陷（L-16/L-17）
unwrap_used = "deny"
expect_used = "deny"
dbg_macro = "deny"
print_stdout = "deny"
# warn — 应审视（L-15/L-18/有度/知止）
non_camel_case_types = "warn"
non_snake_case = "warn"
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
todo = "warn"
unreachable = "warn"
unused_variables = "warn"
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

推进至 2/3（审阅修正完成）。待人类确认后晋升 3/3（通过 clippy 配置验证）。

### 审阅修正记录（鉴九段审阅）

| # | 发现 | 修正 |
|----|------|------|
| G1 | upstream 指向鉴（方法论工具），非法源 | upstream 改为 Canon（L-14~L-19 已就位） |
| G2 | 法层链引用 L-04（引用规则），与代码无关 | 重写为 L-14~L-19 代码术层约束准入 |
| D1 | 段四反例仅覆盖 3 条规则（旧版）；段七仅 3 类 | 段四扩展至全部 26 条逐条反例举证；段七扩展至全部 26 条可证伪条件 |
| D2 | 「术层整洁」非道层推导 | 每条 deny 规则重写道/法依据（L-16/L-17/L-18） |
| D3 | `unsafe_code = "forbid"` 无出口 | forbid→deny，allow 须加 `// SAFETY:` 注 |
| D4 | 命名约束缺失；clone 规则不完整 | 新增 `non_camel_case_types`/`non_snake_case`；完善 clone 规则说明 |

### decided-by

本提案通过鉴的九段反推法从道层推导产生，经鉴审阅修正 6 项间隙。规则已映射至 Canon L-14~L-19 法层授权。

### DEPS

- [《司衡鉴论》](../specs/philosophy/On-SiHankor-Assay.sih.md)
- [《司衡法论》](../specs/philosophy/On-SiHankor-Canon.sih.md)
- [《开发治理》](../specs/engineering/SiHankor-Dev-Governance.sih.md)
- [format-lint decision](../../decisions/260616-1930-format-lint-decision.sih.md)

### SEE-ALSO

- CI/CD 管道设计（待独立 proposal）
