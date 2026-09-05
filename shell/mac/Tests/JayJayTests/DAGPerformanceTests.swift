@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class DAGPerformanceTests: XCTestCase {
    func testLargeGraphLaneProjection() {
        let layout = DAGLayout(entries: Self.entries)
        let options = XCTMeasureOptions()
        options.iterationCount = 3
        measure(metrics: [XCTClockMetric()], options: options) {
            let start = ContinuousClock.now
            var total = 0
            for _ in 0 ..< 20 {
                for lane in 0 ..< 32 {
                    total += layout.displayLane(for: lane)
                }
            }
            XCTAssertEqual(total, 1800)
            XCTAssertLessThan(start.duration(to: .now), .milliseconds(200))
        }
    }

    func testLargeGraphRowMenuEligibility() {
        let layout = DAGLayout(entries: Self.entries)
        let options = XCTMeasureOptions()
        options.iterationCount = 3
        measure(metrics: [XCTClockMetric()], options: options) {
            let start = ContinuousClock.now
            let viewModel = DAGViewModel(
                entries: Self.entries,
                selectedId: "change-0",
                selectedIds: ["change-0"],
                compareFromId: nil,
                contextTargetId: nil,
                rebaseDrag: nil,
                bookmarkDrag: nil,
                colorScheme: .light,
                layout: layout
            )
            for target in Self.entries.prefix(20) {
                XCTAssertTrue(viewModel.canMergeSelectedChange(with: target.change))
            }
            XCTAssertLessThan(start.duration(to: .now), .milliseconds(200))
        }
    }

    private static let entries: [GraphEntry] = {
        let rowCount = 12000
        let heads = (0 ..< 32).map { entry("head-\($0)", parents: ["commit-\(rowCount - 1 - $0)"]) }
        let chain = (0 ..< rowCount).map { index in
            entry("\(index)", parents: index + 1 < rowCount ? ["commit-\(index + 1)"] : [])
        }
        return heads + chain
    }()

    private static func entry(_ id: String, parents: [String]) -> GraphEntry {
        GraphEntry(
            change: mockChangeInfo(changeId: "change-\(id)", commitId: "commit-\(id)", parents: parents),
            edges: parents.map { GraphEdge(target: $0, edgeType: .direct) }
        )
    }
}
