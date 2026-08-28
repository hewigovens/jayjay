import Foundation
import JayJayCore

enum RepoModalState: Identifiable {
    case createBookmark(rev: String)
    case stackedPr(rev: String)
    case confirmChange(RepoChangeConfirmation)
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
            case let .confirmChange(confirmation): confirmation.id
            case .submoduleAttention: "submodule-attention"
            case .undoLog: "undo-log"
            case .bookmarkManager: "bookmark-manager"
            case .workspaceCreate: "workspace-create"
            case let .confirmWorkspaceDelete(workspace): "workspace-delete-\(workspace.name)"
            case .sponsorPrompt: "sponsor-prompt"
        }
    }
}

enum RepoChangeConfirmation {
    case abandon(rev: String)
    case abandonSelection(revisions: [String])
    case squashSelection(revisions: [String])
    case rebase(request: DAGRebaseRequest)

    var id: String {
        switch self {
            case let .abandon(rev): "abandon-\(rev)"
            case let .abandonSelection(revisions): selectionId(prefix: "abandon", revisions: revisions)
            case let .squashSelection(revisions): selectionId(prefix: "squash", revisions: revisions)
            case let .rebase(request): "rebase-\(request.sourceCommitId)-\(request.destCommitId)"
        }
    }

    private func selectionId(prefix: String, revisions: [String]) -> String {
        let first = revisions.first ?? ""
        let last = revisions.last ?? ""
        return "\(prefix)-selection-\(revisions.count)-\(first)-\(last)"
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
