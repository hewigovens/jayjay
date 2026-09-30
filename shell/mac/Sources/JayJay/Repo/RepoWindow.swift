import JayJayCore
import SwiftUI

struct RepoWindow: View {
    private static let removingWorkspace = "Cannot switch to a workspace while it is being removed."

    let repoPath: String
    let windowNumber: Int?
    let onSwitchWorkspace: (String) -> Void
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?
    @Environment(AppSettings.self) private var settings
    @Environment(RepoWindowManager.self) private var windowManager

    var body: some View {
        Group {
            if let model = viewModel {
                let isSwitching = model.repoPath != repoPath
                // Keep the old content visible during a switch, then reset only repository-scoped view state at handoff.
                RepoContentView(viewModel: model, isSwitchingWorkspace: isSwitching, onSwitchWorkspace: { path in
                    if let windowNumber {
                        windowManager.workspaceDrafts.preserve(from: model, in: windowNumber)
                    }
                    onSwitchWorkspace(path)
                })
                .id(model.repoPath)
                .disabled(isSwitching)
                .allowsHitTesting(!isSwitching)
            } else if let err = initError {
                RepoInitErrorView(repoPath: repoPath, error: err, onInitialize: initJJRepo)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task(id: repoPath) { await openRepo() }
        .navigationTitle(URL(fileURLWithPath: viewModel?.repoPath ?? repoPath).repositoryDisplayName)
        .toolbar(removing: .title)
        .background(WindowConfigurator { $0.representedURL = URL(fileURLWithPath: repoPath) })
    }

    private func openRepo() async {
        let path = repoPath
        guard viewModel?.repoPath != path else { return }
        let includeSubmodules = settings.enableGitSubmoduleSupport
        // Off the main thread so the app stays responsive while loading large checkouts.
        let result = await Task.detached {
            Result {
                let repo = try JayJayRepo.open(path: path)
                return (
                    repo: repo,
                    primaryRoot: workspacePrimaryRoot(path: path) ?? path,
                    workingCopyIsLarge: repo.workingCopyIsLarge(),
                    configWarning: repo.checkUserConfig()
                )
            }
        }.value
        guard !Task.isCancelled else { return }
        switch result {
            case let .success(opened):
                let model = RepoViewModel(
                    path: path,
                    repo: opened.repo,
                    primaryRoot: opened.primaryRoot,
                    workingCopyIsLarge: opened.workingCopyIsLarge,
                    configWarning: opened.configWarning,
                    includeSubmoduleStatuses: includeSubmodules
                )
                if let windowNumber, let draft = windowManager.workspaceDrafts.draft(for: path, in: windowNumber) {
                    model.commitSummaryDraft = draft.summary
                    model.commitDescriptionDraft = draft.body
                }
                guard windowManager.register(model) else {
                    leaveSwitch(error: Self.removingWorkspace, closingWindow: true)
                    return
                }
                await show(model)
            case let .failure(error):
                leaveSwitch(error: error.friendlyDescription, closingWindow: false)
        }
    }

    /// The source stays on screen until the destination's first refresh lands, so a switch paints once.
    private func show(_ model: RepoViewModel) async {
        if let reveal = windowManager.takePendingReveal(for: model.repoPath) {
            model.revealAncestors(of: reveal.headChangeId, selecting: reveal.rev)
        } else {
            // Huge checkouts skip the snapshot on open (it's the slow part); small repos refresh eagerly.
            model.refresh(selecting: "@", snapshotWorkingCopy: !model.workingCopyIsLarge)
        }
        if let previous = viewModel {
            await model.waitForFirstLoad()
            guard !Task.isCancelled else {
                windowManager.retire(model)
                return
            }
            let abandoned: String? = if model.isShuttingDown {
                Self.removingWorkspace
            } else if model.workspaceVanished {
                "The workspace no longer exists."
            } else {
                nil
            }
            if let abandoned {
                windowManager.retire(model)
                leaveSwitch(error: abandoned, closingWindow: true)
                return
            }
            windowManager.retire(previous)
        }
        initError = nil
        viewModel = model
    }

    /// Returns to the source while it can still be shown; otherwise fails the way a window without a source would.
    private func leaveSwitch(error: String, closingWindow: Bool) {
        if let previous = viewModel {
            if windowManager.endSwitch(returningTo: previous) {
                previous.error = error
                onSwitchWorkspace(previous.repoPath)
                return
            }
            viewModel = nil
        }
        if closingWindow {
            windowManager.closeRepoWindow(at: repoPath)
        } else {
            initError = error
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
