import AppKit

extension RepoWindowManager {
    func showCloneRepository() {
        // The Window scene keeps its view state after closing, so a window opened from closed gets a fresh form.
        if !liveWindows.contains(where: { $0.identifier?.rawValue.hasPrefix(AppWindows.cloneRepository) == true }) {
            cloneRepositoryGeneration += 1
        }
        presentWindow(AppWindows.cloneRepository)
    }
}
