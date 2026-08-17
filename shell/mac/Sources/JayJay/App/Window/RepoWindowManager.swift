import AppKit
import JayJayCore
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private struct WeakRepoViewModel {
        weak var value: RepoViewModel?
    }

    private(set) var openRepoPaths: [String] = []
    private let settings: AppSettings
    private var openRepoAction: ((String) -> Void)?
    private var showRepoListAction: ((Bool) -> Void)?
    private var isRepoListRequested = false
    private var repoViewModels: [String: [WeakRepoViewModel]] = [:]
    private var removalCountsByRepoPath: [String: Int] = [:]

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

        // On-screen or miniaturized only: a mid-close window is neither, and reactivating it would countermand the close and strand the window; a miniaturized one must still deminiaturize on Dock reopen.
        if let window = NSApp.windows.first(where: {
            $0.identifier?.rawValue == AppWindows.main && ($0.isVisible || $0.isMiniaturized)
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

    func closeRepoWindow(at path: String) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        NSApp.windows.filter {
            $0.representedURL?.standardizedFileURL.path == normalizedPath
        }.forEach { $0.close() }
        refreshOpenRepoPaths()
    }

    func register(_ viewModel: RepoViewModel) -> Bool {
        let path = normalizedRepositoryPath(path: viewModel.repoPath)
        guard !isRemovingRepo(at: path) else { return false }
        repoViewModels = repoViewModels.compactMapValues { models in
            let liveModels = models.filter { $0.value != nil }
            return liveModels.isEmpty ? nil : liveModels
        }
        repoViewModels[path, default: []].append(WeakRepoViewModel(value: viewModel))
        return true
    }

    /// Quiesce before close: closing releases the view model and its tasks with it.
    func withWorkspaceRemoval(at path: String, _ body: @MainActor () async -> Void) async {
        guard !path.isEmpty else { return await body() }
        let normalizedPath = normalizedRepositoryPath(path: path)
        removalCountsByRepoPath[normalizedPath, default: 0] += 1
        defer {
            if let count = removalCountsByRepoPath[normalizedPath] {
                removalCountsByRepoPath[normalizedPath] = count > 1 ? count - 1 : nil
            }
        }
        for viewModel in repoViewModels[normalizedPath]?.compactMap(\.value) ?? [] {
            await viewModel.prepareForRemoval()
        }
        repoViewModels[normalizedPath] = nil
        closeRepoWindow(at: path)
        await body()
    }

    func isRemovingRepo(at path: String) -> Bool {
        removalCountsByRepoPath[normalizedRepositoryPath(path: path)] != nil
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
        guard !isRemovingRepo(at: path) else { return }
        settings.recordOpenedRepo(normalizedPath)

        if activateRepoWindow(matching: normalizedPath) {
            return
        }

        openRepoAction?(normalizedPath)
    }
}
