import Foundation
import JayJayCore

enum RepoModalState: Identifiable {
    case createBookmark(rev: String)
    case stackedPr(rev: String)
    case confirmAbandon(rev: String)
    case confirmRebase(request: DAGRebaseRequest)
    case submoduleAttention
    case undoLog
    case bookmarkManager
    case workspaceCreate
    case confirmWorkspaceDelete(workspace: WorkspaceInfo)
    case sponsorPrompt

    var id: String {
        switch self {
            case let .createBookmark(rev): "bookmark-\(rev)"
            case let .stackedPr(rev): "stacked-pr-\(rev)"
            case let .confirmAbandon(rev): "abandon-\(rev)"
            case let .confirmRebase(request):
                "rebase-\(request.sourceCommitId)-\(request.destCommitId)"
            case .submoduleAttention: "submodule-attention"
            case .undoLog: "undo-log"
            case .bookmarkManager: "bookmark-manager"
            case .workspaceCreate: "workspace-create"
            case let .confirmWorkspaceDelete(workspace): "workspace-delete-\(workspace.name)"
            case .sponsorPrompt: "sponsor-prompt"
        }
    }
}

enum RepoAlertState: Identifiable {
    case error(String)
    case configWarning(String)

    var id: String {
        switch self {
            case let .error(message): "error-\(message)"
            case let .configWarning(message): "config-warning-\(message)"
        }
    }
}

enum RepoOverlayState: Identifiable {
    case loading
    case toast(RepoToastState)

    var id: String {
        switch self {
            case .loading:
                "loading"
            case let .toast(state):
                "toast-\(state.id)"
        }
    }
}
