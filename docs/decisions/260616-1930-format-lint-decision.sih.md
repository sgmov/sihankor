---
id: 260616-1930-format-lint-decision
stage: 3/3
upstream: 260628-1100-document-conventions
---
# format-lint 设计决策

## 背景

[《format-lint proposal》](../proposals/260616-1900-format-lint-proposal.sih.md) 通过四步分析法推导格式约束引擎化方案。提案经审阅发现 8 项间隙并全部闭合，现晋升至 2/3。

## 方案选择

| 维度       | 决策                               | 法依据                                          |
| ---------- | ---------------------------------- | ----------------------------------------------- |
| 架构       | 独立 binary `sihankor-fmt`         | 有度：格式域与语义域术层分离                    |
| 规则       | C-01~C-10（含 C-01a）11 条         | 法源 Document-Conventions $八                   |
| pre-commit | 默认开启，`.sih/config.yml` 可关闭 | 顺因（不执行=无约束）+ 知止（人类保有最终权限） |
| 中英混合   | Warning only                       | $8.9 原文精神                                   |

## ADR

### decided-by

人类审阅确认，提案四步分析逻辑自洽，8 项间隙已闭合。

### DEPS

- [format-lint proposal](../proposals/260616-1900-format-lint-proposal.sih.md)
- [《文档约定》](../specs/engineering/SiHankor-Document-Conventions.sih.md)
