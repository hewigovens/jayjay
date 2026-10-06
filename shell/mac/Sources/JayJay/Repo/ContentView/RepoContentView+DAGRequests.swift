import JayJayCore

extension RepoContentView {
    func handleDAGRequest(_ request: DAGRequest) {
        switch request {
            case let .rebase(rebase):
                confirmRebase(rebase)
            case let .abandon(rev):
                confirmAbandon(rev)
            case let .abandonSelection(revisions):
                modal = .confirmChange(.abandonSelection(revisions: revisions))
            case let .squashSelection(revisions):
                modal = .confirmChange(.squashSelection(revisions: revisions))
            case let .createBookmark(rev):
                presentBookmarkCreate(rev: rev)
            case let .createTag(rev):
                presentTagCreate(rev: rev)
            case let .deleteTagOnRemote(name):
                modal = .confirmTagDeleteOnRemote(name: name)
            case let .createStackedPRs(rev):
                modal = .stackedPr(rev: rev)
            case let .showAncestors(changeId, commitId):
                viewModel.revealAncestors(of: changeId, selecting: commitId)
            case let .openWorkspace(workspace):
                windowManager.openRepo(workspace.path)
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
