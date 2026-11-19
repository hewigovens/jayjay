web-result = 差異結果
web-ignore-whitespace = 忽略空白
web-show-source = 顯示原始碼
web-all-lines = 所有行
web-eof = 檔案結尾沒有換行字元
web-empty = 貼上一些文字，或開啟兩個檔案進行比較。
web-binary-file = 不顯示二進位檔案。
web-mode-changed = 檔案模式已從 { $old } 變更為 { $new }。
web-empty-file = 此檔案是空的。
web-no-text-changes = 沒有可顯示的文字變更。
web-show-rest = 顯示此差異的其餘部分
web-whitespace-only = 只有空白發生了變更。關閉「忽略空白」即可查看。
web-lines = 差異行
web-render-error = 無法顯示差異。
web-worker-error = 差異工作執行緒已停止。請重新載入頁面以重新啟動。
web-loading = 正在載入…
web-engine-error = 引擎錯誤
web-privacy = 你的檔案不會離開此裝置。
web-files = 檔案
web-resize-files = 調整檔案列表大小
web-filter-files = 篩選檔案
web-shortcuts = 鍵盤快速鍵
web-shortcut-mark = 將檔案標記為已審查，並前往下一個未審查的檔案
web-shortcut-next = 下一個檔案
web-shortcut-previous = 上一個檔案
web-shortcut-next-unreviewed = 下一個未審查的檔案
web-shortcut-previous-unreviewed = 上一個未審查的檔案
web-hide-reviewed = 隱藏已審查
web-only-notes = 有備註
web-import-change = 匯入變更
web-export-review = 匯出審查狀態
web-import-review = 匯入審查狀態
web-reviewed = 已審查
web-unreviewed = 未審查
web-partial = 部分審查
web-changed-since-review = 審查後有變更
web-mark-reviewed = 標記為已審查
web-group = 變更群組
web-notes = 審查備註
web-add-note = 新增備註
web-note-body = 撰寫審查備註
web-save-note = 儲存備註
web-cancel = 取消
web-resolve = 解決
web-current = 目前
web-stale = 過期
web-orphaned = 孤立
web-resolved = 已解決
web-loading-file = 正在載入檔案和語法…
web-saved-locally = 已儲存在此裝置上
web-retry = 重試
web-local-data = 本機審查資料
web-load-avatars = 從 Gravatar、GitHub 或 GitLab 載入作者頭像
web-import-replaces = 匯入會取代此儲存庫的本機審查狀態。在儲存其他編輯之前，你都可以還原匯入。
web-syntax-unavailable = 語法醒目提示無法使用。差異和審查操作仍可正常使用。
web-all-files = 所有檔案
web-undo-import = 還原匯入
web-pull-heading = 審查 Pull Request，不遺失進度
web-pull-lede-lead = 審查標記在 force push 後依然保留。
web-pull-lede = 每個差異區塊都有一個穩定的審查識別碼，由其內容和附近上下文產生指紋，因此重定基底之後，只有發生變化的差異區塊會重新回到待審查狀態。GitHub 的「已檢視」則會重設整個檔案。標記和備註都儲存在你的瀏覽器中。
web-pull-url = Pull Request 網址
web-pull-open = 開啟
web-pull-extension = 在 Diff Viewer 中開啟
web-pull-extension-detail = （Chrome 和 Edge 擴充功能）
web-pull-invalid = 這不是 GitHub 的 Pull Request、比較或提交網址。
web-pull-unsupported = JayJay Diff 無法開啟此 GitHub 頁面：
web-pull-supported = 請嘗試類似 github.com/owner/repository/pull/123、github.com/owner/repository/compare/main...branch 或 github.com/owner/repository/commit/1a2b3c4 的網址。
web-pull-two-dot = JayJay Diff 無法開啟雙點 (..) 比較。三點 (...) 比較會顯示自合併基底以來的變更：
web-pull-patch-open = 開啟 diff 或 patch 檔案
web-pull-patch-detail = （不會上傳）
web-pull-patch-pasted = 貼上的 diff
web-pull-patch-invalid = 這不是 git diff。請用 git diff、jj diff --git 或 gh pr diff 產生。
web-pull-patch-series = 這個修補檔裡有多個提交修改了同一個檔案。請改為開啟包含全部變更的單一 diff。
web-pull-patch-large = 這個 diff 太大，無法在這裡審查。
web-pull-on-github = 在 GitHub 上開啟
web-pull-not-found = { $kind ->
    [compare] 此比較不存在，或其儲存庫為私有儲存庫。
    [commit] 此提交不存在，或其儲存庫為私有儲存庫。
   *[pull] 此 Pull Request 不存在，或其儲存庫為私有儲存庫。
} JayJay Diff 只能開啟公開儲存庫。
web-pull-rate-limited = GitHub 目前正在限制請求。請登入 GitHub 以使用你自己的額度，或幾分鐘後再試。
web-pull-too-large = { $kind ->
    [compare] 此比較涉及
    [commit] 此提交涉及
   *[pull] 此 Pull Request 涉及
} 超過 2,000 個檔案，數量太多，無法在這裡審查。
web-pull-unavailable = { $kind ->
    [compare] 無法從 GitHub 載入此比較。
    [commit] 無法從 GitHub 載入此提交。
   *[pull] 無法從 GitHub 載入此 Pull Request。
}
web-pull-sign-in = 使用 GitHub 登入
web-pull-device-code = 在 GitHub 上輸入此驗證碼
web-pull-copy-open = 拷貝驗證碼並開啟 GitHub
web-pull-waiting = 正在等待你核准…
web-pull-sign-in-failed = 登入未能完成。
web-pull-sign-out = 登出 GitHub
web-pull-unauthorized = 你的 GitHub 登入已過期，請重新登入。
web-pull-refresh = 重新整理差異
web-pull-share-notes = 與代理程式分享備註
web-pull-share-copied = 連結已拷貝。把它交給你的代理程式吧，7 天後失效。
web-pull-share-link = 分享連結
web-pull-share-copy-failed = 請選取連結並手動拷貝。
web-pull-share-failed = 無法建立連結。
web-pull-samples = 在上方輸入 Pull Request 網址，或試試下面這些：
web-pull-how = 運作方式
web-pull-how-move = 在檔案間移動
web-pull-how-move-detail = 按 N 和 P 會跳過你已審查的檔案。
web-pull-how-mark = 標記後繼續
web-pull-how-mark-detail = 只有發生變化的差異區塊會重新變為未審查。
web-pull-feature-hide = 隱藏已審查的檔案
web-pull-feature-hide-detail = 列表中只保留尚未審查的內容。
web-pull-feature-tree = 檔案樹與篩選
web-pull-feature-tree-detail = 依資料夾瀏覽，或輸入內容依路徑篩選。
web-pull-feature-stacks = 提交與堆疊
web-pull-feature-stacks-detail = 一次一個提交，或整個堆疊。
web-pull-feature-sync = 與 GitHub 同步
web-pull-feature-sync-detail = 審查的同時，自動勾選 GitHub 的「已檢視」核取方塊。
web-pull-source = GitHub 上的原始碼
web-pull-footer-lead = 隸屬於
web-pull-footer = ，一個 Jujutsu 的原生用戶端。
web-pull-sync-viewed = 將已審查的檔案同步到 GitHub 的「已檢視」核取方塊
web-pull-sync-viewed-scope = 勾選「已檢視」核取方塊需要你公開儲存庫的寫入權限。本站不會將它用於任何其他用途。
web-pull-syncing-viewed = 已審查的檔案會同步到 GitHub 的「已檢視」核取方塊
web-pull-sync-viewed-failed = 無法更新 GitHub 上的「已檢視」核取方塊。
web-pull-sync-viewed-restricted = 此組織限制了第三方應用程式，因此無法更新「已檢視」核取方塊。請在 GitHub 上為 JayJay Diff 申請存取權限。
web-pull-commits = 提交
web-pull-all-commits = 所有提交
web-pull-stack = 堆疊的 Pull Request
web-pull-merged = 已合併
web-pull-closed = 已關閉

web-font-section = 字型
web-font-diff = 差異
web-font-interface = 介面
web-increase-font-size = 增大字型大小
web-decrease-font-size = 縮小字型大小
