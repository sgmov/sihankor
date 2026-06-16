---
id: 260616-1745-editor-format-drift
stage: 1/3
verified: 260616
---

# 编辑器格式与文档格式漂移

> .sih.md 文件每次通过工具修改后，缩进、换行、列表前缀等格式细节与编辑器预设的格式化规则不一致，产生无意义的 diff 噪声。

## 问题

- AI 工具（如 Reasonix 的 edit_file）写入时使用空格缩进、特定换行风格
- 人类编辑器（如 VS Code + Prettier/Markdown 扩展）可能配置了 Tab 缩进、不同换行宽度、尾随空格裁剪
- 每次修改后 git diff 中混入格式变更，遮蔽实质内容差异

## 建议

1. **定义 .sih.md 的统一格式化规则**，以编辑器配置为准，工具写入适配编辑器规则
2. **在 .editorconfig 中声明**，确保跨工具一致
3. **纳入 Document-Conventions**，作为文档风格约束的工程实现部分

## 当前状态

待决策。在格式化规则确立前，每次修复后检查 `git diff --ignore-all-space` 确认无实质内容被格式噪声覆盖。
