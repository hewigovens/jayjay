/// Tab stops in declaration order; controls and inputs register while visible.
enum KeyboardFocusStop: CaseIterable {
    case dag, fileList, treeToggle, filterToggle
    case expandDescription, editDescription, diffLayout, editDiff, editFile
    case sidebarToggle, refresh, pull, push, revsetBack, revsetPresets, revsetFilter, revsetReset, editor, terminal, settings
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
