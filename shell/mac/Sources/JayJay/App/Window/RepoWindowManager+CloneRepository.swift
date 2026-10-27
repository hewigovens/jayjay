import AppKit
import JayJayCore

extension RepoWindowManager {
    func showCloneRepository() {
        // The Window scene keeps its view state after closing, so a window opened from closed gets a fresh form.
        if !liveWindows.contains(where: { $0.identifier?.rawValue.hasPrefix(AppWindows.cloneRepository) == true }) {
            cloneRepositoryGeneration += 1
        }
        presentWindow(AppWindows.cloneRepository)
    }

    func openRepositoryPicker() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.message = Localized.Menu.chooseAJujutsuRepository
        if panel.runModal() == .OK, let url = panel.url {
            openRepo(url.path)
        }
    }

    func openRemoteRepository() {
        if !liveWindows.contains(where: { $0.identifier?.rawValue.hasPrefix(AppWindows.openRemoteRepository) == true }) {
            openRemoteRepositoryGeneration += 1
        }
        presentWindow(AppWindows.openRemoteRepository)
    }
}
