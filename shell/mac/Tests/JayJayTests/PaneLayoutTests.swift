@testable import JayJay
import XCTest

final class PaneLayoutTests: XCTestCase {
    func testOperationLogLeavesRoomForTheRestOfTheWindow() {
        XCTAssertEqual(PaneLayout.operationLogRange(windowWidth: 2000, sidebarShown: true), 220 ... 560)
        XCTAssertEqual(PaneLayout.operationLogRange(windowWidth: 1200, sidebarShown: true), 220 ... 288)
        XCTAssertEqual(PaneLayout.operationLogRange(windowWidth: 1200, sidebarShown: false), 220 ... 559)
    }
}
