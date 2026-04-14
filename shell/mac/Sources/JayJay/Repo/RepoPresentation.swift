import Foundation

enum RepoModalState: Identifiable {
    case createBookmark(rev: String)
    case confirmAbandon(rev: String)
    case confirmRebase(request: DAGRebaseRequest)
    case submoduleAttention
    case undoLog
    case bookmarkManager
    case workspaceCreate
    case sponsorPrompt

    var id: String {
        switch self {
            case .createBookmark(let rev): "bookmark-\(rev)"
            case .confirmAbandon(let rev): "abandon-\(rev)"
            case .confirmRebase(let request):
                "rebase-\(request.sourceCommitId)-\(request.destCommitId)"
            case .submoduleAttention: "submodule-attention"
            case .undoLog: "undo-log"
            case .bookmarkManager: "bookmark-manager"
            case .workspaceCreate: "workspace-create"
            case .sponsorPrompt: "sponsor-prompt"
        }
    }
}

enum RepoAlertState: Identifiable {
    case error(String)
    case configWarning(String)

    var id: String {
        switch self {
            case .error(let message): "error-\(message)"
            case .configWarning(let message): "config-warning-\(message)"
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
            case .toast(let state):
                "toast-\(state.id)"
        }
    }
}
