import AppKit
import JayJayCore
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private struct WeakRepoViewModel {
        weak var value: RepoViewModel?
    }

    private struct OpenedRepo: Sendable {
        let repo: JayJayRepo
        let workingCopyIsLarge: Bool
        let configWarning: String?
    }

    private enum RepoOpenResult: Sendable {
        case success(OpenedRepo)
        case failure(String)
    }

    private(set) var openRepoPaths: [String] = []
    private let settings: AppSettings
    private var openRepoAction: ((String) -> Void)?
    private var showRepoListAction: ((Bool) -> Void)?
    private var isRepoListRequested = false
    /// Weakly held per-path view models, so destructive flows can quiesce every window's repo tasks before its checkout is moved or deleted.
    private var repoViewModels: [String: [WeakRepoViewModel]] = [:]
    /// Repository opens begin before a view model exists, so removal tracks and awaits them separately.
    private var repoOpenTasks: [String: [UUID: Task<RepoOpenResult, Never>]] = [:]
    /// Counts active destructive flows per repository, so one completion cannot release another flow's open barrier.
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

    /// Close every window showing `path`. Callers forgetting or deleting a workspace must close its windows first, or a window can keep snapshotting a working copy that no longer exists.
    func closeRepoWindow(at path: String) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        NSApp.windows.filter {
            $0.representedURL?.standardizedFileURL.path == normalizedPath
        }.forEach { $0.close() }
        refreshOpenRepoPaths()
    }

    private func registerRepoViewModel(_ viewModel: RepoViewModel) {
        repoViewModels = repoViewModels.compactMapValues { models in
            let liveModels = models.filter { $0.value != nil }
            return liveModels.isEmpty ? nil : liveModels
        }
        let path = normalizedRepositoryPath(path: viewModel.repoPath)
        repoViewModels[path, default: []].append(WeakRepoViewModel(value: viewModel))
    }

    func loadRepoViewModel(
        at path: String,
        includeSubmoduleStatuses: Bool
    ) async -> (viewModel: RepoViewModel?, error: String?) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        guard !isRemovingRepo(at: normalizedPath) else { return (nil, nil) }
        let taskId = UUID()
        let task = Task.detached {
            do {
                let repo = try JayJayRepo.open(path: path)
                return RepoOpenResult.success(
                    OpenedRepo(
                        repo: repo,
                        workingCopyIsLarge: repo.workingCopyIsLarge(),
                        configWarning: repo.checkUserConfig()
                    )
                )
            } catch {
                return RepoOpenResult.failure(error.friendlyDescription)
            }
        }
        repoOpenTasks[normalizedPath, default: [:]][taskId] = task
        let result = await withTaskCancellationHandler {
            await task.value
        } onCancel: {
            task.cancel()
        }
        guard repoOpenTasks[normalizedPath]?[taskId] != nil else { return (nil, nil) }
        repoOpenTasks[normalizedPath]?[taskId] = nil
        if repoOpenTasks[normalizedPath]?.isEmpty == true {
            repoOpenTasks[normalizedPath] = nil
        }
        guard !Task.isCancelled, !isRemovingRepo(at: normalizedPath) else {
            return (nil, nil)
        }
        switch result {
            case let .success(opened):
                let viewModel = RepoViewModel(
                    path: path,
                    repo: opened.repo,
                    workingCopyIsLarge: opened.workingCopyIsLarge,
                    configWarning: opened.configWarning,
                    includeSubmoduleStatuses: includeSubmoduleStatuses
                )
                registerRepoViewModel(viewModel)
                return (viewModel, nil)
            case let .failure(error):
                return (nil, error)
        }
    }

    /// Quiesces the window's repo tasks and only then closes it, so a caller about to forget, move, or delete the checkout cannot race an in-flight snapshot or mutation. Quiescing must precede the close: closing releases the view model, after which its tasks can no longer be awaited.
    func closeRepoWindowForWorkspaceRemoval(at path: String) async {
        let normalizedPath = normalizedRepositoryPath(path: path)
        removalCountsByRepoPath[normalizedPath, default: 0] += 1
        if let openTasks = repoOpenTasks[normalizedPath] {
            for task in openTasks.values {
                task.cancel()
            }
            for task in openTasks.values {
                _ = await task.value
            }
            repoOpenTasks[normalizedPath] = nil
        }
        let viewModels = repoViewModels[normalizedPath]?.compactMap(\.value) ?? []
        for viewModel in viewModels {
            await viewModel.prepareForRemoval()
        }
        repoViewModels[normalizedPath] = nil
        closeRepoWindow(at: path)
    }

    /// A workspace forgotten outside this window still needs the same shutdown barrier, but there is no local removal flow to release the open exclusion afterward.
    func closeRepoWindowAfterWorkspaceVanished(at path: String) async {
        await closeRepoWindowForWorkspaceRemoval(at: path)
        finishWorkspaceRemoval(at: path)
    }

    func finishWorkspaceRemoval(at path: String) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        guard let count = removalCountsByRepoPath[normalizedPath] else { return }
        if count == 1 {
            removalCountsByRepoPath[normalizedPath] = nil
        } else {
            removalCountsByRepoPath[normalizedPath] = count - 1
        }
    }

    private func isRemovingRepo(at normalizedPath: String) -> Bool {
        removalCountsByRepoPath[normalizedPath] != nil
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
        guard !isRemovingRepo(at: normalizedPath) else { return }
        settings.recordOpenedRepo(normalizedPath)

        if activateRepoWindow(matching: normalizedPath) {
            return
        }

        openRepoAction?(normalizedPath)
    }
}
