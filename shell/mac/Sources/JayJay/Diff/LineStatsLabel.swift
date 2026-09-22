import JayJayCore
import SwiftUI

/// `+N -M` for one file, omitting a zero side; renders nothing when the file has no line changes.
struct LineStatsLabel: View {
    let stats: FileDiffStats
    var size: CGFloat = 11
    var muted = false

    var body: some View {
        if stats.hasLineChanges {
            HStack(spacing: 4) {
                if stats.insertions > 0 {
                    Text("+\(stats.insertions)")
                        .foregroundStyle(muted ? AnyShapeStyle(.secondary) : AnyShapeStyle(.green))
                }
                if stats.deletions > 0 {
                    Text("-\(stats.deletions)")
                        .foregroundStyle(muted ? AnyShapeStyle(.secondary) : AnyShapeStyle(.red))
                }
            }
            .jayjayFont(size, weight: .semibold, design: .monospaced)
            .fixedSize()
            .accessibilityElement(children: .ignore)
            .accessibilityLabel("\(stats.insertions) added, \(stats.deletions) removed")
        }
    }
}

extension FileDiffStats {
    var hasLineChanges: Bool { insertions > 0 || deletions > 0 }
}
