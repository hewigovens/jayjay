overview-loading-overview-ellipsis = Loading overview…
overview-no-mutable-changes-every-change-is = No mutable changes. Every change is on trunk or immutable.
overview-filter-lanes = Filter lanes
overview-workspaces = WORKSPACES
overview-changes = CHANGES
overview-bookmarks = BOOKMARKS
overview-abandon-this-change-its-descendants-are = Abandon this change? Its descendants are rebased onto its parent, and a workspace checked out here moves to a new empty change.
overview-abandon-lane-ellipsis = Abandon Lane…
overview-copy-head-change-id = Copy Head Change ID
overview-abandon-change-ellipsis = Abandon Change…
overview-no-file-changes = No file changes
overview-this-change-has-conflicts = This change has conflicts
overview-changes-sentence = Changes
overview-filter-lanes-title = Filter Lanes
overview-window-title = Repo Overview — { $name }
overview-abandon-title = Abandon “{ $title }”?
overview-abandon-lane-message = Abandon all { NUMBER($count) } changes of this lane? A workspace checked out in it moves to a new empty change on the base.
overview-rebase-onto = Rebase onto { $trunk }
overview-rebase-lane-onto = Rebase Lane onto { $trunk }
overview-open-in-window = Open { $name } in Window
overview-show-in = Show { $name } in { $app }
overview-forget-delete-from-disk = Forget & Delete { $name } from Disk…
overview-filter-lanes-shortcut = Filter lanes ({ $shortcut })
overview-on-trunk = On { $trunk }
overview-commits-behind =
    { $count ->
        [one] { $count } commit behind { $trunk }
       *[other] { $count } commits behind { $trunk }
    }
overview-behind-short = { NUMBER($count) } behind { $trunk }
overview-forked-from = Forked from { $base }
overview-on-off-trunk = On { $base }, off { $trunk }
overview-at-the-head = at the head
overview-below-the-head = { NUMBER($count) } below the head
overview-empty-checkout-above =
    { $count ->
        [one] Empty, undescribed checkout above { $count } change
       *[other] Empty, undescribed checkout above { $count } changes
    }
overview-changes-with-conflicts =
    { $count ->
        [one] { $count } change with conflicts
       *[other] { $count } changes with conflicts
    }
overview-changes-without-description =
    { $count ->
        [one] { $count } change without a description
       *[other] { $count } changes without a description
    }
overview-lane-summary = { $changes } · { $age }
overview-stat-lanes =
    { $count ->
        [one] lane
       *[other] lanes
    }
overview-stat-changes =
    { $count ->
        [one] change
       *[other] changes
    }
overview-stat-workspaces =
    { $count ->
        [one] workspace
       *[other] workspaces
    }
overview-stat-behind-trunk = behind trunk
overview-stat-attention =
    { $count ->
        [one] needs attention
       *[other] need attention
    }
overview-fork-point = fork point
overview-updated = updated { $age }
