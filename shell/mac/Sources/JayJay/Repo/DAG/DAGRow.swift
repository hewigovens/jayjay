import AppKit
import JayJayCore
import SwiftUI

private struct DAGRefsRowBoundsPreferenceKey: PreferenceKey {
    static let defaultValue: Anchor<CGRect>? = nil

    static func reduce(value: inout Anchor<CGRect>?, nextValue: () -> Anchor<CGRect>?) {
        value = value ?? nextValue()
    }
}

struct DAGRow: View {
    @Environment(\.colorScheme) var colorScheme
    @Environment(\.jayjayFontSize) var baseFontSize
    @Environment(\.jayjayFontFamily) var fontFamily
    let viewModel: DAGRowViewModel
    var actions: (any DAGActions & BookmarkActions)?
    var onRequest: ((DAGRequest) -> Void)?
    var prHostName: String?
    var conflictedBookmarkNames: Set<String> = []
    var workspacesByName: [String: WorkspaceInfo] = [:]
    var onBookmarkDragChanged: ((String, String, DragGesture.Value) -> Void)?
    var onBookmarkDragEnded: ((String, DragGesture.Value) -> Void)?
    @State private var isContextTarget = false

    /// Non-private: read by the DAGRow+Refs extension.
    var change: ChangeInfo {
        viewModel.change
    }

    var body: some View {
        let viewModel = viewModel.contextTargeted(isContextTarget)
        rowBody(viewModel)
            .modifier(DAGRowWiggle(isArmed: viewModel.isRebaseArmed))
            .contentShape(Rectangle())
            .onHover { isContextTarget = $0 }
    }

    private func summaryColumn(_ viewModel: DAGRowViewModel) -> some View {
        VStack(alignment: .leading, spacing: 5) {
            refsRow
                .lineLimit(1)
                .anchorPreference(key: DAGRefsRowBoundsPreferenceKey.self, value: .bounds) { $0 }

            if let descriptionLine = viewModel.descriptionLine {
                Text(descriptionLine)
                    .jayjayFont(13, weight: .medium).lineLimit(2)
                    .help(change.description)
            } else {
                Text("(no description)").jayjayFont(13).foregroundStyle(.tertiary)
            }

            HStack(spacing: 6) {
                Text(change.commitId.highlighted(
                    scheme: colorScheme,
                    font: fontFamily.identifierFont(baseSize: baseFontSize),
                    prefixColor: AppColors.commitIdPrefix(colorScheme)
                ))
                .fixedSize(horizontal: true, vertical: false)
                .help("Commit: \(change.commitId.id)")
                .accessibilityLabel("Commit \(change.commitId.compact)")
                CommitAvatar(email: change.author.email, size: 14)
                Text(change.author.name)
                Text(Date.relativeLabel(millis: change.author.timestampMillis)).foregroundStyle(.secondary)
            }
            .jayjayFont(11).lineLimit(1).truncationMode(.tail).foregroundStyle(.secondary)
        }
        .padding(.vertical, dagRowVerticalPadding)
        .padding(.trailing, 10)
    }

    private func rowBody(_ viewModel: DAGRowViewModel) -> some View {
        HStack(alignment: .top, spacing: 0) {
            Color.clear.frame(width: viewModel.graphWidth)
            summaryColumn(viewModel)
            Spacer(minLength: 0)
        }
        .overlayPreferenceValue(DAGRefsRowBoundsPreferenceKey.self) { refsRowBounds in
            GeometryReader { geo in
                DAGGraphColumn(
                    viewModel: viewModel,
                    nodeCenterY: refsRowBounds.map { geo[$0].midY } ?? dagFallbackNodeCenterY
                )
                .frame(width: viewModel.graphWidth)
            }
            .allowsHitTesting(false)
        }
        .padding(.leading, dagRowLeadingPadding)
        .background(viewModel.rowBackground)
        .scaleEffect(viewModel.scale)
        .opacity(viewModel.opacity)
        .overlay(alignment: .leading) {
            if let accent = viewModel.leadingAccentColor {
                RoundedRectangle(cornerRadius: 2, style: .continuous)
                    .fill(accent)
                    .frame(width: 3)
            }
        }
        .overlay {
            switch viewModel.outlineState {
                case .hoverTarget?:
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(Color.accentColor, lineWidth: 2)
                        .padding(.vertical, 2)
                case .armed?:
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(
                            Color.accentColor.opacity(0.7),
                            style: StrokeStyle(lineWidth: 1.5, dash: [5, 4])
                        )
                        .padding(.vertical, 2)
                case nil:
                    EmptyView()
            }
        }
        .overlay(alignment: .trailing) {
            if let dragTargetText = viewModel.dragTargetText {
                dragTargetBubble(dragTargetText)
                    .padding(.trailing, 10)
            }
        }
    }

    private func dragTargetBubble(_ text: String) -> some View {
        HStack(spacing: 6) {
            Text(text)
                .jayjayFont(10, weight: .medium)
                .lineLimit(1)
            if viewModel.showsReturnHint {
                hintChip("return")
            }
            hintChip("esc")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            viewModel.isRebaseHoverTarget ? Color.accentColor.opacity(0.14) : Color.clear,
            in: Capsule()
        )
        .glassEffect(in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.accentColor.opacity(viewModel.isRebaseHoverTarget ? 0.5 : 0.2), lineWidth: 1)
        )
    }

    private func hintChip(_ text: String) -> some View {
        Text(text.uppercased())
            .jayjayFont(8, weight: .semibold, design: .monospaced)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 4)
            .padding(.vertical, 2)
            .background(Color.primary.opacity(0.06), in: Capsule())
    }
}

/// A modifier rather than a TimelineView branch: swapping the row's view when it arms cancels the drag in flight.
private struct DAGRowWiggle: ViewModifier {
    let isArmed: Bool
    @State private var tilted = false

    func body(content: Content) -> some View {
        content
            .rotationEffect(.degrees(isArmed ? (tilted ? 1.1 : -1.1) : 0))
            .animation(isArmed ? .easeInOut(duration: 0.09).repeatForever(autoreverses: true) : .default, value: tilted)
            .onChange(of: isArmed) { _, armed in tilted = armed }
    }
}
