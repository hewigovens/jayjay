import JayJayCore
import JayJayDiffUI
import SwiftUI

struct MergeHunkList: View {
    let highlights: [MergeHunkHighlights]
    let result: String
    @Binding var selectedHunk: UInt32?
    let onUseSource: (MergeEditorHunk, MergeHunkSource) -> Void

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 12) {
                ForEach(highlights) { item in
                    let unresolved = mergeHunkIsUnresolved(result: result, hunk: item.hunk)
                    MergeHunkCard(
                        highlights: item,
                        isUnresolved: unresolved,
                        onSelect: { selectedHunk = item.id },
                        onUseSource: { source in
                            selectedHunk = item.id
                            onUseSource(item.hunk, source)
                        }
                    )
                }
            }
            .padding(12)
        }
        .accessibilityIdentifier(AID.Conflict.editorHunkList)
        .background(Color(nsColor: .textBackgroundColor))
    }
}

private struct MergeHunkCard: View {
    let highlights: MergeHunkHighlights
    let isUnresolved: Bool
    let onSelect: () -> Void
    let onUseSource: (MergeHunkSource) -> Void

    @State private var measuredDiffHeight: CGFloat?
    @Environment(\.jayjayFontSize) private var fontSize

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Text("Conflict \(highlights.hunk.index + 1)")
                    .jayjayFont(11, weight: .semibold)
                Text(isUnresolved ? "Unresolved" : "Resolved")
                    .jayjayFont(10, weight: .medium)
                    .foregroundStyle(isUnresolved ? .orange : .green)
                Spacer()
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 7)
            Divider()
            HStack(spacing: 12) {
                actionButton("Accept Left", shortcut: "⌥←", source: .left)
                    .accessibilityIdentifier(AID.Conflict.hunkUse(highlights.hunk.index, "left"))
                actionButton("Accept Right", shortcut: "⌥→", source: .right)
                    .accessibilityIdentifier(AID.Conflict.hunkUse(highlights.hunk.index, "right"))
                actionButton("Accept Base", source: .base)
                    .accessibilityIdentifier(AID.Conflict.hunkUse(highlights.hunk.index, "base"))
                Spacer()
                HStack(spacing: 8) {
                    Label("Left", systemImage: "minus")
                    Label("Right", systemImage: "plus")
                }
                .jayjayFont(10, design: .monospaced)
                .foregroundStyle(.tertiary)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            Divider()
            NativeDiffView(
                diff: highlights.unified,
                showsChangeMarkers: true,
                onContentHeightChanged: { height in
                    if abs((measuredDiffHeight ?? 0) - height) > 0.5 {
                        measuredDiffHeight = height
                    }
                }
            )
            .frame(height: measuredDiffHeight ?? estimatedDiffHeight)
        }
        .background(Color(nsColor: .textBackgroundColor))
        .overlay(
            RoundedRectangle(cornerRadius: 7)
                .stroke(Color.primary.opacity(0.12), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 7))
        .contentShape(Rectangle())
        .onTapGesture(perform: onSelect)
    }

    private var estimatedDiffHeight: CGFloat {
        max(CGFloat(max(highlights.unified.lines.count, 1)) * max(18, fontSize + 5) + 24, 44)
    }

    private func actionButton(
        _ title: String,
        shortcut: String? = nil,
        source: MergeHunkSource
    ) -> some View {
        Button { onUseSource(source) } label: {
            HStack(spacing: 4) {
                Text(title)
                if let shortcut {
                    Text(shortcut)
                        .foregroundStyle(.tertiary)
                }
            }
            .jayjayFont(10)
            .foregroundStyle(.secondary)
        }
        .buttonStyle(.plain)
        .disabled(!isUnresolved)
        .help(title.replacingOccurrences(of: "Accept ", with: "Use ") + " for this conflict")
    }
}
