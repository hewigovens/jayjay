/// Tab stops in declaration order; controls and inputs register while visible.
enum KeyboardFocusStop: CaseIterable {
    case dag, fileList, treeToggle, filterToggle, reviewMode, showAllFiles
    case expandDescription, editDescription, emptyStateAction, diffLayout, editDiff, editFile, markReviewed
    case sidebarToggle, refresh, pull, push, revsetBack, revsetPresets, revsetFilter, revsetReset, editor, terminal, operationLog, operationLogList
    case commitSummary, commitDescription

    var isInSidebar: Bool {
        switch self {
            case .dag, .commitSummary, .commitDescription, .refresh, .pull, .push: true
            default: false
        }
    }

    /// Stops that take first responder and keep their own keys.
    var ownsKeys: Bool {
        switch self {
            case .commitSummary, .commitDescription, .operationLogList: true
            default: false
        }
    }
}
