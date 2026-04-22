import XCTest

final class BookmarkManagerScene: SceneBase {
    func testOpenViaShortcut() throws {
        let app = try XCTUnwrap(app)
        keyStroke("b", modifiers: [.command, .shift])

        let title = app.staticTexts["Bookmark Manager"]
        XCTAssertTrue(title.waitForExistence(timeout: 5), "Bookmark Manager did not open")

        keyStroke(.escape)
    }
}
