import AppKit
import JayJayCore
import SwiftUI

@MainActor
@Observable
final class RepoWindowManager {
    private struct WorkspaceKey: Hashable {
        let repositoryStorePath: String
        let name: String
    }

    private struct RegisteredRepo {
        weak var viewModel: RepoViewModel?
        let path: String
        let workspace: WorkspaceKey
    }

    private(set) var openRepoPaths: [String] = []
    private let settings: AppSettings
    private var openRepoAction: ((String) -> Void)?
    private var showRepoListAction: ((Bool) -> Void)?
    private var isRepoListRequested = false
    private var registeredRepos: [ObjectIdentifier: RegisteredRepo] = [:]
    private var removalCountsByRepoPath: [String: Int] = [:]
    private var removalCountsByWorkspace: [WorkspaceKey: Int] = [:]

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

    func repoWindowWillClose(at path: String) {
        let normalizedPath = normalizedRepositoryPath(path: path)
        compactRegistrations()
        let closing = registeredRepos.filter { $0.value.path == normalizedPath }
        for registration in closing.values {
            registration.viewModel?.beginShutdown()
        }
        registeredRepos = registeredRepos.filter { $0.value.path != normalizedPath }
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
            guard let representedPath = $0.representedURL?.path else { return false }
            return normalizedRepositoryPath(path: representedPath) == normalizedPath
        }.forEach { $0.close() }
        refreshOpenRepoPaths()
    }

    func register(_ viewModel: RepoViewModel) -> Bool {
        let path = normalizedRepositoryPath(path: viewModel.repoPath)
        let workspace = workspaceKey(for: viewModel)
        guard !isRemovingRepo(at: path), removalCountsByWorkspace[workspace] == nil else { return false }
        compactRegistrations()
        registeredRepos[ObjectIdentifier(viewModel)] = RegisteredRepo(
            viewModel: viewModel,
            path: path,
            workspace: workspace
        )
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
        compactRegistrations()
        let removing = registeredRepos.filter { $0.value.path == normalizedPath }
        for viewModel in removing.values.compactMap(\.viewModel) {
            await viewModel.prepareForRemoval()
        }
        await body()
        registeredRepos = registeredRepos.filter { $0.value.path != normalizedPath }
        closeRepoWindow(at: path)
    }

    /// Keep windows quiesced but visible until the forget succeeds, then close every path for that workspace identity.
    func withWorkspaceRemoval(
        _ workspace: WorkspaceInfo,
        from sourceViewModel: RepoViewModel,
        _ body: @MainActor () async -> Bool
    ) async {
        let key = WorkspaceKey(
            repositoryStorePath: sourceViewModel.repo.repositoryStorePath(),
            name: workspace.name
        )
        let normalizedPath = workspace.path.isEmpty ? nil : normalizedRepositoryPath(path: workspace.path)
        incrementRemovalCount(for: key, path: normalizedPath)
        defer { decrementRemovalCount(for: key, path: normalizedPath) }

        compactRegistrations()
        let removing = registeredRepos.filter {
            $0.value.workspace == key || $0.value.path == normalizedPath
        }
        let paths = Set(removing.values.map(\.path) + [normalizedPath].compactMap(\.self))
        let windows = repoWindows(at: paths)
        let viewModels = removing.values.compactMap(\.viewModel)
        for viewModel in viewModels {
            await viewModel.prepareForRemoval()
        }

        guard await body() else {
            viewModels.forEach { $0.resumeAfterFailedRemoval() }
            return
        }

        registeredRepos = registeredRepos.filter { !removing.keys.contains($0.key) }
        windows.forEach { $0.close() }
        paths.forEach(closeRepoWindow)
    }

    func isRemovingRepo(at path: String) -> Bool {
        removalCountsByRepoPath[normalizedRepositoryPath(path: path)] != nil
    }

    private func compactRegistrations() {
        registeredRepos = registeredRepos.filter { $0.value.viewModel != nil }
    }

    private func workspaceKey(for viewModel: RepoViewModel) -> WorkspaceKey {
        WorkspaceKey(
            repositoryStorePath: viewModel.repo.repositoryStorePath(),
            name: viewModel.repo.workspaceName()
        )
    }

    private func incrementRemovalCount(for workspace: WorkspaceKey, path: String?) {
        removalCountsByWorkspace[workspace, default: 0] += 1
        if let path {
            removalCountsByRepoPath[path, default: 0] += 1
        }
    }

    private func decrementRemovalCount(for workspace: WorkspaceKey, path: String?) {
        if let count = removalCountsByWorkspace[workspace] {
            removalCountsByWorkspace[workspace] = count > 1 ? count - 1 : nil
        }
        if let path, let count = removalCountsByRepoPath[path] {
            removalCountsByRepoPath[path] = count > 1 ? count - 1 : nil
        }
    }

    private func repoWindows(at paths: Set<String>) -> [NSWindow] {
        NSApp.windows.filter {
            guard let path = $0.representedURL?.path else { return false }
            return paths.contains(normalizedRepositoryPath(path: path))
        }
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
