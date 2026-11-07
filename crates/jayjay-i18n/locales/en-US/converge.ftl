converge-title = Resolve Divergent Change
converge-menu = Resolve Divergent Change…
converge-apply = Converge
converge-applying = Converging…
converge-apply-and-resolve = Converge & Resolve
converge-detail-versions = { $count ->
    [one] { $count } version
   *[other] { $count } versions
}
converge-resolve = Resolve…
converge-loading = Loading preview…
converge-refresh = Refresh Preview
converge-unavailable = Reconnect to the repository, then refresh the preview.
converge-not-divergent = This change is no longer divergent; there is nothing to resolve.
converge-stale = The repository changed. Nothing was applied. Review the refreshed preview and choose again.
converge-versions = Versions
converge-versions-hint = Keep one version as-is, or decide item by item below.
converge-outside-filter = Includes versions outside the current graph filter.
converge-keep = Keep { $version }
converge-use-version = Use { $version }
converge-edit = Edit…
converge-choices = Choices
converge-decided-automatically = { $decided } of { $total } decided automatically
converge-description = Description
converge-contents = Contents
converge-author = Author
converge-parents = Parents
converge-unresolved = Unresolved
converge-merged = Merged automatically
converge-using = Using { $version }
converge-edited = Edited
converge-result-keeps = Result keeps { $version }
converge-versions-disagree = The versions disagree
converge-same = { $count ->
    [2] Same in both
   *[other] Same in all versions
}
converge-follows-parents = Follows the chosen parents
converge-immutable = Immutable
converge-no-description = No description
converge-unresolved-count = { $count ->
    [one] { $count } choice unresolved
   *[other] { $count } choices unresolved
}
converge-ready = Ready
converge-choose-author-hint = Choose an author to continue.
converge-choose-description-hint = Choose a description to continue.
converge-choose-parents-hint = Choose parents to continue.
converge-changed-files = { $count ->
    [one] { $count } file differs
   *[other] { $count } files differ
}
converge-compare-with = Compare with
converge-will-conflict = { $count ->
    [one] { $count } file will conflict
   *[other] { $count } files will conflict
}
converge-conflicts-followup = Converging keeps the conflict; resolve it afterwards.
converge-blocked = Cannot converge versions
converge-immutable-reason = A version or descendant is immutable and cannot be rewritten.
converge-details = Details
converge-rebase-descendants = { $count ->
    [one] Rebases { $count } descendant
   *[other] Rebases { $count } descendants
}
converge-move-bookmarks = { $count ->
    [one] Moves { $count } bookmark
   *[other] Moves { $count } bookmarks
}
converge-move-workspaces = { $count ->
    [one] Updates { $count } workspace
   *[other] Updates { $count } workspaces
}
converge-no-effects = Nothing else moves: no descendants, bookmarks or workspaces.
converge-undo-hint = Undo from the operation log.
converge-version-summary = { $count ->
    [one] { $count } version → 1 change
   *[other] { $count } versions → 1 change
}
