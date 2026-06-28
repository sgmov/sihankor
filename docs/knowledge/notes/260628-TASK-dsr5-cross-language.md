# DSR-5: 跨语言观测窗验证

## 目标

选择并扫描一个非 Rust 开源项目，验证司衡观测窗（scanner + predictor）的跨语言泛化能力。

## 候选项目选择

从 `/tmp` 下的 preload 候选中选择满足三重去血缘的项目：
- **异语言**（非 Rust）
- **异 CLI 范式**（非命令行工具）
- **异作者群**（非 BurntSushi/sharkdp 系）

若 preload 不可用，自行选择知名开源项目并说明理由。

## 执行步骤

### 1. 项目扫描

```bash
cargo run --bin sihankor-observe -- scan <project-root>
```

收集：.md/.sih.md 文件数、5 维结构特征、5 维规则预测。

### 2. 数据对比

与 DSR-4 ripgrep 基线对比。

### 3. 写 DSR-5 review note

`docs/knowledge/notes/260628-dsr5-review.sih.md`，stage 2/3。

### 4. Ratify DSR-4 notes

确认 `260628-2200-dsr4-ripgrep-review` 和 `260628-1730-dsr4-review` 已 3/3。

## 约束

- 不修改目标项目文件
- 不调用 rebuild_index
- 只跑 sihankor-observe scan
- commit 只含司衡仓库内的 DSR note 和数据文件

## 验收

- cargo test --all-targets 全过
- DSR-5 note 存在且格式合规
- DSR-4 notes 已 3/3
