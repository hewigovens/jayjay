/// Tab stops in declaration order; controls and inputs register while visible.
enum KeyboardFocusStop: CaseIterable {
    case dag, fileList, treeToggle, filterToggle
    case expandDescription, editDescription, markReviewed, diffLayout, editDiff, editFile
    case refresh, pull, push, revsetBack, revsetPresets, revsetFilter, revsetReset, editor, terminal
    case commitSummary, commitDescription

    var isInSidebar: Bool {
        switch self {
            case .dag, .commitSummary, .commitDescription: true
            default: false
        }
    }

    var isTextInput: Bool {
        switch self {
            case .commitSummary, .commitDescription: true
            default: false
        }
    }
}
