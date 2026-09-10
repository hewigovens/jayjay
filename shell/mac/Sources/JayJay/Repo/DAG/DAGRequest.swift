import JayJayCore

/// A DAG interaction the content view answers with presentation state — a sheet, a revset filter, a window — rather than a jj mutation.
enum DAGRequest {
    case rebase(DAGRebaseRequest)
    case abandon(rev: String)
    case abandonSelection(revisions: [String])
    case squashSelection(revisions: [String])
    case createBookmark(rev: String)
    case createStackedPRs(rev: String)
    case showAncestors(commitId: String)
    case openWorkspace(WorkspaceInfo)
}
