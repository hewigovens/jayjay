import AppKit
import JayJayCore

extension RepoWindowManager {
    /// Resolves nil when registration is refused or `timeout` passes before the window registers.
    func waitForRegistration(at path: String, timeout: Duration) async -> RepoViewModel? {
        let key = normalizedRepositoryPath(path: path)
        compactRegistrations()
        if let model = registeredViewModel(at: key) {
            return model
        }
        return await registrationWaiters.wait(for: key, timeout: timeout)
    }

    var scriptingRepositories: [ScriptRepository] {
        rankedScriptingModels.map(ScriptRepository.init)
    }

    var frontScriptingRepository: ScriptRepository? {
        rankedScriptingModels.first.map(ScriptRepository.init)
    }

    /// Ordered front to back, then hidden, then minimized: a hidden app has no visible windows but its repositories stay scriptable.
    private var rankedScriptingModels: [RepoViewModel] {
        compactRegistrations()
        let windows = liveRepoWindows
        let path = { (window: NSWindow) in
            window.representedURL.map { normalizedRepositoryPath(path: $0.repositoryLocationPath) }
        }
        let visible = NSApp.orderedWindows.filter { $0.isVisible && windows.contains($0) }.compactMap(path)
        let minimized = Set(windows.filter(\.isMiniaturized).compactMap(path))
        var seen = Set<String>()
        return registeredRepos.values
            .compactMap { registration -> (path: String, model: RepoViewModel)? in
                guard let model = registration.viewModel, model.switchTarget == nil, !model.isShuttingDown,
                      seen.insert(registration.path).inserted else { return nil }
                return (registration.path, model)
            }
            .map { (model: $0.model, path: $0.path, rank: visible.firstIndex(of: $0.path) ?? visible.count + (minimized.contains($0.path) ? 1 : 0)) }
            .sorted { ($0.rank, $0.path) < ($1.rank, $1.path) }
            .map(\.model)
    }
}
