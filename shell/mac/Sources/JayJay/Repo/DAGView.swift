import SwiftUI
import JayJayBindings

struct DAGView: View {
    let entries: [GraphEntry]
    let selectedId: String?
    let onSelect: (String) -> Void
    var onNew: ((String) -> Void)?
    var onSquash: ((String) -> Void)?
    var onAbandon: ((String) -> Void)?
    var onCreateBookmark: ((String) -> Void)?

    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        Group {
            if entries.isEmpty {
                ContentUnavailableView("No Changes Matched", systemImage: "line.3.horizontal.decrease.circle",
                                       description: Text("Try a broader revset or refresh."))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                let layout = DAGLayout(entries: entries)
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(entries.enumerated()), id: \.element.change.changeId) { index, entry in
                            DAGRow(
                                entry: entry, layout: layout, index: index,
                                isSelected: selectedId == entry.change.changeId,
                                isLast: index == entries.count - 1,
                                colorScheme: colorScheme
                            )
                            .contentShape(Rectangle())
                            .onTapGesture { onSelect(entry.change.changeId) }
                            .contextMenu {
                                Button("New child change") { onNew?(entry.change.changeId) }
                                Button("Squash into parent") { onSquash?(entry.change.changeId) }
                                Button("Create bookmark here...") { onCreateBookmark?(entry.change.changeId) }
                                Divider()
                                Button("Abandon", role: .destructive) { onAbandon?(entry.change.changeId) }
                            }
                        }
                    }
                    .padding(.vertical, 6)
                }
                .background(
                    LinearGradient(colors: [Color.primary.opacity(colorScheme == .dark ? 0.03 : 0.015), .clear],
                                   startPoint: .topLeading, endPoint: .bottomTrailing))
            }
        }
    }
}

// MARK: - Lane layout

private let laneWidth: CGFloat = 16
private let nodeRadius: CGFloat = 4

/// Pre-computes which lane (column) each commit occupies.
struct DAGLayout {
    /// Lane index for each commit ID.
    let lanes: [String: Int]
    /// Total number of active lanes at each row.
    let activeLanesPerRow: [Int]
    /// The commit IDs in entries order.
    let commitIds: [String]

    init(entries: [GraphEntry]) {
        var lanes: [String: Int] = [:]
        var activeLanes: [String?] = [] // Each slot: commit ID occupying it, or nil if free
        var activeCounts: [Int] = []

        for entry in entries {
            let cid = entry.change.commitId

            // Find or assign a lane for this commit
            if lanes[cid] == nil {
                // Check if any edge from a previous entry points to us — reuse that lane
                if let existingLane = activeLanes.firstIndex(of: cid) {
                    lanes[cid] = existingLane
                } else {
                    // Assign first free lane
                    if let free = activeLanes.firstIndex(of: nil) {
                        activeLanes[free] = cid
                        lanes[cid] = free
                    } else {
                        lanes[cid] = activeLanes.count
                        activeLanes.append(cid)
                    }
                }
            }

            let myLane = lanes[cid]!

            // Free my lane
            activeLanes[myLane] = nil

            // Reserve lanes for my edges (children point to parents)
            for edge in entry.edges {
                if edge.edgeType == .missing { continue }
                let target = edge.target
                if lanes[target] == nil {
                    // Assign target to my lane if free, else new lane
                    if activeLanes[myLane] == nil {
                        activeLanes[myLane] = target
                        lanes[target] = myLane
                    } else if let free = activeLanes.firstIndex(of: nil) {
                        activeLanes[free] = target
                        lanes[target] = free
                    } else {
                        lanes[target] = activeLanes.count
                        activeLanes.append(target)
                    }
                }
            }

            activeCounts.append(activeLanes.count)
        }

        self.lanes = lanes
        self.activeLanesPerRow = activeCounts
        self.commitIds = entries.map(\.change.commitId)
    }

    func lane(for commitId: String) -> Int {
        lanes[commitId] ?? 0
    }

    func maxLanes() -> Int {
        activeLanesPerRow.max() ?? 1
    }
}

// MARK: - Row

struct DAGRow: View {
    let entry: GraphEntry
    let layout: DAGLayout
    let index: Int
    let isSelected: Bool
    let isLast: Bool
    let colorScheme: ColorScheme

    private var change: ChangeInfo { entry.change }

    var body: some View {
        HStack(alignment: .top, spacing: 0) {
            graphColumn
                .frame(width: CGFloat(max(layout.maxLanes(), 1)) * laneWidth + 8)

            VStack(alignment: .leading, spacing: 5) {
                HStack(spacing: 6) {
                    Text(shortId(change.changeId))
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(change.isWorkingCopy ? Color.accentColor : .secondary)
                    if change.isWorkingCopy { tag("@", tint: .accentColor.opacity(0.18)) }
                    if change.hasConflict { tag("conflict", tint: .red.opacity(0.18)) }
                    ForEach(change.bookmarks, id: \.self) { tag($0, tint: .primary.opacity(0.08)) }
                }

                if !change.description.isEmpty {
                    Text(change.description.components(separatedBy: "\n").first ?? "")
                        .jayjayFont(13, weight: .medium).lineLimit(1)
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
        .background(isSelected ? AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10)) : AnyShapeStyle(.clear))
        .overlay(alignment: .leading) {
            if isSelected {
                RoundedRectangle(cornerRadius: 2, style: .continuous).fill(Color.accentColor).frame(width: 3)
            }
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
                for (cid, lane) in layout.lanes {
                    let laneX = CGFloat(lane) * laneWidth + laneWidth / 2 + 4
                    // Draw a line if this lane has a commit that spans across this row
                    if cid != change.commitId && isLaneActiveAtRow(cid: cid, lane: lane) {
                        let path = Path { p in p.move(to: CGPoint(x: laneX, y: 0)); p.addLine(to: CGPoint(x: laneX, y: height)) }
                        ctx.stroke(path, with: .color(.secondary.opacity(0.2)), style: StrokeStyle(lineWidth: 1))
                    }
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
                            p.addQuadCurve(to: CGPoint(x: targetX, y: height),
                                           control: CGPoint(x: targetX, y: midY))
                        }
                    }
                    let style = edge.edgeType == .indirect
                        ? StrokeStyle(lineWidth: 1, dash: [3, 3])
                        : StrokeStyle(lineWidth: 1)
                    ctx.stroke(path, with: .color(.secondary.opacity(0.3)), style: style)
                }

                // Draw node
                let nodeRect = CGRect(x: myX - nodeRadius, y: nodeY - nodeRadius,
                                      width: nodeRadius * 2, height: nodeRadius * 2)
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

    /// Check if a commit's lane should show a continuation line at this row.
    private func isLaneActiveAtRow(cid: String, lane: Int) -> Bool {
        // A lane is active at this row if the commit appears BEFORE this row
        // and its parent(s) appear AFTER this row
        guard let cidIndex = layout.commitIds.firstIndex(of: cid) else { return false }
        // The commit must be above us (earlier in the list)
        if cidIndex >= index { return false }
        // Check if any entry between cidIndex and the end has an edge pointing to something below us
        // Simplified: the lane is active if the commit is above and not yet resolved
        return cidIndex < index
    }

    private var selectionBackground: some ShapeStyle {
        isSelected ? AnyShapeStyle(Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10)) : AnyShapeStyle(.clear)
    }

    @ViewBuilder
    private func tag(_ title: String, tint: Color) -> some View {
        Text(title).jayjayFont(9, weight: .semibold)
            .padding(.horizontal, 5).padding(.vertical, 2)
            .background(tint, in: Capsule())
    }

    private func shortId(_ id: String) -> String { String(id.prefix(12)) }
}
