import AppKit
import JayJayCore
import SwiftUI

struct OverviewCanvas: View {
    let lanes: [OverviewLane]
    let laneIds: [String]
    let groups: [OverviewGroup]
    let trunkName: String
    @Binding var selectedLaneId: String?
    @Binding var selectedChangeId: String?
    let onShowInGraph: (OverviewLane, OverviewChange?) -> Void
    let actions: OverviewLaneActions

    @Environment(\.colorScheme) private var colorScheme

    private typealias Geo = OverviewGeometry

    var body: some View {
        let placement = OverviewPlacement(lanes: lanes, groups: groups)
        ZStack(alignment: .topLeading) {
            Canvas { context, _ in
                drawLines(in: &context, placement: placement)
            }
            .frame(width: placement.size.width, height: placement.size.height)
            .allowsHitTesting(false)

            ForEach(Array(placement.bands.enumerated()), id: \.offset) { _, band in
                OverviewTrunkLabel(base: band.base, trunkName: trunkName)
                    .frame(width: Geo.trunkLabelWidth, height: Geo.rowHeight, alignment: .leading)
                    .position(x: Geo.spineX + 12 + Geo.trunkLabelWidth / 2, y: band.y - Geo.rowHeight / 2 - 2)
            }

            ForEach(placement.lanes) { placed in
                let lane = lanes[placed.laneIndex]
                let laneId = laneIds[placed.laneIndex]
                Button {
                    selectedLaneId = laneId
                    selectedChangeId = nil
                } label: {
                    OverviewLaneCard(lane: lane, trunkName: trunkName, isSelected: selectedLaneId == laneId)
                        .frame(width: Geo.columnWidth, height: Geo.cardHeight)
                }
                .buttonStyle(.plain)
                .simultaneousGesture(TapGesture(count: 2).onEnded { onShowInGraph(lane, nil) })
                .contextMenu { laneMenu(lane, id: laneId) }
                .position(x: placed.x + Geo.columnWidth / 2, y: placed.cardCenterY)
                .accessibilityIdentifier(AID.Overview.lane(lane.head.changeId.prefix))

                ForEach(Array(lane.changes.enumerated()), id: \.element.commitId.id) { offset, change in
                    Button {
                        selectedLaneId = laneId
                        selectedChangeId = change.commitId.id
                    } label: {
                        OverviewChangeRow(change: change, isChosen: selectedChangeId == change.commitId.id)
                            .frame(width: Geo.columnWidth, height: Geo.rowHeight)
                    }
                    .buttonStyle(.plain)
                    .simultaneousGesture(TapGesture(count: 2).onEnded { onShowInGraph(lane, change) })
                    .contextMenu { changeMenu(lane, change) }
                    .position(x: placed.x + Geo.columnWidth / 2, y: placed.nodeY(offset))
                    .help(change.title)
                }
            }
        }
        .frame(width: placement.size.width, height: placement.size.height, alignment: .topLeading)
        .background(OverviewScrollRevealer(rect: selectionRect(in: placement)))
    }

    private func selectionRect(in placement: OverviewPlacement) -> CGRect? {
        guard let placed = placement.lanes.first(where: { laneIds[$0.laneIndex] == selectedLaneId }) else { return nil }
        let changes = lanes[placed.laneIndex].changes
        let rect = if let offset = changes.firstIndex(where: { $0.commitId.id == selectedChangeId }) {
            CGRect(x: placed.x, y: placed.nodeY(offset) - Geo.rowHeight / 2, width: Geo.columnWidth, height: Geo.rowHeight)
        } else {
            CGRect(x: placed.x, y: placed.cardCenterY - Geo.cardHeight / 2, width: Geo.columnWidth, height: Geo.cardHeight)
        }
        return rect.insetBy(dx: -Geo.columnGap, dy: -Geo.cardGap)
    }

    @ViewBuilder
    private func laneMenu(_ lane: OverviewLane, id: String) -> some View {
        Button("Show in Graph") { onShowInGraph(lane, nil) }
        ForEach(lane.workspaces, id: \.name) { workspace in
            if let info = actions.workspaceInfo(workspace.name), info.isPathResolved {
                if !workspace.isCurrent {
                    Button("Open \(workspace.name) in Window") { actions.openWorkspace(info) }
                }
                Button("Reveal \(workspace.name) in Finder") { RepositoryActions.showInFinder(repoPath: info.path) }
            }
        }
        Divider()
        if lane.isBehindTrunk {
            Button("Rebase Lane onto \(trunkName)") { actions.rebaseOntoTrunk(lane) }
        }
        Button("Abandon Lane…", role: .destructive) { actions.abandon(.lane(lane, id: id)) }
        ForEach(lane.workspaces.filter { !$0.isCurrent }, id: \.name) { workspace in
            if let info = actions.workspaceInfo(workspace.name) {
                Divider()
                Button("Forget Workspace \(workspace.name)") { actions.forgetWorkspace(info, false) }
                if info.isPathResolved {
                    Button("Forget & Delete \(workspace.name) from Disk…", role: .destructive) { actions.forgetWorkspace(info, true) }
                }
            }
        }
        Divider()
        Button("Copy Head Change ID") { copy(lane.head.changeId.id) }
    }

    @ViewBuilder
    private func changeMenu(_ lane: OverviewLane, _ change: OverviewChange) -> some View {
        Button("Show in Graph") { onShowInGraph(lane, change) }
        Divider()
        Button("Abandon Change…", role: .destructive) { actions.abandon(.change(change)) }
        Divider()
        Button("Copy Change ID") { copy(change.changeId.id) }
    }

    private func copy(_ text: String) {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(text, forType: .string)
    }

    private func drawLines(in context: inout GraphicsContext, placement: OverviewPlacement) {
        let line = AppColors.graphLine(colorScheme)
        let muted = line.opacity(0.55)

        for placed in placement.lanes {
            let x = placed.x + Geo.nodeInset
            var stem = Path()
            stem.move(to: CGPoint(x: x, y: placed.cardCenterY + Geo.cardHeight / 2))
            stem.addLine(to: CGPoint(x: x, y: placed.bandY))
            context.stroke(stem, with: .color(muted), style: StrokeStyle(lineWidth: 1.2))
        }

        for band in placement.bands {
            var path = Path()
            path.move(to: CGPoint(x: Geo.spineX + 6, y: band.y))
            path.addLine(to: CGPoint(x: band.lastX, y: band.y))
            context.stroke(path, with: .color(muted), style: StrokeStyle(lineWidth: 1.2))

            var diamond = Path()
            diamond.move(to: CGPoint(x: Geo.spineX, y: band.y - 6))
            diamond.addLine(to: CGPoint(x: Geo.spineX + 6, y: band.y))
            diamond.addLine(to: CGPoint(x: Geo.spineX, y: band.y + 6))
            diamond.addLine(to: CGPoint(x: Geo.spineX - 6, y: band.y))
            diamond.closeSubpath()
            context.fill(diamond, with: .color(Color(nsColor: .windowBackgroundColor)))
            context.stroke(diamond, with: .color(band.base.kind == .olderTrunk ? .orange : line), style: StrokeStyle(lineWidth: 1.8))
        }

        if let first = placement.bands.first, let last = placement.bands.last, first.y != last.y {
            var spine = Path()
            spine.move(to: CGPoint(x: Geo.spineX, y: first.y + 6))
            spine.addLine(to: CGPoint(x: Geo.spineX, y: last.y - 6))
            context.stroke(spine, with: .color(muted), style: StrokeStyle(lineWidth: 1.2, dash: [3, 4]))
        }
    }
}

struct OverviewLaneActions {
    let workspaceInfo: (String) -> WorkspaceInfo?
    let openWorkspace: (WorkspaceInfo) -> Void
    let rebaseOntoTrunk: (OverviewLane) -> Void
    let abandon: (OverviewAbandonRequest) -> Void
    let forgetWorkspace: (WorkspaceInfo, _ deleteFromDisk: Bool) -> Void
}

enum OverviewGeometry {
    static let spineX: CGFloat = 20
    static let trunkLabelWidth: CGFloat = 170
    static let gutterWidth: CGFloat = 200
    static let columnWidth: CGFloat = 232
    static let columnGap: CGFloat = 14
    static let cardHeight: CGFloat = 86
    static let cardGap: CGFloat = 10
    static let rowHeight: CGFloat = 24
    static let nodeInset: CGFloat = 12
    static let topPadding: CGFloat = 14
    static let bandSpacing: CGFloat = 40
}

struct OverviewPlacement {
    struct Band {
        let base: OverviewBase
        let y: CGFloat
        let lastX: CGFloat
    }

    struct Lane: Identifiable {
        let laneIndex: Int
        let x: CGFloat
        let bandY: CGFloat
        let count: Int

        var id: Int {
            laneIndex
        }

        func nodeY(_ offset: Int) -> CGFloat {
            bandY - CGFloat(count - offset) * OverviewGeometry.rowHeight
        }

        var cardCenterY: CGFloat {
            nodeY(0) - OverviewGeometry.rowHeight / 2 - OverviewGeometry.cardGap - OverviewGeometry.cardHeight / 2
        }
    }

    let bands: [Band]
    let lanes: [Lane]
    let size: CGSize

    init(lanes allLanes: [OverviewLane], groups: [OverviewGroup]) {
        typealias Geo = OverviewGeometry
        var bands: [Band] = []
        var lanes: [Lane] = []
        var widestGroup = 0
        var previousBandY: CGFloat?
        for group in groups {
            let counts = group.lanes.map { allLanes[Int($0)].changes.count }
            let top = previousBandY.map { $0 + Geo.bandSpacing } ?? Geo.topPadding
            let bandY = top + Geo.cardHeight + Geo.cardGap + Geo.rowHeight * (CGFloat(counts.max() ?? 0) + 0.5)
            previousBandY = bandY
            var lastX = Geo.spineX
            for (column, laneIndex) in group.lanes.enumerated() {
                let x = Geo.gutterWidth + CGFloat(column) * (Geo.columnWidth + Geo.columnGap)
                lanes.append(Lane(laneIndex: Int(laneIndex), x: x, bandY: bandY, count: counts[column]))
                lastX = x + Geo.nodeInset
            }
            widestGroup = max(widestGroup, group.lanes.count)
            bands.append(Band(base: group.base, y: bandY, lastX: lastX))
        }
        self.bands = bands
        self.lanes = lanes
        size = CGSize(
            width: Geo.gutterWidth + CGFloat(widestGroup) * (Geo.columnWidth + Geo.columnGap) + 40,
            height: (bands.last?.y ?? 0) + 40
        )
    }
}

/// `ScrollViewReader` cannot target positioned views, so AppKit scrolls the enclosing clip view instead.
private struct OverviewScrollRevealer: NSViewRepresentable {
    let rect: CGRect?

    func makeNSView(context: Context) -> FlippedView {
        FlippedView()
    }

    func updateNSView(_ view: FlippedView, context: Context) {
        guard let rect, context.coordinator.revealed != rect else { return }
        context.coordinator.revealed = rect
        DispatchQueue.main.async { view.scrollToVisible(rect) }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    final class Coordinator {
        var revealed: CGRect?
    }

    final class FlippedView: NSView {
        override var isFlipped: Bool {
            true
        }
    }
}

private struct OverviewChangeRow: View {
    let change: OverviewChange
    let isChosen: Bool

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    var body: some View {
        HStack(spacing: 6) {
            node
                .frame(width: OverviewGeometry.nodeInset * 2, height: OverviewGeometry.rowHeight)
            Text(change.changeId.highlighted(scheme: colorScheme, font: fontFamily.scaledFont(10, baseSize: baseFontSize, design: .monospaced)))
                .lineLimit(1)
                .layoutPriority(1)
            if change.hasConflict {
                Image(systemName: "exclamationmark.triangle.fill")
                    .jayjayFont(9, weight: .semibold)
                    .foregroundStyle(.red)
                    .accessibilityLabel("Conflicted")
                    .help("This change has conflicts")
            }
            Text(change.title)
                .jayjayFont(11)
                .foregroundStyle(change.description.isEmpty ? .tertiary : .primary)
                .lineLimit(1)
                .truncationMode(.tail)
            Spacer(minLength: 0)
        }
        .padding(.trailing, 4)
        .background(isChosen ? Color.accentColor.opacity(0.15) : Color.clear, in: RoundedRectangle(cornerRadius: 4))
        .contentShape(Rectangle())
    }

    private var node: some View {
        let radius: CGFloat = change.workspaces.isEmpty ? 4 : 4.5
        return Circle()
            .fill(fill)
            .overlay(Circle().stroke(stroke, lineWidth: 1.5))
            .frame(width: radius * 2, height: radius * 2)
    }

    private var fill: Color {
        if change.hasConflict {
            return .red
        }
        if !change.workspaces.isEmpty {
            return AppColors.workspace(colorScheme)
        }
        if change.isEmpty {
            return Color(nsColor: .windowBackgroundColor)
        }
        return .secondary
    }

    private var stroke: Color {
        change.isEmpty && change.workspaces.isEmpty ? .secondary : .clear
    }
}

private struct OverviewTrunkLabel: View {
    let base: OverviewBase
    let trunkName: String

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    var body: some View {
        HStack(spacing: 6) {
            Text(label)
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(tint)
                .lineLimit(1)
            Text(base.commitId.highlighted(
                scheme: colorScheme,
                font: fontFamily.scaledFont(10, baseSize: baseFontSize, design: .monospaced),
                prefixColor: AppColors.commitIdPrefix(colorScheme)
            ))
            .lineLimit(1)
        }
        .padding(.horizontal, 4)
        .background(Color(nsColor: .windowBackgroundColor))
        .help(base.description)
    }

    private var label: String {
        switch base.kind {
            case .trunk: base.bookmarks.first ?? trunkName
            case .olderTrunk: "\(base.behindTrunk) behind \(trunkName)"
            case .mutable: "fork point"
            case .other: base.bookmarks.first ?? base.changeId.prefix
        }
    }

    private var tint: Color {
        switch base.kind {
            case .trunk: AppColors.workspace(colorScheme)
            case .olderTrunk: .orange
            case .mutable, .other: .secondary
        }
    }
}
