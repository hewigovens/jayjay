import JayJayCore

extension RepoContentView {
    func handleDAGRequest(_ request: DAGRequest) {
        if let presented = request.presentedModal {
            modal = presented
            return
        }
        switch request {
            case .rebase, .rebaseCommand, .rebaseOntoTrunk, .rebaseSelectionOntoTrunk:
                handleRebaseRequest(request)
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
            case .converge, .editDescription, .abandonSelection, .squashSelection, .deleteTagOnRemote, .createStackedPRs, .bisectFrom:
                break
        }
    }

    private func handleRebaseRequest(_ request: DAGRequest) {
        switch request {
            case let .rebase(drop):
                if settings.confirmDragRebase {
                    modal = .confirmChange(.rebase(request: drop))
                } else {
                    runDAGRebase(drop)
                }
            case let .rebaseCommand(command):
                runRebase(command)
            case let .rebaseOntoTrunk(change):
                runRebase(.ontoTrunk(change, trunkName: viewModel.trunkName))
            case let .rebaseSelectionOntoTrunk(revisions):
                runRebase(.selection(revisions, onto: "trunk()", destLabel: viewModel.trunkName))
            default:
                break
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
