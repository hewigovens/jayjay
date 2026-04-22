import XCTest

final class ReviewSplitScene: SceneBase {
    func testMultiSelectAndSplit() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // Simple fixture's @ has wip1.txt and wip2.txt.
        let files = fileRows(of: app)
        XCTAssertTrue(files.element(boundBy: 0).waitForExistence(timeout: 5), "Working copy files did not appear")

        files.element(boundBy: 0).click()
        keyStroke(.space)
        XCUIElement.perform(withKeyModifiers: .shift) {
            files.element(boundBy: 1).click()
        }
    }
}
