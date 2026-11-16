import JayJayCore
import SwiftUI

/// Shown when no file is selected: every file's diff in one scroll, each under its own header with line counts.
struct AllFilesDiff: View {
    let hunks: [DiffHunk]
    let rev: String
    let commitId: String
    let repo: JayJayRepo?
    let actions: (any ChangeActions & DAGActions)?
    let isWorkingCopy: Bool
    let diffStore: DiffStore
    let stats: [String: FileDiffStats]

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 0) {
                ForEach(hunks, id: \.path) { hunk in
                    DiffSection(
                        hunk: hunk,
                        rev: rev,
                        commitId: commitId,
                        repo: repo,
                        actions: actions,
                        isWorkingCopy: isWorkingCopy,
                        diffStore: diffStore,
                        reviewStore: nil,
                        noteEditor: .constant(nil),
                        lineStats: stats[hunk.path],
                        fitsContent: true
                    )
                    .id("\(commitId)|\(hunk.path)")
                    Divider()
                }
            }
        }
    }
}
