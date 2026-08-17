import JayJayCore
import SwiftUI

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?
    @Environment(AppSettings.self) private var settings
    @Environment(RepoWindowManager.self) private var windowManager

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
        let result = await windowManager.loadRepoViewModel(
            at: path,
            includeSubmoduleStatuses: includeSubmodules
        )
        guard !Task.isCancelled else { return }
        if let model = result.viewModel {
            viewModel = model
            // Huge checkouts skip the snapshot on open (it's the slow part); small repos refresh eagerly.
            model.refresh(selecting: "@", snapshotWorkingCopy: !model.workingCopyIsLarge)
        } else if let error = result.error {
            initError = error
        } else {
            // Removal can begin after the scene was requested but before its repo open was registered.
            windowManager.closeRepoWindow(at: path)
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
