import Foundation
import JayJayCore

extension RepoViewModel {
    func showEvolog(rev: String) {
        guard !rev.isEmpty else { return }
        // Select first so the detail pane focuses the change being inspected.
        if selectedChangeId != rev {
            select(changeId: rev)
        }
        evologRev = rev
        evologEntries = nil
        loadEvolog(rev: rev)
    }

    func restoreEvologVersion(_ commitId: String) {
        guard let rev = evologRev else { return }
        performResult(
            selecting: rev,
            gatedBy: RepoActionGate(
                state: \.isRestoringEvologVersion,
                busyMessage: "A version is already being restored"
            ),
            onSuccess: { viewModel, _ in viewModel.loadEvolog(rev: rev) },
            { try $0.restoreVersion(rev: rev, version: commitId) }
        )
    }

    private func loadEvolog(rev: String) {
        runRepoTask { repo in
            try repo.evolog(rev: rev)
        } onSuccess: { vm, entries in
            guard vm.evologRev == rev else { return } // user moved on while loading
            vm.evologEntries = entries
        } onFailure: { vm, error in
            guard vm.evologRev == rev else { return } // don't clobber a newer request
            vm.evologRev = nil
            vm.present(error: error)
        }
    }

    func dismissEvolog() {
        evologRev = nil
        evologEntries = nil
    }
}
