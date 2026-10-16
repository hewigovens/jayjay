import XCTest

final class FileHistoryCompareScene: SceneBase {
    override static var fixtureName: String {
        "file-history"
    }

    func testPinMoveSwapAndClearTheComparison() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()
        rows.element(boundBy: 0).click()

        let file = fileRows(of: app).element(boundBy: 0)
        XCTAssertTrue(file.waitForExistence(timeout: 5), "No file rows for the selected change")
        file.rightClick()
        app.menuItems["File History"].firstMatch.click()
        let window = app.windows["file-history-window"]
        XCTAssertTrue(window.waitForExistence(timeout: 5), "File History window did not open")
        XCTAssertTrue(app.element(AID.FileHistory.row(2)).firstMatch.waitForExistence(timeout: 5), "Expected three revisions")

        app.element(AID.FileHistory.row(0)).firstMatch.click()
        app.buttons["Pin as Base"].click()
        let sameRevision = app.staticTexts["Same Revision as the Base"]
        XCTAssertTrue(sameRevision.waitForExistence(timeout: 5), "Pinning the target should compare it with itself")

        app.typeKey(.downArrow, modifierFlags: [])
        XCTAssertTrue(sameRevision.waitForNonExistence(timeout: 5), "Arrowing should move only the target")
        let base = app.element(AID.FileHistory.base)
        let target = app.element(AID.FileHistory.target)
        let (pinned, moved) = (base.value as? String, target.value as? String)
        XCTAssertNotNil(pinned)
        XCTAssertNotEqual(pinned, moved)

        app.element(AID.FileHistory.swap).click()
        XCTAssertEqual(base.value as? String, moved)
        XCTAssertEqual(target.value as? String, pinned)

        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(app.element(AID.FileHistory.swap).waitForNonExistence(timeout: 5), "Esc should clear the base first")
        XCTAssertTrue(window.exists, "Clearing the base must keep File History open")
        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(window.waitForNonExistence(timeout: 5), "A second Esc should close the window")
    }
}
