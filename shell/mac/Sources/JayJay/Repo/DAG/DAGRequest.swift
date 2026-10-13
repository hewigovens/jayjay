import JayJayCore

/// A DAG interaction the content view answers with presentation state — a sheet, a revset filter, a window — rather than a jj mutation.
enum DAGRequest {
    case rebase(DAGRebaseRequest)
    case abandon(rev: String)
    case editDescription(rev: String, description: String)
    case abandonSelection(revisions: [String])
    case squashSelection(revisions: [String])
    case createBookmark(rev: String)
    case createTag(rev: String)
    case deleteTagOnRemote(name: String)
    case createStackedPRs(rev: String)
    case showAncestors(changeId: String, commitId: String)
    case openWorkspace(WorkspaceInfo)
}

extension DAGRequest {
    var presentedModal: RepoModalState? {
        switch self {
            case let .editDescription(rev, description): .editDescription(rev: rev, description: description)
            case let .abandonSelection(revisions): .confirmChange(.abandonSelection(revisions: revisions))
            case let .squashSelection(revisions): .confirmChange(.squashSelection(revisions: revisions))
            case let .deleteTagOnRemote(name): .confirmTagDeleteOnRemote(name: name)
            case let .createStackedPRs(rev): .stackedPr(rev: rev)
            case .rebase, .abandon, .createBookmark, .createTag, .showAncestors, .openWorkspace: nil
        }
    }
}
