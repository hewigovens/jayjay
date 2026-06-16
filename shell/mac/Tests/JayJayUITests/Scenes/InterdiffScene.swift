import XCTest

final class InterdiffScene: SceneBase {
    func testShiftClickTwoRows() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        rows.element(boundBy: 0).click()
        XCUIElement.perform(withKeyModifiers: .shift) {
            let target = rows.element(boundBy: 3)
            XCTAssertTrue(target.waitForExistence(timeout: 5), "Comparison target row did not appear")
            target.click()
        }

        let banner = app.descendants(matching: .any)[AID.Compare.banner]
        XCTAssertTrue(banner.waitForExistence(timeout: 5), "Compare banner did not appear")
    }
}
