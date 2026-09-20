/// Tab stops in declaration order; controls and inputs register while visible.
enum KeyboardFocusStop: CaseIterable {
    case dag, fileList, treeToggle, filterToggle
    case expandDescription, diffLayout, editDescription, editDiff
    case sidebarToggle, revsetFilter, revsetInput, refresh, pull, push, editor, terminal, settings
    case commitSummary, commitDescription

    var isInSidebar: Bool {
        switch self {
            case .dag, .revsetInput, .commitSummary, .commitDescription: true
            default: false
        }
    }

    var isTextInput: Bool {
        switch self {
            case .revsetInput, .commitSummary, .commitDescription: true
            default: false
        }
    }
}
