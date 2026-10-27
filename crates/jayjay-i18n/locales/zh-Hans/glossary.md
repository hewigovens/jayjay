# 简体中文术语表

本表约定 JayJay 简体中文界面里的核心术语。新增文案时优先沿用这里的译法；确需改动时，同时更新本表。

## 基本原则

- `JayJay` 是产品名，任何语言都不翻译。
- `jj`、命令名、revset 语法、配置键、文件路径、URL、快捷键名保持原样。
- 菜单和按钮用动词短语，不加句号；说明文字用完整句，句末加句号。
- 中文与拉丁字母、数字之间留一个半角空格，例如 `jj 仓库`、`3 个文件`。
- 使用全角标点：`，`、`。`、`：`、`；`、`？`、`！`；省略号统一用 `…`。
- 界面引用的操作或配置名使用全角双引号：`“完成”`、`“压缩到父提交”`，不用西文直引号 `"`。
- 中文语境下的补充说明统一使用全角括号：`保存说明（jj describe）`、`关闭注释视图（esc）`。
- 占位符两边保留空格，例如 `{ $count } 个文件`。

## 核心术语

| English | 简体中文 | 说明 |
| --- | --- | --- |
| change | 变更 | jj 的核心概念，指会随时间演化的提交 |
| change ID | 变更 ID | |
| commit | 提交 | |
| commit ID | 提交 ID | |
| revision | 修订 | 与 commit 同义时按上下文使用 |
| revset | revset | 不翻译，保持小写 |
| bookmark | 书签 | |
| tag | 标签 | |
| working copy | 工作副本 | 与“工作区”区分 |
| working-copy commit | 工作副本提交 | |
| workspace | 工作区 | 对应 Git 的 worktree |
| operation log / oplog | 操作日志 | |
| evolution log / evolog | 演化日志 | 界面上也用“演化” |
| immutable | 不可变 | |
| mutable | 可变 | |
| divergent | 分叉 | |
| conflict | 冲突 | |
| trunk | 主干 | |
| parent / child | 父提交 / 子提交 | |
| ancestor / descendant | 祖先 / 后代 | |
| describe | 编辑说明 | 命令 `jj describe` 不译 |
| abandon | 放弃 | |
| squash | 压缩 | |
| split | 拆分 | |
| rebase | 变基 | |
| absorb | 吸收 | |
| duplicate | 复制 | |
| revert | 还原 | |
| undo | 撤销 | |
| restore | 恢复 | |
| forget | 忘记 | |
| bisect | 二分查找 | |
| fetch / push / pull | 获取 / 推送 / 拉取 | |
| pull request | Pull Request | 保持英文，与 GitHub 界面一致 |
| remote | 远程 | |
| review note | 评审备注 | |
| review mark | 评审标记 | |
| hunk | 差异块 | |
| diff | 差异 | |
| snapshot | 快照 | |
| rewrite | 重写 | |
| head | 头部 | |
| stack / stacked PR | 堆叠 / 堆叠 PR | |
| agent | 代理 | 指标记变更的 AI 代理 |

## 保持英文的类别

以下内容不进入翻译 catalog，也不随界面语言变化：

- `OK`、`LFS`、`Enter`、`Git`、`Jujutsu`、字体名、编辑器/终端名、代码托管平台名。
- jj 命令、revset、配置键、路径、URL。
- jj 输出、核心层的内部错误、日志。
- 提交信息里的技术后缀（例如 `(submodule)`），避免仓库内容随界面语言变化。
- CLI 帮助、AI 提示词、Help Book、网站与发布文档。
