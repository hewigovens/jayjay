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

    func testNarrowGraphUsesPreferredPitchAndGrowsOncePerColumn() {
        let one = DAGGeometry(logicalColumnCount: 1, availableSidebarWidth: 1000)
        let two = DAGGeometry(logicalColumnCount: 2, availableSidebarWidth: 1000)

        XCTAssertEqual(one.lanePitch, DAGGeometry.preferredLanePitch)
        XCTAssertEqual(two.lanePitch, DAGGeometry.preferredLanePitch)
        XCTAssertEqual(two.graphWidth, one.graphWidth + DAGGeometry.preferredLanePitch)
    }

    func testWidthBudgetReachedCompressesUniformly() {
        // A narrow sidebar caps the 45% budget well below the preferred width for 10 columns.
        let geometry = DAGGeometry(logicalColumnCount: 10, availableSidebarWidth: 200)

        XCTAssertLessThan(geometry.lanePitch, DAGGeometry.preferredLanePitch)
        XCTAssertEqual(geometry.graphWidth, 200 * DAGGeometry.maxSidebarFraction, accuracy: 0.01)
    }

    func testGraphWidthNeverExceedsResponsiveOrAbsoluteCap() {
        let responsiveCapped = DAGGeometry(logicalColumnCount: 3, availableSidebarWidth: 200)
        XCTAssertLessThanOrEqual(responsiveCapped.graphWidth, 200 * DAGGeometry.maxSidebarFraction + 0.01)

        let absoluteCapped = DAGGeometry(logicalColumnCount: 100, availableSidebarWidth: 10000)
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

    func testChangingSidebarWidthChangesPitchButNotLogicalColumnCount() {
        let narrow = DAGGeometry(logicalColumnCount: 8, availableSidebarWidth: 150)
        let wide = DAGGeometry(logicalColumnCount: 8, availableSidebarWidth: 1000)

        XCTAssertEqual(narrow.logicalColumnCount, wide.logicalColumnCount)
        XCTAssertNotEqual(narrow.lanePitch, wide.lanePitch)
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
