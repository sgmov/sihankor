---
id: 260629-0900-fmt-fix-mode-decision
stage: 3/3
upstream: 260629-0900-fmt-fix-mode
decided-by: moc
---

# sihankor-fmt --fix 模式决策

## 背景

[《fmt-fix-mode 提案》](../proposals/260629-0900-fmt-fix-mode.sih.md) 提出为 sihankor-fmt 增加确定性修复能力：Tier 1 从文件 mtime 取 HHMM 修 V-F-01，Tier 2 级联更新引用。提案经审阅修正 3 项事实错误后晋升 2/3：

- "14 条 Fatal 规则"更正为 6 条（Fatal 级仅 V-F-01/03/04/05/06/07）
- Cargo.toml 已含 chrono 依赖，时区处理无需新增
- rebuild_index 的 prune 标志尚未落地，dry-run 基线改为引用未落地依赖

## 方案选择

| 维度     | 决策                                     | 法依据                         |
| -------- | ---------------------------------------- | ------------------------------ |
| 修复范围 | 仅 V-F-01 的 HHMM 缺失                   | 有度：只修可机械判定的违规     |
| 分层     | Tier 1 改 id，Tier 2 级联引用            | 知止：确定性修复与引用更新解耦 |
| 写盘护栏 | --cascade 默认 dry-run，--confirm 才写盘 | 顺因：人类保有最终权限         |
| 数据源   | 文件 mtime（非 Local::now）              | 顺因：反映落盘时刻             |
| 日期校验 | mtime 日期须与 id YYMMDD 一致            | 知止：失真时跳过并 warn        |

## ADR

### decided-by

moc 审阅确认：提案修正 3 项事实错误后逻辑自洽，分层修复（Tier 1/Tier 2）与 dry-run 护栏符合知止原则，决议采纳。

### DEPS

- [fmt-fix-mode 提案](../proposals/260629-0900-fmt-fix-mode.sih.md)
- [format-lint 决策](./260616-1930-format-lint-decision.sih.md)
- [rebuild-stale-prune 提案](../proposals/260629-0131-rebuild-stale-prune.sih.md)
