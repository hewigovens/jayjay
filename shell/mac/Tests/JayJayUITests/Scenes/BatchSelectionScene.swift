import XCTest

final class BatchSelectionScene: SceneBase {
    override class var fixtureName: String {
        "dag-long"
    }

    func testLinearSelectionOffersSquashAbandonAndRebase() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        let first = rows.element(boundBy: 0)
        let second = rows.element(boundBy: 1)
        let third = rows.element(boundBy: 2)
        let destination = rows.element(boundBy: 3)
        XCTAssertTrue(first.waitForExistence(timeout: 10), "DAG never populated")

        first.click()
        XCUIElement.perform(withKeyModifiers: .command) {
            second.click()
            third.click()
        }

        rightClickCenter(second)
        let squash = app.menuItems["Squash 3 selected…"]
        XCTAssertTrue(squash.waitForExistence(timeout: 3))
        XCTAssertTrue(squash.isEnabled)
        let abandon = app.menuItems["Abandon 3 selected…"]
        XCTAssertTrue(abandon.waitForExistence(timeout: 3))
        XCTAssertTrue(abandon.isEnabled)
        squash.click()
        XCTAssertTrue(app.staticTexts["Squash 3 Changes?"].waitForExistence(timeout: 3))
        app.buttons["Cancel"].click()

        rightClickCenter(second)
        XCTAssertTrue(abandon.waitForExistence(timeout: 3))
        abandon.click()
        XCTAssertTrue(app.staticTexts["Abandon 3 Changes?"].waitForExistence(timeout: 3))
        app.buttons["Cancel"].click()

        rightClickCenter(destination)
        let rebase = app.menuItems["Rebase 3 selected onto this"]
        XCTAssertTrue(rebase.waitForExistence(timeout: 3))
        XCTAssertTrue(rebase.isEnabled)
    }
}
