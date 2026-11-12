web-result = 差分の結果
web-ignore-whitespace = 空白を無視
web-show-source = ソースを表示
web-all-lines = すべての行
web-eof = ファイル末尾に改行がありません
web-empty = テキストを貼り付けるか、2 つのファイルを開いて比較してください。
web-binary-file = バイナリ ファイルは表示されません。
web-mode-changed = ファイル モードが { $old } から { $new } に変更されました。
web-empty-file = このファイルは空です。
web-no-text-changes = 表示できるテキストの変更はありません。
web-show-rest = この差分の残りを表示
web-whitespace-only = 空白のみが変更されています。表示するには「空白を無視」をオフにしてください。
web-lines = 差分の行
web-render-error = 差分を表示できませんでした。
web-worker-error = 差分ワーカーが停止しました。ページを再読み込みして再起動してください。
web-loading = 読み込み中…
web-engine-error = エンジン エラー
web-privacy = ファイルはこのデバイスの外に出ません。
web-files = ファイル
web-resize-files = ファイル一覧のサイズを変更
web-filter-files = ファイルを絞り込む
web-shortcuts = キーボード ショートカット
web-shortcut-mark = ファイルをレビュー済みにして、次の未レビューのファイルへ移動
web-shortcut-next = 次のファイル
web-shortcut-previous = 前のファイル
web-shortcut-next-unreviewed = 次の未レビューのファイル
web-shortcut-previous-unreviewed = 前の未レビューのファイル
web-hide-reviewed = レビュー済みを隠す
web-only-notes = メモあり
web-import-change = チェンジを読み込む
web-export-review = レビュー状態を書き出す
web-import-review = レビュー状態を読み込む
web-reviewed = レビュー済み
web-unreviewed = 未レビュー
web-partial = 一部レビュー済み
web-changed-since-review = レビュー後に変更あり
web-mark-reviewed = レビュー済みにする
web-group = チェンジのグループ
web-notes = レビュー メモ
web-add-note = メモを追加
web-note-body = レビュー メモを書く
web-save-note = メモを保存
web-cancel = キャンセル
web-resolve = 解決
web-current = 最新
web-stale = 古い
web-orphaned = 孤立
web-resolved = 解決済み
web-loading-file = ファイルとシンタックスを読み込み中…
web-saved-locally = このデバイスに保存済み
web-retry = 再試行
web-local-data = ローカルのレビュー データ
web-load-avatars = 作成者のアバターを Gravatar、GitHub、GitLab から読み込む
web-import-replaces = 読み込むと、このリポジトリのローカルのレビュー状態が置き換えられます。別の編集を保存するまでは、取り消すことができます。
web-syntax-unavailable = シンタックス ハイライトは利用できません。差分とレビューの操作は引き続き使えます。
web-all-files = すべてのファイル
web-undo-import = 読み込みを取り消す
web-pull-heading = プル リクエストを、見ていた場所を見失わずにレビュー
web-pull-lede-lead = レビューのマークは force push 後も残ります。
web-pull-lede = 各ハンクには、内容と周辺のコンテキストから算出したフィンガープリントによる安定したレビュー ID があるので、リベース後は変更されたハンクだけが再びレビュー対象になります。GitHub の「表示済み」はファイル全体がリセットされます。マークとメモはブラウザー内に保存されます。
web-pull-url = プル リクエストの URL
web-pull-open = 開く
web-pull-extension = Diff Viewer で開く
web-pull-extension-detail = は、Chrome と Edge の GitHub プル リクエストに JayJay Diff ボタンを追加します。
web-pull-invalid = GitHub のプル リクエスト、比較、コミットの URL ではありません。
web-pull-unsupported = JayJay Diff ではこの GitHub ページを開けません:
web-pull-supported = github.com/owner/repository/pull/123、github.com/owner/repository/compare/main...branch、github.com/owner/repository/commit/1a2b3c4 のような URL をお試しください。
web-pull-two-dot = JayJay Diff では 2 ドット (..) の比較は開けません。3 ドット (...) の比較なら、マージ ベース以降の変更を表示できます:
web-pull-on-github = GitHub で開く
web-pull-not-found = { $kind ->
    [compare] この比較は存在しないか、リポジトリが非公開です。
    [commit] このコミットは存在しないか、リポジトリが非公開です。
   *[pull] このプル リクエストは存在しないか、リポジトリが非公開です。
} JayJay Diff が開けるのは公開リポジトリのみです。
web-pull-rate-limited = GitHub がリクエストを制限しています。GitHub にサインインして自分の上限を使うか、数分後にもう一度お試しください。
web-pull-too-large = { $kind ->
    [compare] この比較の変更は
    [commit] このコミットの変更は
   *[pull] このプル リクエストの変更は
} 2,000 ファイルを超えており、ここでレビューするには多すぎます。
web-pull-unavailable = { $kind ->
    [compare] GitHub からこの比較を読み込めませんでした。
    [commit] GitHub からこのコミットを読み込めませんでした。
   *[pull] GitHub からこのプル リクエストを読み込めませんでした。
}
web-pull-sign-in = GitHub でサインイン
web-pull-device-code = GitHub でこのコードを入力してください
web-pull-copy-open = コードをコピーして GitHub を開く
web-pull-waiting = 承認を待っています…
web-pull-sign-in-failed = サインインが完了しませんでした。
web-pull-sign-out = GitHub からサインアウト
web-pull-unauthorized = GitHub のサインインの有効期限が切れました。もう一度サインインしてください。
web-pull-refresh = 差分を更新
web-pull-share-notes = メモをエージェントと共有
web-pull-share-copied = リンクをコピーしました。エージェントに渡してください。有効期限は 7 日間です。
web-pull-share-link = 共有リンク
web-pull-share-copy-failed = リンクを選択して、手動でコピーしてください。
web-pull-share-failed = リンクを作成できませんでした。
web-pull-samples = 上にプル リクエストの URL を入力するか、次の例を試してください:
web-pull-how = しくみ
web-pull-how-move = ファイル間を移動
web-pull-how-move-detail = N と P で、レビュー済みのファイルを飛ばして移動できます。
web-pull-how-mark = マークして次へ
web-pull-how-mark-detail = 変更されたハンクだけが、未レビューに戻ります。
web-pull-feature-hide = レビュー済みのファイルを隠す
web-pull-feature-hide-detail = レビューが残っているものだけが一覧に表示されます。
web-pull-feature-tree = ファイル ツリーと絞り込み
web-pull-feature-tree-detail = フォルダーごとに参照するか、入力してパスで絞り込めます。
web-pull-feature-stacks = コミットとスタック
web-pull-feature-stacks-detail = コミットを 1 つずつ、またはスタック全体を。
web-pull-feature-sync = GitHub と同期
web-pull-feature-sync-detail = レビューを進めながら、GitHub の「表示済み」ボックスにチェックを入れます。
web-pull-source = GitHub のソース
web-pull-footer-lead = 所属:
web-pull-footer = 、Jujutsu 用のネイティブ クライアントです。
web-pull-sync-viewed = レビュー済みのファイルを GitHub の「表示済み」チェックボックスに同期
web-pull-sync-viewed-scope = 「表示済み」ボックスにチェックを入れるには、公開リポジトリへの書き込み権限が必要です。このサイトでは、それ以外の目的には使用しません。
web-pull-syncing-viewed = レビュー済みのファイルは GitHub の「表示済み」チェックボックスに同期されます
web-pull-sync-viewed-failed = GitHub の「表示済み」チェックボックスを更新できませんでした。
web-pull-sync-viewed-restricted = この Organization はサードパーティ製アプリを制限しているため、「表示済み」チェックボックスを更新できません。GitHub で JayJay Diff のアクセスをリクエストしてください。
web-pull-commits = コミット
web-pull-all-commits = すべてのコミット
web-pull-stack = スタックされたプル リクエスト
web-pull-merged = マージ済み
web-pull-closed = クローズ済み

web-font-section = フォント
web-font-diff = 差分
web-font-interface = インターフェイス
web-increase-font-size = フォント サイズを大きくする
web-decrease-font-size = フォント サイズを小さくする
