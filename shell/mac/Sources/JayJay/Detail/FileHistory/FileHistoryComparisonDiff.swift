import JayJayCore
import SwiftUI

struct FileHistoryComparisonDiff: View {
    let from: String?
    let to: String
    let path: String
    let repo: JayJayRepo?
    let diffStore: DiffStore

    @State private var loaded: Result<DiffHunk?, any Error>?
    @State private var attempt = 0

    var body: some View {
        content
            .task(id: attempt) { await loadHunk() }
    }

    @ViewBuilder
    private var content: some View {
        switch loaded {
            case let .success(hunk?):
                DiffSection(
                    hunk: hunk,
                    rev: to,
                    repo: repo,
                    actions: nil,
                    isWorkingCopy: false,
                    diffStore: diffStore,
                    reviewStore: nil,
                    noteEditor: .constant(nil),
                    compareFromRev: from
                )
            case .success(nil):
                ContentUnavailableView(
                    Localized.Repo.noChangesToThisFile,
                    systemImage: "doc",
                    description: Text(Localized.Repo.theFileIsTheSameIn)
                )
            case let .failure(error):
                DiffLoadErrorView(message: error.friendlyDescription) {
                    loaded = nil
                    attempt += 1
                }
            case nil:
                Color.clear
        }
    }

    private func loadHunk() async {
        guard let repo else {
            loaded = .success(nil)
            return
        }
        let (from, to, path) = (from, to, path)
        loaded = await Task.detached {
            Result {
                let detail = if let from {
                    try repo.interdiffSummary(fromRev: from, toRev: to)
                } else {
                    try repo.showSummary(rev: to)
                }
                return detail.diff.first { $0.path == path }
            }
        }.value
    }
}
