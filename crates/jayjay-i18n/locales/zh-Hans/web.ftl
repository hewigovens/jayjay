web-result = 差异结果
web-ignore-whitespace = 忽略空白
web-show-source = 显示源码
web-all-lines = 所有行
web-eof = 文件末尾没有换行符
web-empty = 粘贴一些文本，或打开两个文件进行比较。
web-binary-file = 不显示二进制文件。
web-mode-changed = 文件模式已从 { $old } 更改为 { $new }。
web-empty-file = 此文件为空。
web-no-text-changes = 没有可显示的文本变更。
web-show-rest = 显示此差异的其余部分
web-whitespace-only = 只有空白发生了变更。关闭“忽略空白”即可查看。
web-lines = 差异行
web-render-error = 无法渲染差异。
web-worker-error = 差异工作线程已停止。请重新加载页面以重启它。
web-loading = 正在加载…
web-engine-error = 引擎错误
web-privacy = 你的文件不会离开此设备。
web-files = 文件
web-resize-files = 调整文件列表大小
web-filter-files = 筛选文件
web-shortcuts = 键盘快捷键
web-shortcut-mark = 将文件标记为已评审，并转到下一个未评审的文件
web-shortcut-next = 下一个文件
web-shortcut-previous = 上一个文件
web-shortcut-next-unreviewed = 下一个未评审的文件
web-shortcut-previous-unreviewed = 上一个未评审的文件
web-hide-reviewed = 隐藏已评审
web-only-notes = 有备注
web-import-change = 导入变更
web-export-review = 导出评审状态
web-import-review = 导入评审状态
web-reviewed = 已评审
web-unreviewed = 未评审
web-partial = 部分评审
web-changed-since-review = 评审后有变更
web-mark-reviewed = 标记为已评审
web-group = 变更分组
web-notes = 评审备注
web-add-note = 添加备注
web-note-body = 撰写评审备注
web-save-note = 保存备注
web-cancel = 取消
web-resolve = 解决
web-current = 当前
web-stale = 过期
web-orphaned = 孤立
web-resolved = 已解决
web-loading-file = 正在加载文件和语法…
web-saved-locally = 已保存在此设备上
web-retry = 重试
web-local-data = 本地评审数据
web-load-avatars = 从 Gravatar、GitHub 或 GitLab 加载作者头像
web-import-replaces = 导入会替换此仓库的本地评审状态。在保存其他编辑之前，你都可以撤销导入。
web-syntax-unavailable = 语法高亮不可用。差异和评审操作仍可正常使用。
web-all-files = 所有文件
web-undo-import = 撤销导入
web-pull-heading = 评审 Pull Request，不丢失进度
web-pull-lede-lead = 评审标记在 force push 后依然保留。
web-pull-lede = 每个差异块都有一个稳定的评审标识，由其内容和附近上下文生成指纹，因此变基之后，只有发生变化的差异块会重新回到待评审状态。GitHub 的“已查看”则会重置整个文件。标记和备注都保存在你的浏览器中。
web-pull-url = Pull Request 链接
web-pull-open = 打开
web-pull-extension = 在 Diff Viewer 中打开
web-pull-extension-detail = 会在 Chrome 和 Edge 的 GitHub Pull Request 页面中添加一个 JayJay Diff 按钮。
web-pull-invalid = 这不是 GitHub 的 Pull Request、比较或提交链接。
web-pull-unsupported = JayJay Diff 无法打开此 GitHub 页面：
web-pull-supported = 请尝试类似 github.com/owner/repository/pull/123、github.com/owner/repository/compare/main...branch 或 github.com/owner/repository/commit/1a2b3c4 的链接。
web-pull-two-dot = JayJay Diff 无法打开双点 (..) 比较。三点 (...) 比较会显示自合并基点以来的变更：
web-pull-on-github = 在 GitHub 上打开
web-pull-not-found = { $kind ->
    [compare] 此比较不存在，或其仓库为私有仓库。
    [commit] 此提交不存在，或其仓库为私有仓库。
   *[pull] 此 Pull Request 不存在，或其仓库为私有仓库。
} JayJay Diff 只能打开公开仓库。
web-pull-rate-limited = GitHub 目前正在限制请求。请登录 GitHub 以使用你自己的额度，或几分钟后再试。
web-pull-too-large = { $kind ->
    [compare] 此比较涉及
    [commit] 此提交涉及
   *[pull] 此 Pull Request 涉及
} 超过 2,000 个文件，数量太多，无法在这里评审。
web-pull-unavailable = { $kind ->
    [compare] 无法从 GitHub 加载此比较。
    [commit] 无法从 GitHub 加载此提交。
   *[pull] 无法从 GitHub 加载此 Pull Request。
}
web-pull-sign-in = 使用 GitHub 登录
web-pull-device-code = 在 GitHub 上输入此验证码
web-pull-copy-open = 复制验证码并打开 GitHub
web-pull-waiting = 正在等待你批准…
web-pull-sign-in-failed = 登录未能完成。
web-pull-sign-out = 退出 GitHub 登录
web-pull-unauthorized = 你的 GitHub 登录已过期，请重新登录。
web-pull-refresh = 刷新差异
web-pull-share-notes = 与智能体分享备注
web-pull-share-copied = 链接已复制。把它交给你的智能体吧，7 天后失效。
web-pull-share-link = 分享链接
web-pull-share-copy-failed = 请选中链接并手动复制。
web-pull-share-failed = 无法创建链接。
web-pull-samples = 在上方输入 Pull Request 链接，或试试下面这些：
web-pull-how = 工作方式
web-pull-how-move = 在文件间移动
web-pull-how-move-detail = 按 N 和 P 会跳过你已评审的文件。
web-pull-how-mark = 标记后继续
web-pull-how-mark-detail = 只有发生变化的差异块会重新变为未评审。
web-pull-feature-hide = 隐藏已评审的文件
web-pull-feature-hide-detail = 列表中只保留尚未评审的内容。
web-pull-feature-tree = 文件树与筛选
web-pull-feature-tree-detail = 按文件夹浏览，或输入内容按路径筛选。
web-pull-feature-stacks = 提交与堆叠
web-pull-feature-stacks-detail = 一次一个提交，或整个堆叠。
web-pull-feature-sync = 与 GitHub 同步
web-pull-feature-sync-detail = 评审的同时，自动勾选 GitHub 的“已查看”复选框。
web-pull-source = GitHub 上的源码
web-pull-footer-lead = 隶属于
web-pull-footer = ，一个 Jujutsu 的原生客户端。
web-pull-sync-viewed = 将已评审的文件同步到 GitHub 的“已查看”复选框
web-pull-sync-viewed-scope = 勾选“已查看”复选框需要你公开仓库的写入权限。本站不会将它用于任何其他用途。
web-pull-syncing-viewed = 已评审的文件会同步到 GitHub 的“已查看”复选框
web-pull-sync-viewed-failed = 无法更新 GitHub 上的“已查看”复选框。
web-pull-sync-viewed-restricted = 该组织限制了第三方应用，因此无法更新“已查看”复选框。请在 GitHub 上为 JayJay Diff 申请访问权限。
web-pull-commits = 提交
web-pull-all-commits = 所有提交
web-pull-stack = 堆叠的 Pull Request
web-pull-merged = 已合并
web-pull-closed = 已关闭

web-font-section = 字体
web-font-diff = 差异
web-font-interface = 界面
web-increase-font-size = 增大字号
web-decrease-font-size = 减小字号
