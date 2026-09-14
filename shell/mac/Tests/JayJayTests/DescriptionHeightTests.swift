@testable import JayJay
import XCTest

final class DescriptionHeightTests: XCTestCase {
    func testExpandedHeightFollowsThePaneAboveTheFloor() {
        XCTAssertEqual(DescriptionHeight.expanded(paneHeight: 400), 160)
        XCTAssertEqual(DescriptionHeight.expanded(paneHeight: 1000), 300)
    }
}
