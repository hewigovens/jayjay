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
