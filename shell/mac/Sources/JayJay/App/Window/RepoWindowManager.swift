import AppKit
import JayJayCore
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private(set) var openRepoPaths: [String] = []
    private let settings: AppSettings
    private var openRepoAction: ((String) -> Void)?
    private var showRepoListAction: ((Bool) -> Void)?
    private var isRepoListRequested = false

    init(settings: AppSettings) {
        self.settings = settings
    }

    func setWindowActions(
        openRepo: @escaping (String) -> Void,
        showRepoList: @escaping (_ openNewWindow: Bool) -> Void
    ) {
        openRepoAction = openRepo
        showRepoListAction = showRepoList
        isRepoListRequested = false
        refreshOpenRepoPaths()
    }

    func showRepoList() {
        if let window = NSApp.windows.first(where: {
            $0.identifier?.rawValue == AppWindows.welcome
        }) {
            isRepoListRequested = false
            activate(window)
            return
        }

        if let window = NSApp.windows.first(where: {
            $0.identifier?.rawValue == AppWindows.main
        }) {
            isRepoListRequested = false
            showRepoListAction?(false)
            activate(window)
            return
        }

        guard !isRepoListRequested, let showRepoListAction else { return }
        isRepoListRequested = true
        showRepoListAction(true)
    }

    func repoWindowWillClose() {
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            refreshOpenRepoPaths()
            if openRepoPaths.isEmpty {
                showRepoList()
            }
        }
    }

    func repoWindowDidAppear() {
        NSApp.windows
            .filter { $0.identifier?.rawValue == AppWindows.welcome }
            .forEach { $0.orderOut(nil) }
        refreshOpenRepoPaths()
    }

    func refreshOpenRepoPaths() {
        var seen = Set<String>()
        openRepoPaths = NSApp.windows.compactMap { window in
            guard window.isVisible || window.isMiniaturized,
                  let path = window.representedURL?.standardizedFileURL.path,
                  seen.insert(path).inserted
            else { return nil }
            return path
        }
    }

    func openRepositoryPicker() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.message = "Choose a Jujutsu repository"
        if panel.runModal() == .OK, let url = panel.url {
            openRepo(url.path)
        }
    }

    @discardableResult
    func activateRepo(_ path: String) -> Bool {
        let standardizedPath = URL(fileURLWithPath: path).standardizedFileURL.path
        if activateRepoWindow(matching: standardizedPath) {
            return true
        }
        let normalizedPath = normalizedRepositoryPath(path: path)
        return normalizedPath != standardizedPath && activateRepoWindow(matching: normalizedPath)
    }

    private func activateRepoWindow(matching path: String) -> Bool {
        guard let window = NSApp.windows.first(where: {
            $0.representedURL?.standardizedFileURL.path == path
        }) else { return false }
        activate(window)
        repoWindowDidAppear()
        return true
    }

    private func activate(_ window: NSWindow) {
        window.deminiaturize(nil)
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func openRepo(_ path: String) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        settings.recordOpenedRepo(normalizedPath)

        if activateRepoWindow(matching: normalizedPath) {
            return
        }

        openRepoAction?(normalizedPath)
    }
}
