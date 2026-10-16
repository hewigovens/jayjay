import JayJayCore
import SwiftUI

struct FileHistoryComparisonDiff: View {
    let from: String?
    let to: String
    let path: String
    let repo: JayJayRepo?
    let diffStore: DiffStore

    @State private var hunk: DiffHunk?
    @State private var isLoaded = false

    var body: some View {
        content
            .task { await loadHunk() }
    }

    @ViewBuilder
    private var content: some View {
        if let hunk {
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
        } else if isLoaded {
            ContentUnavailableView(
                "No Changes to This File",
                systemImage: "doc",
                description: Text("The file is the same in both revisions.")
            )
        } else {
            Color.clear
        }
    }

    private func loadHunk() async {
        guard let repo else {
            isLoaded = true
            return
        }
        let (from, to, path) = (from, to, path)
        hunk = await Task.detached {
            let detail = try? {
                if let from {
                    return try repo.interdiffSummary(fromRev: from, toRev: to)
                }
                return try repo.showSummary(rev: to)
            }()
            return detail?.diff.first { $0.path == path }
        }.value
        isLoaded = true
    }
}
