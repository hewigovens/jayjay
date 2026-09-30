import Foundation
import JayJayCore

extension RepoWindowManager {
    func switchRepo(from viewModel: RepoViewModel, to path: String, changePath: (String) -> Void) {
        let target = normalizedRepositoryPath(path: URL(fileURLWithPath: path).standardizedFileURL.path)
        guard !isRemovingRepo(at: target), target != normalizedRepositoryPath(path: viewModel.repoPath) else { return }
        settings.recordOpenedRepo(target)
        if activateRepo(target) {
            return
        }
        // Keep the source model alive until the destination opens, so a failed switch can return to it.
        viewModel.switchTarget = target
        changePath(target)
    }

    /// Ends a switch that did not land; false, after retiring the source, when it was removed meanwhile or another window now shows it.
    func endSwitch(returningTo source: RepoViewModel) -> Bool {
        source.switchTarget = nil
        guard !source.isShuttingDown, !source.workspaceVanished, repoWindow(at: source.repoPath) == nil else {
            retire(source)
            return false
        }
        return true
    }
}
