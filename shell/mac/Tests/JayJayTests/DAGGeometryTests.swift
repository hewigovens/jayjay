@testable import JayJay
import JayJayCore
import SwiftUI
import XCTest

final class DAGGeometryTests: XCTestCase {
    func testLinkComponentsPreserveEveryTypedRendererSegment() {
        let cell = DagLinkCell(
            vertical: .direct,
            horizontal: .indirect,
            leftFork: .direct,
            rightFork: .indirect,
            leftMerge: .direct,
            rightMerge: .indirect,
            isChild: true
        )

        XCTAssertEqual(
            cell.components,
            [
                .vertical(.direct),
                .horizontal(.indirect),
                .leftFork(.direct),
                .rightFork(.indirect),
                .leftMerge(.direct),
                .rightMerge(.indirect)
            ]
        )
    }

    func testForksAndMergesRetainRoundedElbows() {
        let bendComponents: [DAGLinkComponent] = [
            .leftFork(.direct),
            .rightFork(.direct),
            .leftMerge(.direct),
            .rightMerge(.direct)
        ]
        let straightComponents: [DAGLinkComponent] = [
            .vertical(.direct),
            .horizontal(.direct)
        ]

        XCTAssertTrue(bendComponents.allSatisfy(\.pathForTest.containsQuadraticCurve))
        XCTAssertTrue(straightComponents.allSatisfy { !$0.pathForTest.containsQuadraticCurve })
    }

    func testNodeColumnLinkStartsOutsideNode() {
        let geometry = DAGGeometry(logicalColumnCount: 3, availableSidebarWidth: 320)
        let nodeColumn = 1
        let otherColumn = 2
        let nodeY: CGFloat = 12
        let nodeRadius: CGFloat = 5

        XCTAssertEqual(
            geometry.linkTopY(forColumn: nodeColumn, nodeColumn: nodeColumn, nodeY: nodeY, nodeRadius: nodeRadius),
            nodeY + nodeRadius
        )
        XCTAssertEqual(
            geometry.linkTopY(forColumn: otherColumn, nodeColumn: nodeColumn, nodeY: nodeY, nodeRadius: nodeRadius),
            nodeY
        )
    }

    func testNarrowGraphUsesPreferredPitchAndGrowsOncePerColumn() {
        let one = DAGGeometry(logicalColumnCount: 1, availableSidebarWidth: 1000)
        let two = DAGGeometry(logicalColumnCount: 2, availableSidebarWidth: 1000)

        XCTAssertEqual(one.lanePitch, DAGGeometry.preferredLanePitch)
        XCTAssertEqual(two.lanePitch, DAGGeometry.preferredLanePitch)
        XCTAssertEqual(two.graphWidth, one.graphWidth + DAGGeometry.preferredLanePitch)
    }

    func testWidthBudgetNeverCompressesBelowLegiblePitch() {
        let geometry = DAGGeometry(logicalColumnCount: 10, availableSidebarWidth: 200)

        XCTAssertEqual(geometry.lanePitch, DAGGeometry.minimumLegibleLanePitch)
        XCTAssertEqual(
            geometry.graphWidth,
            DAGGeometry.horizontalPadding + 10 * DAGGeometry.minimumLegibleLanePitch
        )
        XCTAssertEqual(geometry.nodeRadius, DAGGeometry.preferredNodeRadius)
    }

    func testGraphUsesResponsiveAndAbsoluteBudgetsWhenTheyRemainLegible() {
        let responsiveCapped = DAGGeometry(logicalColumnCount: 3, availableSidebarWidth: 200)
        XCTAssertLessThanOrEqual(responsiveCapped.graphWidth, 200 * DAGGeometry.maxSidebarFraction + 0.01)

        let absoluteCapped = DAGGeometry(logicalColumnCount: 12, availableSidebarWidth: 10000)
        XCTAssertLessThanOrEqual(absoluteCapped.graphWidth, DAGGeometry.absoluteGraphMaxWidth)
    }

    func testFirstAndLastColumnCentresStayInsideFrame() {
        for columns in [1, 2, 5, 12] {
            for width: CGFloat in [80, 200, 1000] {
                let geometry = DAGGeometry(logicalColumnCount: columns, availableSidebarWidth: width)
                let first = geometry.xPosition(forColumn: 0)
                let last = geometry.xPosition(forColumn: columns - 1)

                XCTAssertGreaterThan(first, 0, "columns=\(columns) width=\(width)")
                XCTAssertLessThan(last, geometry.graphWidth, "columns=\(columns) width=\(width)")
            }
        }
    }

    /// Rendering and rebase hit-testing both call `xPosition(forColumn:)` on the same `DAGGeometry` value for a row's `nodeColumn` — this only holds if the function is pure, so two identically-configured geometries must agree exactly.
    func testNodePositionIsDeterministicForRebaseHitTesting() {
        let a = DAGGeometry(logicalColumnCount: 6, availableSidebarWidth: 260)
        let b = DAGGeometry(logicalColumnCount: 6, availableSidebarWidth: 260)

        for column in 0 ..< 6 {
            XCTAssertEqual(a.xPosition(forColumn: column), b.xPosition(forColumn: column))
        }
    }

    func testChangingSidebarWidthCannotMakePitchSubLegible() {
        let narrow = DAGGeometry(logicalColumnCount: 8, availableSidebarWidth: 150)
        let wide = DAGGeometry(logicalColumnCount: 8, availableSidebarWidth: 1000)

        XCTAssertEqual(narrow.logicalColumnCount, wide.logicalColumnCount)
        XCTAssertNotEqual(narrow.lanePitch, wide.lanePitch)
        XCTAssertEqual(narrow.lanePitch, DAGGeometry.minimumLegibleLanePitch)
    }

    func testContinuationMarkerUsesABoundaryLocalStub() {
        let outgoing = DAGContinuationMarkerGeometry(
            direction: .outgoing,
            x: 20,
            rowHeight: 44
        )
        let incoming = DAGContinuationMarkerGeometry(
            direction: .incoming,
            x: 20,
            rowHeight: 44
        )

        XCTAssertGreaterThan(outgoing.tip.y, outgoing.shaftStart.y)
        XCTAssertEqual(outgoing.tip.y, 42)
        XCTAssertEqual(outgoing.shaftStart.y, 34)
        XCTAssertEqual(outgoing.tip.x, 20)
        XCTAssertLessThan(incoming.tip.y, incoming.shaftStart.y)
        XCTAssertEqual(incoming.tip.y, 2)
        XCTAssertEqual(incoming.shaftStart.y, 10)
        XCTAssertEqual(incoming.tip.x, 20)
        for point in outgoing.points + incoming.points {
            XCTAssertGreaterThanOrEqual(point.y, 0)
            XCTAssertLessThanOrEqual(point.y, 44)
        }
    }
}

private extension DAGLinkComponent {
    var pathForTest: Path {
        path(in: .init(x: 10, topY: 0, centerY: 10, bottomY: 20, halfPitch: 10, cornerRadius: 6))
    }
}

private extension Path {
    var containsQuadraticCurve: Bool {
        var result = false
        cgPath.applyWithBlock { element in
            result = result || element.pointee.type == .addQuadCurveToPoint
        }
        return result
    }
}
