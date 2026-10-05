import AppKit
@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

@MainActor
final class OverviewCanvasTests: XCTestCase {
    func testLargeOverviewRendersAndReselectsQuickly() throws {
        let (lanes, groups) = Self.overview(groupCount: 520, changesPerLane: 12)
        let laneIds = lanes.map(\.head.changeId.id)
        let placement = OverviewPlacement(lanes: lanes, groups: groups)
        let settings = try AppSettings(defaults: XCTUnwrap(UserDefaults(suiteName: "OverviewCanvasTests")))
        func canvas(selecting laneId: String) -> AnyView {
            AnyView(
                ScrollView([.horizontal, .vertical]) {
                    OverviewCanvas(
                        lanes: lanes,
                        laneIds: laneIds,
                        placement: placement,
                        trunkName: "main",
                        selectedLaneId: .constant(laneId),
                        selectedChangeId: .constant(nil),
                        isLanePanelShown: .constant(false),
                        onShowInGraph: { _, _ in },
                        actions: OverviewLaneActions(
                            workspaceInfo: { _ in nil },
                            openWorkspace: { _ in },
                            rebaseOntoTrunk: { _ in },
                            abandon: { _ in },
                            forgetWorkspace: { _, _ in }
                        )
                    )
                }
                .environment(settings)
            )
        }
        let hosting = NSHostingView(rootView: canvas(selecting: laneIds[0]))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1000, height: 700), styleMask: [.titled], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false
        defer { window.close() }
        window.contentView = hosting
        window.orderBack(nil)

        let firstRender = ContinuousClock.now
        hosting.layoutSubtreeIfNeeded()
        hosting.displayIfNeeded()
        XCTAssertLessThan(firstRender.duration(to: .now), .milliseconds(500))

        let reselect = ContinuousClock.now
        hosting.rootView = canvas(selecting: laneIds[1])
        hosting.layoutSubtreeIfNeeded()
        hosting.displayIfNeeded()
        XCTAssertLessThan(reselect.duration(to: .now), .milliseconds(200))
    }

    func testPlacementCullsToTheVisibleRegion() throws {
        let (lanes, groups) = Self.overview(groupCount: 40, changesPerLane: 12)
        let placement = OverviewPlacement(lanes: lanes, groups: groups)
        let deep = try XCTUnwrap(placement.lanes.last)
        let top = OverviewPlacement.tiled(CGRect(x: 0, y: 0, width: 1000, height: 700))
        XCTAssertFalse(placement.lanes(in: top).contains { $0.laneIndex == deep.laneIndex })

        let screen = CGRect(x: 0, y: deep.frame.minY, width: 1000, height: 700)
        let around = OverviewPlacement.tiled(screen)
        XCTAssertTrue(around.contains(screen))
        XCTAssertTrue(placement.lanes(in: around).contains { $0.laneIndex == deep.laneIndex })
        XCTAssertEqual(deep.rowOffsets(in: around), Array(0 ..< deep.count))
        let visibleBandYs = placement.bands(in: around).map(\.y)
        XCTAssertTrue(visibleBandYs.contains(deep.bandY))
        XCTAssertFalse(visibleBandYs.contains(placement.bands[0].y))

        let firstRowOnly = CGRect(x: deep.x, y: deep.nodeY(0) - 1, width: 10, height: 2)
        XCTAssertEqual(deep.rowOffsets(in: firstRowOnly), [0])
    }

    private static func overview(groupCount: Int, changesPerLane: Int) -> ([OverviewLane], [OverviewGroup]) {
        var lanes: [OverviewLane] = []
        var groups: [OverviewGroup] = []
        for group in 0 ..< groupCount {
            let base = OverviewBase(
                changeId: ShortId(id: "base\(group)", shortLen: 4),
                commitId: ShortId(id: "basecommit\(group)", shortLen: 4),
                description: "base \(group)",
                timestampMillis: 0,
                kind: group == 0 ? .trunk : .olderTrunk,
                bookmarks: [],
                behindTrunk: UInt32(group)
            )
            let laneCount = group.isMultiple(of: 100) ? 6 : group.isMultiple(of: 5) ? 2 : 1
            var indexes: [UInt32] = []
            for lane in 0 ..< laneCount {
                let changes = (0 ..< changesPerLane).map { change in
                    OverviewChange(
                        changeId: ShortId(id: "g\(group)l\(lane)c\(change)", shortLen: 6),
                        commitId: ShortId(id: "commit\(group)-\(lane)-\(change)", shortLen: 6),
                        description: "change \(change) of lane \(lane) in group \(group)",
                        fullDescription: "",
                        timestampMillis: 0,
                        isEmpty: false,
                        hasConflict: false,
                        bookmarks: change == 0 ? ["feature/\(group)-\(lane)"] : [],
                        workspaces: []
                    )
                }
                indexes.append(UInt32(lanes.count))
                lanes.append(OverviewLane(changes: changes, base: base, workspaces: [], latestTimestampMillis: 0, attention: []))
            }
            groups.append(OverviewGroup(base: base, lanes: indexes))
        }
        return (lanes, groups)
    }
}
