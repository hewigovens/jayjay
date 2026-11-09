import XCTest

final class UndoScene: SceneBase {
    func testOpenOperationLog() throws {
        let app = try XCTUnwrap(app)
        keyStroke("u", modifiers: [.command, .shift])

        let title = app.staticTexts["Operation Log"]
        XCTAssertTrue(title.waitForExistence(timeout: 5), "Operation Log did not open")
        let restore = app.buttons["Restore"]
        XCTAssertTrue(restore.waitForExistence(timeout: 5), "Restore button missing")
        expectation(for: NSPredicate(format: "isEnabled == true"), evaluatedWith: restore)
        waitForExpectations(timeout: 5)

        keyStroke(.escape)
    }
}
