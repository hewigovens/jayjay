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
    private var openWindowAction: ((_ id: String, _ value: String?) -> Void)?
    private var dismissWindowAction: ((_ id: String) -> Void)?
    var pendingRepoAfterOnboarding: String?
    /// SwiftUI's launch presentation is only a hint; applied once the first scene registers its window actions.
    var launchScene: LaunchScene?
    private var registeredRepos: [ObjectIdentifier: RegisteredRepo] = [:]
    private var removalCountsByRepoPath: [String: Int] = [:]
    private var removalCountsByWorkspace: [WorkspaceKey: Int] = [:]

    init(settings: AppSettings) {
        self.settings = settings
    }

    func setWindowActions(
        presenting sceneID: String,
        openWindow: @escaping (_ id: String, _ value: String?) -> Void,
        dismissWindow: @escaping (_ id: String) -> Void
    ) {
        openWindowAction = openWindow
        dismissWindowAction = dismissWindow
        refreshOpenRepoPaths()
        guard let launchScene else { return }
        self.launchScene = nil
        if !launchScene.isPresented(by: sceneID) {
            DispatchQueue.main.async { [weak self] in self?.apply(launchScene) }
        }
    }

    /// Present the routed scene before dismissing the one SwiftUI chose: the window actions belong to that scene and die with it.
    private func apply(_ scene: LaunchScene) {
        switch scene {
            case .externalTool:
                [AppWindows.onboarding, AppWindows.repoList, AppWindows.repo].forEach { dismissWindowAction?($0) }
            case .onboarding:
                showOnboarding()
                dismissWindowAction?(AppWindows.repoList)
                dismissWindowAction?(AppWindows.repo)
            case let .repo(path):
                openRepo(path)
                dismissWindowAction?(AppWindows.onboarding)
                dismissWindowAction?(AppWindows.repoList)
            case .repoList:
                showRepoList()
                dismissWindowAction?(AppWindows.onboarding)
                dismissWindowAction?(AppWindows.repo)
        }
    }

    /// Deferred behind any pending launch route: the window actions belong to the empty window and die with it.
    func emptyRepoWindowDidAppear() {
        DispatchQueue.main.async { [weak self] in self?.dismissWindowAction?(AppWindows.repo) }
    }

    func showRepoList() {
        guard settings.hasCompletedOnboarding else {
            showOnboarding()
            return
        }
        openWindowAction?(AppWindows.repoList, nil)
        // openWindow alone leaves a miniaturized window in the Dock; reopen must bring it back.
        activateWindow(identified: AppWindows.repoList)
    }

    func finishOnboarding() {
        settings.hasCompletedOnboarding = true
        if let path = pendingRepoAfterOnboarding {
            pendingRepoAfterOnboarding = nil
            openRepo(path)
        } else {
            showRepoList()
        }
        dismissWindowAction?(AppWindows.onboarding)
    }

    private func showOnboarding() {
        openWindowAction?(AppWindows.onboarding, nil)
        activateWindow(identified: AppWindows.onboarding)
    }

    private func hideRepoList() {
        dismissWindowAction?(AppWindows.repoList)
    }

    private func activateWindow(identified id: String) {
        if let window = liveWindows.first(where: { $0.identifier?.rawValue == id }) {
            activate(window)
        }
    }

    /// A closed window can linger in NSApp.windows until SwiftUI releases it; activating one would resurrect a shut-down repository.
    private var liveWindows: [NSWindow] {
        NSApp.windows.filter { $0.isVisible || $0.isMiniaturized }
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

    func prepareForTermination() {
        compactRegistrations()
        registeredRepos.values.compactMap(\.viewModel).forEach { $0.prepareForTermination() }
    }

    func repoWindowDidAppear() {
        hideRepoList()
        refreshOpenRepoPaths()
    }

    func refreshOpenRepoPaths() {
        var seen = Set<String>()
        openRepoPaths = liveWindows.compactMap { window in
            guard let path = window.representedURL?.standardizedFileURL.path,
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
        repoWindows(at: [normalizedRepositoryPath(path: path)]).forEach { $0.close() }
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
        liveWindows.filter {
            guard let path = $0.representedURL?.path else { return false }
            return paths.contains(normalizedRepositoryPath(path: path))
        }
    }

    private func activateRepoWindow(matching path: String) -> Bool {
        guard let window = liveWindows.first(where: {
            $0.representedURL?.standardizedFileURL.path == path
        }) else { return false }
        activate(window)
        hideRepoList()
        refreshOpenRepoPaths()
        return true
    }

    private func activate(_ window: NSWindow) {
        window.deminiaturize(nil)
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func openRepo(_ path: String) {
        let normalizedPath = normalizedRepositoryPath(path: URL(fileURLWithPath: path).standardizedFileURL.path)
        guard !isRemovingRepo(at: path) else { return }
        settings.recordOpenedRepo(normalizedPath)
        guard settings.hasCompletedOnboarding else {
            pendingRepoAfterOnboarding = normalizedPath
            showOnboarding()
            return
        }
        if activateRepo(path) {
            return
        }
        openWindowAction?(AppWindows.repo, normalizedPath)
    }
}
