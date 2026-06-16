---
id: 260616-1214-engine-mvp-parser
stage: 1/3
upstream:
  - 260611-0000-sihankor-engine-design-summary
  - 260616-1200-sihankor-dev-governance
---

# Engine MVP: frontmatter 解析器

> engine 自举的起点。实现 .sih.md 的 frontmatter 解析能力，使其能读取自身 docs/ 中的文档并执行基本校验。

## 一、解决的 gap

[GAP: 引擎核心模块未实现 — 阻塞所有文档解析和 frontmatter 校验]

## 二、方案对比

### 方案一：serde_yaml + 自定义 deserializer

- 优点：生态成熟，YAML 解析稳健，错误信息可读
- 缺点：依赖 serde 全家桶，编译体积较大
- 社区：serde_yaml 是 Rust 生态中 YAML 的事实标准

### 方案二：手写最小 YAML 解析器

- 优点：零依赖，编译体积最小，完全控制
- 缺点：只支持 frontmatter 场景的 YAML 子集，扩展成本高，边界 case 容易出错

### 方案三：yaml-rust2

- 优点：纯 Rust 实现，无 unsafe，API 简洁
- 缺点：社区相对较小，文档不如 serde_yaml 丰富

**推荐方案一**。理由：serde_yaml + serde 可以顺带用于解析 config.yml 和 semantic.yml，投入一份依赖解决三种 YAML 解析场景（知止：不做重复投入）。编译体积在 MCP server 场景中不是瓶颈。

## 三、验收标准

1. 解析 `docs/` 下所有 .sih.md 的 frontmatter，提取 `id`、`stage`、`upstream`、`successor` 字段
2. 对 field 级别的格式错误（如 stage 不是合法值、id 格式不匹配）输出带行号的错误
3. 解析失败不崩溃——返回 Partial 结果 + 错误列表（道四：解析器也可能出错）
4. 解析速度：docs/ 下当前 25 份文档 < 50ms

## 四、对规约的影响

- Engine-Design-Summary $二.1 文档实体：frontmatter 解析字段展开为 struct 定义（已在设计摘要中）
- 需要补充：Document 实体的 `DocStatus` 枚举增加解析失败状态

## 五、实现模块

```
src/
  parser/
    mod.rs           # 模块入口
    frontmatter.rs   # frontmatter 解析 + 校验
    document.rs      # 完整文档解析（frontmatter + 正文）
    error.rs         # 解析错误类型
```

## 六、semantic.yml 映射（实现后填充）

```yaml
frontmatter-parser:
  derives-from: 260616-1214-engine-mvp-parser
  code:
    - symbol: parse_frontmatter
      location: src/parser/frontmatter.rs
      fidelity: direct
```

## 七、引导阶段声明

本提案走引擎开发治理链。因 engine 尚未实现，第五步（自动验证 + semantic.yml 填充）待 parser 自身完成后由它对自己执行——循环自举。验证方式：parser 解析自身的 proposal 文档（即本文），输出 frontmatter 解析结果，人类对比验证。
