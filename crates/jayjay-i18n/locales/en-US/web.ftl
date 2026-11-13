web-result = Diff result
web-ignore-whitespace = Ignore whitespace
web-show-source = Show source
web-all-lines = All lines
web-eof = No newline at end of file
web-empty = Paste some text or open two files to compare.
web-binary-file = Binary file not shown.
web-mode-changed = File mode changed from { $old } to { $new }.
web-empty-file = This file is empty.
web-no-text-changes = No text changes to show.
web-show-rest = Show the rest of this diff
web-whitespace-only = Only whitespace changed. Turn off “Ignore whitespace” to see it.
web-lines = Diff lines
web-render-error = Couldn’t render the diff.
web-worker-error = The diff worker stopped. Reload the page to restart it.
web-loading = Loading…
web-engine-error = Engine error
web-privacy = Your files stay on this device.
web-files = Files
web-resize-files = Resize file list
web-filter-files = Filter files
web-shortcuts = Keyboard shortcuts
web-shortcut-mark = Mark the file reviewed and go to the next unreviewed one
web-shortcut-next = Next file
web-shortcut-previous = Previous file
web-shortcut-next-unreviewed = Next unreviewed file
web-shortcut-previous-unreviewed = Previous unreviewed file
web-hide-reviewed = Hide reviewed
web-only-notes = With notes
web-import-change = Import change
web-export-review = Export review state
web-import-review = Import review state
web-reviewed = Reviewed
web-unreviewed = Unreviewed
web-partial = Partly reviewed
web-changed-since-review = Changed since review
web-mark-reviewed = Mark reviewed
web-group = Change group
web-notes = Review notes
web-add-note = Add note
web-note-body = Write a review note
web-save-note = Save note
web-cancel = Cancel
web-resolve = Resolve
web-current = Current
web-stale = Stale
web-orphaned = Orphaned
web-resolved = Resolved
web-loading-file = Loading file and syntax…
web-saved-locally = Saved on this device
web-retry = Retry
web-local-data = Local review data
web-load-avatars = Load author avatars from Gravatar, GitHub, or GitLab
web-import-replaces = Import replaces this repository’s local review state. You can undo it until another edit is saved.
web-syntax-unavailable = Syntax highlighting is unavailable. The diff and review controls still work.
web-all-files = All files
web-undo-import = Undo import
web-pull-heading = Review pull requests without losing your place
web-pull-lede-lead = Review marks survive force-pushes.
web-pull-lede = Each hunk has a stable review identity, fingerprinted from its content and nearby context, so after a rebase only the hunks that changed come back for review. GitHub’s Viewed resets the whole file. Marks and notes stay in your browser.
web-pull-url = Pull request URL
web-pull-open = Open
web-pull-extension = Open in Diff Viewer
web-pull-extension-detail = extension for Chrome and Edge
web-pull-invalid = That isn’t a GitHub pull request, comparison, or commit URL.
web-pull-unsupported = JayJay Diff can’t open this GitHub page:
web-pull-supported = Try a URL like github.com/owner/repository/pull/123, github.com/owner/repository/compare/main...branch, or github.com/owner/repository/commit/1a2b3c4.
web-pull-two-dot = JayJay Diff can’t open two-dot (..) comparisons. The three-dot (...) comparison shows the changes since the merge base:
web-pull-patch-open = Open a diff or patch file
web-pull-patch-detail = , never uploaded
web-pull-patch-pasted = Pasted diff
web-pull-patch-invalid = That isn’t a git diff. Create one with git diff, jj diff --git, or gh pr diff.
web-pull-patch-series = That patch has several commits that change the same file. Open one diff of the whole change instead.
web-pull-patch-large = That diff is too large to review here.
web-pull-on-github = Open on GitHub
web-pull-not-found = { $kind ->
    [compare] This comparison doesn’t exist, or its repository is private.
    [commit] This commit doesn’t exist, or its repository is private.
   *[pull] This pull request doesn’t exist, or its repository is private.
} JayJay Diff opens public repositories only.
web-pull-rate-limited = GitHub is limiting requests right now. Sign in with GitHub to use your own limit, or try again in a few minutes.
web-pull-too-large = { $kind ->
    [compare] This comparison changes
    [commit] This commit changes
   *[pull] This pull request changes
} more than 2,000 files, too many to review here.
web-pull-unavailable = { $kind ->
    [compare] Couldn’t load this comparison from GitHub.
    [commit] Couldn’t load this commit from GitHub.
   *[pull] Couldn’t load this pull request from GitHub.
}
web-pull-sign-in = Sign in with GitHub
web-pull-device-code = Enter this code on GitHub
web-pull-copy-open = Copy code and open GitHub
web-pull-waiting = Waiting for you to approve it…
web-pull-sign-in-failed = Signing in didn’t finish.
web-pull-sign-out = Sign out of GitHub
web-pull-unauthorized = Your GitHub sign-in has expired. Sign in again.
web-pull-refresh = Refresh diff
web-pull-share-notes = Share notes with an agent
web-pull-share-copied = Link copied. Give it to your agent; it expires in 7 days.
web-pull-share-link = Share link
web-pull-share-copy-failed = Select the link to copy it manually.
web-pull-share-failed = Couldn’t create the link.
web-pull-samples = Enter a pull request URL above, or try one of these:
web-pull-how = How it works
web-pull-how-move = Move through files
web-pull-how-move-detail = N and P skip the files you’ve reviewed.
web-pull-how-mark = Mark and move on
web-pull-how-mark-detail = Only hunks that change come back unreviewed.
web-pull-feature-hide = Hide reviewed files
web-pull-feature-hide-detail = Only what’s left to review stays in the list.
web-pull-feature-tree = File tree and filter
web-pull-feature-tree-detail = Browse by folder, or type to filter by path.
web-pull-feature-stacks = Commits and stacks
web-pull-feature-stacks-detail = One commit at a time, or a whole stack.
web-pull-feature-sync = Synced with GitHub
web-pull-feature-sync-detail = Tick GitHub’s Viewed boxes as you go.
web-pull-source = Source on GitHub
web-pull-footer-lead = Part of
web-pull-footer = , a native client for Jujutsu.
web-pull-sync-viewed = Sync reviewed files to GitHub’s Viewed checkboxes
web-pull-sync-viewed-scope = Ticking Viewed boxes needs write access to your public repositories. This site uses it for nothing else.
web-pull-syncing-viewed = Reviewed files sync to GitHub’s Viewed checkboxes
web-pull-sync-viewed-failed = Couldn’t update Viewed checkboxes on GitHub.
web-pull-sync-viewed-restricted = This organization restricts third-party apps, so Viewed checkboxes can’t be updated. Request access for JayJay Diff on GitHub.
web-pull-commits = Commits
web-pull-all-commits = All commits
web-pull-stack = Stacked pull requests
web-pull-merged = Merged
web-pull-closed = Closed

web-font-section = Font
web-font-diff = Diff
web-font-interface = Interface
web-increase-font-size = Increase font size
web-decrease-font-size = Decrease font size
