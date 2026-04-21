import XCTest

final class UndoScene: SceneBase {
    func testOpenOperationLog() throws {
        let app = try XCTUnwrap(app)
        keyStroke("u", modifiers: [.command, .shift])

        let title = app.staticTexts["Operation Log"]
        XCTAssertTrue(title.waitForExistence(timeout: 5), "Undo/Operation Log view did not open")

        keyStroke(.escape)
    }
}
