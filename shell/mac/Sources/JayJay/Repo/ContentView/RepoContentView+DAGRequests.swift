import JayJayCore

extension RepoContentView {
    func handleDAGRequest(_ request: DAGRequest) {
        if let presented = request.presentedModal {
            modal = presented
            return
        }
        switch request {
            case let .rebase(rebase):
                confirmRebase(rebase)
            case let .abandon(rev):
                confirmAbandon(rev)
            case let .createBookmark(rev):
                presentBookmarkCreate(rev: rev)
            case let .createTag(rev):
                presentTagCreate(rev: rev)
            case let .showAncestors(changeId, commitId):
                viewModel.revealAncestors(of: changeId, selecting: commitId)
            case let .openWorkspace(workspace):
                windowManager.openRepo(workspace.path)
            case let .bisectBetween(first, second):
                viewModel.startBisectBetween(first: first, second: second)
            case .editDescription, .abandonSelection, .squashSelection, .deleteTagOnRemote, .createStackedPRs, .bisectFrom:
                break
        }
    }

    private func confirmRebase(_ request: DAGRebaseRequest) {
        if settings.confirmDragRebase {
            modal = .confirmChange(.rebase(request: request))
        } else {
            runDAGRebase(request)
        }
    }

    private func confirmAbandon(_ rev: String) {
        if settings.skipAbandonConfirmation {
            viewModel.abandon(rev: rev)
        } else {
            modal = .confirmChange(.abandon(rev: rev))
        }
    }
}
