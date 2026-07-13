import AppKit
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private let settings: AppSettings
    private var openRepoAction: ((String) -> Void)?
    private var showRepoListAction: (() -> Void)?
    private var isRepoListRequested = false

    init(settings: AppSettings) {
        self.settings = settings
    }

    func setWindowActions(
        openRepo: @escaping (String) -> Void,
        showRepoList: @escaping () -> Void
    ) {
        openRepoAction = openRepo
        showRepoListAction = showRepoList
        isRepoListRequested = false
    }

    func showRepoList() {
        if let window = NSApp.windows.first(where: {
            $0.identifier?.rawValue == AppWindows.welcome
        }) {
            isRepoListRequested = false
            window.deminiaturize(nil)
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        guard !isRepoListRequested, let showRepoListAction else { return }
        isRepoListRequested = true
        showRepoListAction()
    }

    func repoWindowWillClose() {
        DispatchQueue.main.async { [weak self] in
            let hasOpenRepoWindow = NSApp.windows.contains {
                $0.representedURL != nil && ($0.isVisible || $0.isMiniaturized)
            }
            if !hasOpenRepoWindow {
                self?.showRepoList()
            }
        }
    }

    func repoWindowDidAppear() {
        NSApp.windows
            .filter { $0.identifier?.rawValue == AppWindows.welcome }
            .forEach { $0.orderOut(nil) }
    }

    func openRepo(_ path: String) {
        let normalizedPath = URL(fileURLWithPath: path).standardizedFileURL.path
        settings.recordOpenedRepo(normalizedPath)

        // Front a live window by representedURL; when all are closed none match, so we open fresh.
        if let window = NSApp.windows.first(where: {
            $0.representedURL?.standardizedFileURL.path == normalizedPath
        }) {
            window.deminiaturize(nil)
            window.makeKeyAndOrderFront(nil)
            repoWindowDidAppear()
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        openRepoAction?(normalizedPath)
    }
}
