/// Tab stops in declaration order; controls and inputs register while visible.
enum KeyboardFocusStop: CaseIterable {
    case dag, fileList, treeToggle, filterToggle
    case editDescription, markReviewed, editDiff, editFile
    case refresh, pull, push, revsetBack, revsetPresets, revsetFilter, revsetReset, editor, terminal
    case commitSummary, commitDescription

    var isInSidebar: Bool {
        switch self {
            case .dag: true
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
