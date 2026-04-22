import XCTest

final class InterdiffScene: SceneBase {
    func testShiftClickTwoRows() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        rows.element(boundBy: 0).click()
        XCUIElement.perform(withKeyModifiers: .shift) {
            rows.element(boundBy: 3).click()
        }
    }
}
