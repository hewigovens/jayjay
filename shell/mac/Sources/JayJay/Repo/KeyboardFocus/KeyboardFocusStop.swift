/// Tab stops in declaration order; controls and inputs register while visible.
enum KeyboardFocusStop: CaseIterable {
    case dag, fileList, treeToggle, filterToggle, reviewMode
    case expandDescription, editDescription, emptyStateAction, markReviewed, diffLayout, editDiff, editFile
    case sidebarToggle, refresh, pull, push, revsetBack, revsetPresets, revsetFilter, revsetReset, editor, terminal, operationLog, operationLogList
    case commitSummary, commitDescription

    var isInSidebar: Bool {
        switch self {
            case .dag, .commitSummary, .commitDescription: true
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
