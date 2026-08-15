import JayJayCore
import SwiftUI

struct RepoWindow: View {
    let repoPath: String
    var onBoundPathChange: (String) -> Void = { _ in }
    @State private var boundPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?
    @State private var skipSnapshotOnNextOpen = false
    @Environment(AppSettings.self) private var settings

    init(repoPath: String, onBoundPathChange: @escaping (String) -> Void = { _ in }) {
        self.repoPath = repoPath
        self.onBoundPathChange = onBoundPathChange
        _boundPath = State(initialValue: repoPath)
    }

    var body: some View {
        Group {
            if let model = viewModel {
                RepoContentView(viewModel: model, onRebindWorkspace: rebind)
            } else if let err = initError {
                RepoInitErrorView(repoPath: boundPath, error: err, onInitialize: initJJRepo)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task(id: boundPath) { await openBoundRepo() }
        .navigationTitle(URL(fileURLWithPath: boundPath).repositoryDisplayName)
        .toolbar(removing: .title)
        .background(WindowConfigurator { $0.representedURL = URL(fileURLWithPath: boundPath) })
    }

    private func rebind(_ path: String) {
        guard path != boundPath else { return }
        initError = nil
        skipSnapshotOnNextOpen = true
        boundPath = path
        onBoundPathChange(path)
    }

    private func openBoundRepo() async {
        let path = boundPath
        let reuseViewModel = skipSnapshotOnNextOpen
        skipSnapshotOnNextOpen = false
        // Last click wins: a later rebind cancels this task during the settle delay.
        if reuseViewModel {
            do {
                try await Task.sleep(for: .milliseconds(160))
            } catch {
                return
            }
            guard path == boundPath else { return }
        }
        if reuseViewModel, viewModel?.hasSyncInFlight == true { return }
        await openRepo(path: path, reuseViewModel: reuseViewModel)
    }

    private func openRepo(path: String? = nil, reuseViewModel: Bool? = nil) async {
        let path = path ?? boundPath
        let includeSubmodules = settings.enableGitSubmoduleSupport
        let reuseViewModel = reuseViewModel ?? false
        let switchGeneration = viewModel?.workspaceSwitchGeneration
        if reuseViewModel {
            viewModel?.isOpeningWorkspace = true
        }
        // Off the main thread so the app stays responsive while loading large checkouts.
        let result = await Task.detached {
            Result {
                let repo = try JayJayRepo.open(path: path)
                return (
                    repo: repo,
                    workingCopyIsLarge: repo.workingCopyIsLarge(),
                    configWarning: repo.checkUserConfig()
                )
            }
        }.value
        guard path == boundPath else {
            if reuseViewModel, viewModel?.workspaceSwitchGeneration == switchGeneration {
                viewModel?.isOpeningWorkspace = false
            }
            return
        }
        switch result {
            case let .success(opened):
                if reuseViewModel, let model = viewModel {
                    // Keep the existing chrome; only retarget the repo. Never snapshot on switch —
                    // that was the multi-second hitch on large working copies.
                    let revision = model.workspaces.first(where: \.isCurrent)?.wcCommitId ?? "@"
                    model.attachWorkspace(
                        path: path,
                        repo: opened.repo,
                        workingCopyIsLarge: opened.workingCopyIsLarge,
                        configWarning: opened.configWarning,
                        selecting: revision.isEmpty ? "@" : revision
                    )
                    model.isOpeningWorkspace = false
                    model.pendingWorkspacePath = nil
                    return
                }
                let model = RepoViewModel(
                    path: path,
                    repo: opened.repo,
                    workingCopyIsLarge: opened.workingCopyIsLarge,
                    configWarning: opened.configWarning,
                    includeSubmoduleStatuses: includeSubmodules
                )
                viewModel = model
                // Huge checkouts skip the snapshot on open (it's the slow part); small repos refresh eagerly.
                model.refresh(selecting: "@", snapshotWorkingCopy: !model.workingCopyIsLarge)
            case let .failure(error):
                if reuseViewModel {
                    viewModel?.isOpeningWorkspace = false
                }
                if viewModel == nil {
                    initError = error.friendlyDescription
                } else {
                    viewModel?.error = error.friendlyDescription
                }
        }
    }

    private func initJJRepo() {
        initError = nil
        let path = boundPath
        Task {
            let result = await Task.detached {
                Result {
                    try initJjGitRepo(path: path)
                }
            }.value
            switch result {
                case .success:
                    await openRepo()
                case let .failure(error):
                    initError = error.friendlyDescription
            }
        }
    }
}

private struct RepoInitErrorView: View {
    let repoPath: String
    let error: String
    let onInitialize: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 40))
                .foregroundStyle(.orange)
            Text("Failed to open repository")
                .jayjayFont(16, weight: .semibold)
            Text(error)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 360)
            if !FileManager.default.fileExists(atPath: "\(repoPath)/.jj") {
                Button("Initialize with jj git init", action: onInitialize)
                    .buttonStyle(.borderedProminent)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
