import JayJayCore
import JayJayDiffUI
import SwiftUI

struct MergeHunkList: View {
    let highlights: [MergeHunkHighlights]
    let result: String
    @Binding var selectedHunk: UInt32?
    let scroll: MergeScrollCoordinator
    let onUseSource: (MergeEditorHunk, MergeHunkSource) -> Void

    @State private var hunkFrames: [UInt32: CGRect] = [:]
    @State private var viewport: CGRect = .zero
    @State private var isScrolling = false
    @State private var revealTarget: UInt32?

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 12) {
                    ForEach(highlights) { item in
                        let unresolved = mergeHunkIsUnresolved(result: result, hunk: item.hunk)
                        MergeHunkCard(
                            highlights: item,
                            isUnresolved: unresolved,
                            isSelected: selectedHunk == item.id,
                            onSelect: { selectedHunk = item.id },
                            onUseSource: { source in
                                selectedHunk = item.id
                                onUseSource(item.hunk, source)
                            }
                        )
                        .id(item.id)
                        .background {
                            GeometryReader { geometry in
                                Color.clear.preference(
                                    key: MergeHunkFrames.self,
                                    value: [item.id: geometry.frame(in: .named("merge-hunk-content"))]
                                )
                            }
                        }
                    }
                }
                .padding(12)
                .coordinateSpace(name: "merge-hunk-content")
            }
            .onPreferenceChange(MergeHunkFrames.self) { frames in
                hunkFrames = frames
                // Native diff heights settle after the initial reveal; retain its target until the user scrolls.
                if !isScrolling {
                    reveal(using: proxy)
                }
                followVisibleHunk()
            }
            .onScrollGeometryChange(for: CGRect.self) { geometry in
                geometry.visibleRect
            } action: { old, visible in
                viewport = visible
                if old.size != visible.size, !isScrolling {
                    reveal(using: proxy)
                }
                followVisibleHunk()
            }
            .onScrollPhaseChange { _, phase in
                isScrolling = phase != .idle && phase != .animating
                if isScrolling {
                    revealTarget = nil
                }
                followVisibleHunk()
            }
            .onChange(of: scroll.visibleHunk) { _, hunk in
                if !isScrolling, let hunk {
                    revealTarget = hunk
                    reveal(using: proxy)
                }
            }
            .onChange(of: selectedHunk, initial: true) { _, hunk in
                if let hunk {
                    revealTarget = hunk
                    reveal(using: proxy)
                }
            }
        }
        .accessibilityIdentifier(AID.Conflict.editorHunkList)
        .background(Color(nsColor: .textBackgroundColor))
    }

    private func reveal(using proxy: ScrollViewProxy) {
        guard let revealTarget else { return }
        let fits = hunkFrames[revealTarget].map { $0.height <= viewport.height } ?? false
        proxy.scrollTo(revealTarget, anchor: fits ? .center : .top)
    }

    private func followVisibleHunk() {
        guard isScrolling,
              let hunk = hunkFrames.min(by: { abs($0.value.midY - viewport.midY) < abs($1.value.midY - viewport.midY) })?.key,
              hunk != scroll.visibleHunk else { return }
        scroll.didScroll(hunk: hunk)
    }
}

private struct MergeHunkFrames: PreferenceKey {
    static let defaultValue: [UInt32: CGRect] = [:]

    static func reduce(value: inout [UInt32: CGRect], nextValue: () -> [UInt32: CGRect]) {
        value.merge(nextValue(), uniquingKeysWith: { _, new in new })
    }
}

private struct MergeHunkCard: View {
    let highlights: MergeHunkHighlights
    let isUnresolved: Bool
    let isSelected: Bool
    let onSelect: () -> Void
    let onUseSource: (MergeHunkSource) -> Void

    @State private var measuredDiffHeight: CGFloat?
    @Environment(\.jayjayFontSize) private var fontSize

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Text("Conflict \(highlights.hunk.index + 1)")
                    .jayjayFont(11, weight: .semibold)
                    .accessibilityIdentifier(AID.Conflict.hunkCard(highlights.id))
                    .accessibilityValue(isSelected ? "Selected" : "")
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
                .stroke(isSelected ? Color.accentColor : Color.primary.opacity(0.12), lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 7))
        .contentShape(Rectangle())
        .onTapGesture(perform: onSelect)
        .accessibilityElement(children: .contain)
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
