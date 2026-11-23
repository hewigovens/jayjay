import JayJayCore
import SwiftUI

struct DiffFileTitle: View {
    let hunk: DiffHunk
    var stats: FileDiffStats?

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.appThemes) private var appThemes

    var body: some View {
        let statusColor = appThemes.colors(for: colorScheme).fileColor(for: hunk.hunkType)
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            FileIcon(path: hunk.path)
            if hunk.hunkType.hasSource, let oldPath = hunk.oldPath {
                Text(oldPath)
                    .jayjayFont(11, design: .monospaced)
                    .strikethrough(hunk.hunkType == .renamed)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .help(oldPath)
                    .foregroundStyle(.secondary)
                Image(systemName: "arrow.right")
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
            }
            FilePathLabel(path: hunk.path, size: 13)
                .textSelection(.enabled)
                .help(hunk.path)
                .accessibilityIdentifier(AID.Diff.section)
            // The glyph's ink sits low on the baseline next to the path's x-height.
            CopyIconButton(value: hunk.path, help: Localized.Diff.copyPath)
                .alignmentGuide(.firstTextBaseline) { $0[.firstTextBaseline] + 1 }
            Text(hunk.hunkType.label)
                .jayjayFont(10, weight: .medium)
                .foregroundStyle(statusColor)
                .padding(.horizontal, 7)
                .padding(.vertical, 1)
                .background(statusColor.opacity(0.12), in: Capsule())
                .fixedSize()
            if let stats {
                LineStatsLabel(stats: stats)
            }
        }
    }
}
