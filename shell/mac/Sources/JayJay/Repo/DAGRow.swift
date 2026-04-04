import AppKit
import JayJayCore
import SwiftUI

struct DAGRow: View {
    let entry: GraphEntry
    let layout: DAGLayout
    let index: Int
    let isSelected: Bool
    let isCompareSource: Bool
    var isContextTarget: Bool = false
    let isLast: Bool
    let colorScheme: ColorScheme
    var onMoveBookmarkForward: ((String) -> Void)?
    var onPushBookmark: ((String) -> Void)?

    private var change: ChangeInfo {
        entry.change
    }

    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            graphColumn
                .frame(width: min(160, CGFloat(max(layout.maxLanes(), 1)) * laneWidth + 8))

            VStack(alignment: .leading, spacing: 5) {
                HStack(spacing: 4) {
                    Text(shortId(change.changeId))
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(change.isWorkingCopy ? Color.accentColor : .secondary)
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

                if !change.description.isEmpty {
                    let firstLine = change.description.components(separatedBy: "\n").first ?? ""
                    Text(firstLine)
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
        .padding(.vertical, 8)
        .padding(.leading, 4)
        .background(rowBackground)
        .overlay(alignment: .leading) {
            if isSelected {
                RoundedRectangle(cornerRadius: 2, style: .continuous).fill(Color.accentColor).frame(width: 3)
            } else if isCompareSource {
                RoundedRectangle(cornerRadius: 2, style: .continuous).fill(Color.orange).frame(width: 3)
            } else if isContextTarget {
                RoundedRectangle(cornerRadius: 2, style: .continuous).fill(Color.secondary).frame(width: 3)
            }
        }
    }

    private var rowBackground: some ShapeStyle {
        if isSelected {
            AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10))
        } else if isCompareSource {
            AnyShapeStyle(Color.orange.opacity(colorScheme == .dark ? 0.15 : 0.08))
        } else if isContextTarget {
            AnyShapeStyle(Color.secondary.opacity(colorScheme == .dark ? 0.12 : 0.06))
        } else {
            AnyShapeStyle(.clear)
        }
    }

    private var graphColumn: some View {
        GeometryReader { geo in
            let myLane = layout.lane(for: change.commitId)
            let myX = CGFloat(myLane) * laneWidth + laneWidth / 2 + 4
            let nodeY: CGFloat = 12
            let height = geo.size.height

            Canvas { ctx, _ in
                // Draw vertical continuation lines for all active lanes
                for lane in layout.activeLaneIndices(at: index) where lane != myLane {
                    let laneX = CGFloat(lane) * laneWidth + laneWidth / 2 + 4
                    let path = Path { p in
                        p.move(to: CGPoint(x: laneX, y: 0))
                        p.addLine(to: CGPoint(x: laneX, y: height))
                    }
                    ctx.stroke(path, with: .color(.secondary.opacity(0.2)), style: StrokeStyle(lineWidth: 1))
                }

                // Draw edges from this node to its parents
                for edge in entry.edges {
                    if edge.edgeType == .missing { continue }
                    let targetLane = layout.lane(for: edge.target)
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
}
