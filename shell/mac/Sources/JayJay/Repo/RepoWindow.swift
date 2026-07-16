import JayJayCore
import SwiftUI

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Group {
            if let model = viewModel {
                RepoContentView(viewModel: model)
            } else if let err = initError {
                RepoInitErrorView(repoPath: repoPath, error: err, onInitialize: initJJRepo)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task { await openRepo() }
        .navigationTitle(URL(fileURLWithPath: repoPath).repositoryDisplayName)
        .toolbar(removing: .title)
        .background(WindowConfigurator { $0.representedURL = URL(fileURLWithPath: repoPath) })
    }

    private func openRepo() async {
        let path = repoPath
        let includeSubmodules = settings.enableGitSubmoduleSupport
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
        switch result {
            case let .success(opened):
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
                initError = error.friendlyDescription
        }
    }

    private func initJJRepo() {
        initError = nil
        let path = repoPath
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
