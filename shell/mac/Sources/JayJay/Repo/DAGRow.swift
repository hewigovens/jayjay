import AppKit
import JayJayCore
import SwiftUI

struct DAGRow: View {
    let viewModel: DAGRowViewModel
    var onMoveBookmarkForward: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?

    private var change: ChangeInfo {
        viewModel.change
    }

    var body: some View {
        if viewModel.isRebaseArmed {
            TimelineView(.animation) { timeline in
                rowBody(wiggleAngle: viewModel.wiggleAngle(at: timeline.date))
            }
        } else {
            rowBody(wiggleAngle: 0)
        }
    }

    private func rowBody(wiggleAngle: Double) -> some View {
        HStack(alignment: .top, spacing: 0) {
            graphColumn
                .frame(width: viewModel.graphWidth)

            VStack(alignment: .leading, spacing: 5) {
                HStack(spacing: 4) {
                    Text(shortId(change.changeId))
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(viewModel.changeIdColor)
                        .lineLimit(1)
                    if change.isWorkingCopy { tag("@", tint: .accentColor.opacity(0.18)) }
                    if change.hasConflict { tag("conflict", tint: .red.opacity(0.18)) }
                    if change.isDivergent { tag("divergent", tint: .orange.opacity(0.18)) }
                    ForEach(change.bookmarks.prefix(3), id: \.self) {
                        bookmarkTag($0)
                    }
                    if change.bookmarks.count > 3 {
                        tag("+\(change.bookmarks.count - 3)", tint: .primary.opacity(0.05))
                            .help(change.bookmarks.joined(separator: ", "))
                    }
                }
                .lineLimit(1)

                if let descriptionLine = viewModel.descriptionLine {
                    Text(descriptionLine)
                        .jayjayFont(13, weight: .medium).lineLimit(1)
                        .help(change.description)
                } else {
                    Text("(no description)").jayjayFont(13).foregroundStyle(.tertiary)
                }

                HStack(spacing: 6) {
                    Text(change.author)
                    Text(shortId(change.commitId)).foregroundStyle(.secondary)
                }
                .jayjayFont(10, design: .monospaced).lineLimit(1).truncationMode(.tail).foregroundStyle(.secondary)
            }
            .padding(.trailing, 10)
            Spacer(minLength: 0)
        }
        .padding(.vertical, dagRowVerticalPadding)
        .padding(.leading, dagRowLeadingPadding)
        .background(viewModel.rowBackground)
        .rotationEffect(.degrees(wiggleAngle))
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

    private var graphColumn: some View {
        GeometryReader { geo in
            let myLane = viewModel.layout.lane(for: change.commitId)
            let myX = CGFloat(myLane) * laneWidth + laneWidth / 2 + 4
            let nodeY = dagNodeCenterY
            let height = geo.size.height

            Canvas { ctx, _ in
                // Draw vertical continuation lines for all active lanes
                for lane in viewModel.layout.activeLaneIndices(at: viewModel.index) where lane != myLane {
                    let laneX = CGFloat(lane) * laneWidth + laneWidth / 2 + 4
                    let path = Path { p in
                        p.move(to: CGPoint(x: laneX, y: 0))
                        p.addLine(to: CGPoint(x: laneX, y: height))
                    }
                    ctx.stroke(path, with: .color(.secondary.opacity(0.2)), style: StrokeStyle(lineWidth: 1))
                }

                // Draw edges from this node to its parents
                for edge in viewModel.entry.edges {
                    if edge.edgeType == .missing { continue }
                    let targetLane = viewModel.layout.lane(for: edge.target)
                    let targetX = CGFloat(targetLane) * laneWidth + laneWidth / 2 + 4

                    let path = Path { p in
                        p.move(to: CGPoint(x: myX, y: nodeY + nodeRadius))
                        if targetLane == myLane {
                            // Straight down
                            p.addLine(to: CGPoint(x: myX, y: height))
                        } else {
                            // Curve to target lane
                            let midY = nodeY + nodeRadius + (height - nodeY - nodeRadius) * 0.4
                            p.addLine(to: CGPoint(x: myX, y: midY))
                            p.addQuadCurve(
                                to: CGPoint(x: targetX, y: height),
                                control: CGPoint(x: targetX, y: midY)
                            )
                        }
                    }
                    let style = edge.edgeType == .indirect
                        ? StrokeStyle(lineWidth: 1, dash: [3, 3])
                        : StrokeStyle(lineWidth: 1)
                    ctx.stroke(path, with: .color(.secondary.opacity(0.3)), style: style)
                }

                // Draw node
                let nodeRect = CGRect(
                    x: myX - nodeRadius,
                    y: nodeY - nodeRadius,
                    width: nodeRadius * 2,
                    height: nodeRadius * 2
                )
                let nodePath = Path(ellipseIn: nodeRect)
                if change.isWorkingCopy {
                    ctx.fill(nodePath, with: .color(.accentColor))
                } else if change.isEmpty {
                    ctx.stroke(nodePath, with: .color(.secondary.opacity(0.5)), style: StrokeStyle(lineWidth: 1.5))
                } else if change.hasConflict {
                    ctx.fill(nodePath, with: .color(.red))
                } else {
                    ctx.fill(nodePath, with: .color(.secondary.opacity(0.5)))
                }

                if viewModel.isRebaseCandidate {
                    ctx.stroke(
                        nodePath,
                        with: .color(.accentColor.opacity(viewModel.isRebaseHoverTarget ? 1 : 0.55)),
                        style: StrokeStyle(lineWidth: viewModel.isRebaseHoverTarget ? 2.5 : 1.4)
                    )
                    if viewModel.isRebaseHoverTarget {
                        let ringRect = nodeRect.insetBy(dx: -4, dy: -4)
                        ctx.stroke(
                            Path(ellipseIn: ringRect),
                            with: .color(.accentColor.opacity(0.45)),
                            style: StrokeStyle(lineWidth: 2)
                        )
                    }
                } else if viewModel.isRebaseSource {
                    ctx.stroke(
                        nodePath,
                        with: .color(.accentColor.opacity(0.75)),
                        style: StrokeStyle(lineWidth: 2)
                    )
                    if viewModel.isRebaseArmed {
                        let ringRect = nodeRect.insetBy(dx: -3, dy: -3)
                        ctx.stroke(
                            Path(ellipseIn: ringRect),
                            with: .color(.accentColor.opacity(0.35)),
                            style: StrokeStyle(lineWidth: 1.5, dash: [3, 3])
                        )
                    }
                }
            }
        }
    }

    private func tag(_ title: String, tint: Color) -> some View {
        Text(title).jayjayFont(9, weight: .semibold)
            .lineLimit(1)
            .truncationMode(.middle)
            .frame(maxWidth: 120)
            .padding(.horizontal, 5).padding(.vertical, 2)
            .background(tint, in: Capsule())
    }

    private func bookmarkTag(_ name: String) -> some View {
        tag(name, tint: .primary.opacity(0.08))
            .help(name)
            .contextMenu {
                Button("Move to @-") {
                    onMoveBookmarkForward?(name)
                }
                Button("Push") {
                    onPushBookmark?(name)
                }
                Divider()
                Button("Copy Bookmark Name") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(name, forType: .string)
                }
            }
    }

    private func shortId(_ id: String) -> String {
        String(id.prefix(12))
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
        .background(.regularMaterial, in: Capsule())
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
