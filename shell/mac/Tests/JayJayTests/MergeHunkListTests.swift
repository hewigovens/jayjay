import Foundation
@testable import JayJay
import XCTest

@MainActor
final class MergeHunkListTests: XCTestCase {
    func testTallVisibleHunkWinsOverPrefetchedNeighbor() {
        let frames: [UInt32: CGRect] = [
            0: CGRect(x: 0, y: 0, width: 400, height: 2000),
            1: CGRect(x: 0, y: 2012, width: 400, height: 100)
        ]
        for y in [1600.0, 1800.0, 2012.0] {
            let viewport = CGRect(x: 0, y: y, width: 400, height: 180)
            XCTAssertEqual(MergeHunkList.nearestHunk(in: frames, viewport: viewport), y < 2000 ? 0 : 1)
        }
    }
}
