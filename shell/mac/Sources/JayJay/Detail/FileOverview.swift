import JayJayCore
import SwiftUI

/// Shown when no file is selected: every file with its line counts, like `jj diff --stat`.
struct FileOverview: View {
    let hunks: [DiffHunk]
    let stats: [String: FileDiffStats]
    let onSelect: (String) -> Void

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.appThemes) private var appThemes

    private var largestChange: UInt32 {
        max(1, hunks.map { (stats[$0.path]?.insertions ?? 0) + (stats[$0.path]?.deletions ?? 0) }.max() ?? 1)
    }

    var body: some View {
        let colors = appThemes.colors(for: colorScheme)
        ScrollView {
            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 0) {
                ForEach(hunks, id: \.path) { hunk in
                    let fileStats = stats[hunk.path]
                    GridRow {
                        hunk.hunkType.badge(colorScheme, colors: colors)
                            .jayjayFont(16)
                        pathLabel(hunk.path)
                        Text(fileStats.map { "+\($0.insertions)" } ?? "")
                            .foregroundStyle(.green)
                            .gridColumnAlignment(.trailing)
                        Text(fileStats.map { "-\($0.deletions)" } ?? "")
                            .foregroundStyle(.red)
                            .gridColumnAlignment(.trailing)
                        changeBar(fileStats)
                    }
                    .jayjayFont(12, design: .monospaced)
                    .padding(.vertical, 5)
                    .contentShape(Rectangle())
                    .onTapGesture { onSelect(hunk.path) }
                }
            }
            .padding(.horizontal, PaneLayout.detailInset)
            .padding(.vertical, 12)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private func pathLabel(_ path: String) -> some View {
        let folder = (path as NSString).deletingLastPathComponent
        let name = (path as NSString).lastPathComponent
        return (Text(folder.isEmpty ? "" : folder + "/").foregroundStyle(.secondary) + Text(name))
            .lineLimit(1)
            .truncationMode(.head)
    }

    private func changeBar(_ fileStats: FileDiffStats?) -> some View {
        let width: CGFloat = 120
        let added = CGFloat(fileStats?.insertions ?? 0) / CGFloat(largestChange) * width
        let removed = CGFloat(fileStats?.deletions ?? 0) / CGFloat(largestChange) * width
        return HStack(spacing: 0) {
            Rectangle().fill(.green.opacity(0.7)).frame(width: added)
            Rectangle().fill(.red.opacity(0.7)).frame(width: removed)
            Spacer(minLength: 0)
        }
        .frame(width: width, height: 6)
        .clipShape(Capsule())
    }
}
