@testable import JayJay
import JayJayCore
import XCTest

final class FileHistoryComparisonTests: XCTestCase {
    func testPinSwapAndClearKeepEndpointsConsistent() {
        var comparison = FileHistoryComparison()
        comparison.select(endpoint("a"))
        comparison.pin(endpoint("a"))
        XCTAssertEqual(comparison.base?.commitId.id, "a")
        XCTAssertEqual(comparison.target?.commitId.id, "a")

        comparison.select(endpoint("b"))
        XCTAssertEqual(comparison.compareFrom, "a")
        XCTAssertEqual(comparison.compareTo, "b")

        comparison.swap()
        XCTAssertEqual(comparison.base?.commitId.id, "b")
        XCTAssertEqual(comparison.target?.commitId.id, "a")
        XCTAssertEqual(comparison.compareFrom, "b")
        XCTAssertEqual(comparison.compareTo, "a")

        comparison.clear()
        XCTAssertNil(comparison.base)
        XCTAssertFalse(comparison.isPinned)
        XCTAssertEqual(comparison.target?.commitId.id, "a")
        XCTAssertNil(comparison.compareFrom)
        XCTAssertEqual(comparison.compareTo, "a")
    }

    func testSameRevisionRequiresBothEndpoints() {
        var comparison = FileHistoryComparison()
        XCTAssertFalse(comparison.isSameRevision, "an empty comparison is not the same revision")

        comparison.select(endpoint("a"))
        XCTAssertFalse(comparison.isSameRevision, "an unpinned target is not compared with itself")

        comparison.pin(endpoint("a"))
        XCTAssertTrue(comparison.isSameRevision)

        comparison.select(endpoint("b"))
        XCTAssertFalse(comparison.isSameRevision)
    }

    func testSwapWithoutABaseIsANoOp() {
        var comparison = FileHistoryComparison()
        comparison.select(endpoint("a"))
        comparison.swap()
        XCTAssertNil(comparison.base)
        XCTAssertEqual(comparison.target?.commitId.id, "a")
    }

    private func endpoint(_ commitId: String) -> FileHistoryEndpoint {
        FileHistoryEndpoint(
            commitId: ShortId(id: commitId, shortLen: 3),
            changeId: ShortId(id: "change-\(commitId)", shortLen: 3)
        )
    }
}
