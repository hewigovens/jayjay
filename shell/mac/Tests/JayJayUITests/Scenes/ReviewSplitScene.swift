import XCTest

final class ReviewSplitScene: SceneBase {
    func testMultiSelectAndSplit() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // Simple fixture's @ has wip1.txt and wip2.txt.
        let files = fileRows(of: app)
        XCTAssertTrue(files.element(boundBy: 0).waitForExistence(timeout: 5), "Working copy files did not appear")

        files.element(boundBy: 0).click()
        XCUIElement.perform(withKeyModifiers: .shift) {
            files.element(boundBy: 1).click()
        }
        XCTAssertTrue(files.element(boundBy: 0).isSelected)
        XCTAssertTrue(files.element(boundBy: 1).isSelected)

        XCUIElement.perform(withKeyModifiers: .command) {
            files.element(boundBy: 1).click()
        }
        XCTAssertTrue(files.element(boundBy: 0).isSelected)
        XCTAssertFalse(files.element(boundBy: 1).isSelected)

        XCUIElement.perform(withKeyModifiers: .command) {
            files.element(boundBy: 1).click()
        }
        XCTAssertTrue(files.element(boundBy: 1).isSelected)

        files.element(boundBy: 0).click()
        keyStroke(.space)

        // Space can lose the key-focus race on cold CI runners; re-focus the row and retry once.
        let splitButton = app.buttons[AID.SplitSheet.openButton]
        if !splitButton.waitForExistence(timeout: 3) {
            files.element(boundBy: 0).click()
            keyStroke(.space)
        }
        XCTAssertTrue(splitButton.waitForExistence(timeout: 5), "Split toolbar button did not appear")

        splitButton.click()

        let messageField = app.textFields[AID.SplitSheet.messageField]
        XCTAssertTrue(messageField.waitForExistence(timeout: 5), "Split sheet message field missing")

        // The file list must be populated on the sheet's first render, before any typing (issue #102) ...
        let fileRow = app.staticTexts[AID.SplitSheet.fileRow("wip1.txt")]
        XCTAssertTrue(fileRow.waitForExistence(timeout: 3), "Split sheet file list not populated before typing")

        // ... and sit below the message field so the focused field never jumps.
        XCTAssertGreaterThan(
            fileRow.frame.minY, messageField.frame.maxY,
            "Split sheet file list should render below the message field"
        )

        keyStroke(.escape)
    }
}
