import AppKit
import JayJayCore
import SwiftUI

struct DAGRow: View {
    let viewModel: DAGRowViewModel
    var prHostName: String?
    var onMoveBookmarkForward: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?
    var onOpenPRForBookmark: ((String) -> Void)?

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
                    if change.isDivergent { tag("divergent", tint: FileStatusColors.modified.opacity(0.18)) }
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
                    Text(change.author.name)
                    Text(shortId(change.commitId)).foregroundStyle(.secondary)
                }
                .jayjayFont(10, design: .monospaced).lineLimit(1).truncationMode(.tail).foregroundStyle(.secondary)
            }
            .padding(.vertical, dagRowVerticalPadding)
            .padding(.trailing, 10)
            Spacer(minLength: 0)
        }
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
                let lineColor = Color.secondary.opacity(0.2)
                let edgeColor = Color.secondary.opacity(0.3)

                for lane in viewModel.layout.activeLaneIndices(at: viewModel.index) where lane != myLane {
                    let laneX = CGFloat(lane) * laneWidth + laneWidth / 2 + 4
                    let path = Path { p in
                        p.move(to: CGPoint(x: laneX, y: 0))
                        p.addLine(to: CGPoint(x: laneX, y: height))
                    }
                    ctx.stroke(path, with: .color(lineColor), style: StrokeStyle(lineWidth: 1))
                }

                // Top stub: connect down from the row above when the lane continues.
                if viewModel.index > 0 {
                    let prevActive = viewModel.layout.activeLaneIndices(at: viewModel.index - 1)
                    if prevActive.contains(myLane) {
                        let path = Path { p in
                            p.move(to: CGPoint(x: myX, y: 0))
                            p.addLine(to: CGPoint(x: myX, y: nodeY - nodeRadius))
                        }
                        ctx.stroke(path, with: .color(lineColor), style: StrokeStyle(lineWidth: 1))
                    }
                }

                // Bottom stub for non-tail nodes on a forking lane.
                let hasSameLaneParent = viewModel.entry.edges.contains { edge in
                    edge.edgeType != .missing && viewModel.layout.lane(for: edge.target) == myLane
                }
                if !hasSameLaneParent {
                    let nextActive = viewModel.layout.activeLaneIndices(at: viewModel.index + 1)
                    if nextActive.contains(myLane) {
                        let path = Path { p in
                            p.move(to: CGPoint(x: myX, y: nodeY + nodeRadius))
                            p.addLine(to: CGPoint(x: myX, y: height))
                        }
                        ctx.stroke(path, with: .color(lineColor), style: StrokeStyle(lineWidth: 1))
                    }
                }

                for edge in viewModel.entry.edges {
                    if edge.edgeType == .missing { continue }
                    let targetLane = viewModel.layout.lane(for: edge.target)
                    let targetX = CGFloat(targetLane) * laneWidth + laneWidth / 2 + 4

                    let path = Path { p in
                        p.move(to: CGPoint(x: myX, y: nodeY + nodeRadius))
                        if targetLane == myLane {
                            p.addLine(to: CGPoint(x: myX, y: height))
                        } else {
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
                    ctx.stroke(path, with: .color(edgeColor), style: style)
                }

                let style = DAGNodeStyle.resolve(change: change)
                let nodeRect = CGRect(
                    x: myX - style.radius,
                    y: nodeY - style.radius,
                    width: style.radius * 2,
                    height: style.radius * 2
                )
                let nodePath = style.path(in: nodeRect)
                switch style.fill {
                    case let .filled(color):
                        ctx.fill(nodePath, with: .color(color))
                    case let .outlined(color, lineWidth):
                        ctx.stroke(nodePath, with: .color(color), style: StrokeStyle(lineWidth: lineWidth))
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
                            style.path(in: ringRect),
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
                            style.path(in: ringRect),
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
                if !isTrunkBookmark(name) {
                    Button(pullRequestLabel) {
                        onOpenPRForBookmark?(name)
                    }
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

    private var pullRequestLabel: String {
        if let prHostName {
            return "Pull Request on \(prHostName)"
        }
        return "Pull Request"
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
