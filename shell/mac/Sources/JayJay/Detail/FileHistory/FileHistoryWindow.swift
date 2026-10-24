import AppKit
import JayJayCore
import SwiftUI

struct FileHistoryWindow: View {
    let request: FileHistoryRequest
    @Environment(RepoWindowManager.self) private var windowManager
    @Environment(\.dismiss) private var dismiss
    @State private var repo: JayJayRepo?
    @State private var history: [ChangeInfo]?
    @State private var error: String?
    @State private var diffStore = DiffStore()

    var body: some View {
        content
            .frame(minWidth: 760, minHeight: 420)
            .navigationTitle(Localized.Repo.historyTitle(path: request.path))
            .navigationSubtitle(URL(fileURLWithPath: request.repoPath).repositoryDisplayName)
            .background(WindowFramePersistence(key: AppWindows.fileHistory))
            .task { await load() }
            .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
                Task { await load() }
            }
    }

    @ViewBuilder
    private var content: some View {
        if let history {
            FileHistoryView(
                history: history,
                path: request.path,
                repo: repo,
                diffStore: diffStore,
                onSelectChange: { windowManager.revealInGraph(repoPath: request.repoPath, revision: $0) },
                onDismiss: { dismiss() }
            )
        } else if let error {
            ContentUnavailableView(Localized.Repo.historyUnavailable, systemImage: "exclamationmark.triangle", description: Text(error))
        } else {
            Color.clear
        }
    }

    private func load() async {
        let (repoPath, path, opened) = (request.repoPath, request.path, repo)
        let result = await Task.detached {
            Result {
                let repo = try opened ?? JayJayRepo.open(path: repoPath)
                return try (repo, repo.fileHistory(path: path))
            }
        }.value
        switch result {
            case let .success((repo, history)):
                self.repo = repo
                self.history = history
                error = nil
            case let .failure(failure):
                error = failure.localizedDescription
        }
    }
}
