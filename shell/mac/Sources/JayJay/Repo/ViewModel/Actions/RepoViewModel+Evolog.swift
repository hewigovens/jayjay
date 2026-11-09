import Foundation
import JayJayCore

extension RepoViewModel {
    func showEvolog(rev: String) {
        guard !rev.isEmpty else { return }
        // Select first so the detail pane focuses the change being inspected.
        if selectedChangeId != rev {
            select(changeId: rev)
        }
        evolog = EvologState(rev: rev)
        loadEvolog(rev: rev)
    }

    func restoreEvologVersion(_ commitId: String) {
        guard let rev = evolog?.rev else { return }
        performResult(
            selecting: rev,
            gatedBy: RepoActionGate(
                state: \.isRestoringEvologVersion,
                busyMessage: Localized.Repo.versionAlreadyBeingRestored
            ),
            onSuccess: { viewModel, _ in viewModel.loadEvolog(rev: rev) },
            { try $0.restoreVersion(rev: rev, version: commitId) }
        )
    }

    private func loadEvolog(rev: String) {
        runRepoTask { repo in
            try repo.evolog(rev: rev)
        } onSuccess: { vm, entries in
            guard vm.evolog?.rev == rev else { return } // user moved on while loading
            vm.evolog?.entries = entries
        } onFailure: { vm, error in
            guard vm.evolog?.rev == rev else { return } // don't clobber a newer request
            vm.evolog = nil
            vm.present(error: error)
        }
    }

    func dismissEvolog() {
        evolog = nil
    }
}
