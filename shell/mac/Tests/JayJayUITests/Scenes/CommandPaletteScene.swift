import XCTest

final class CommandPaletteScene: SceneBase {
    func testOpenAndSearch() throws {
        let app = try XCTUnwrap(app)
        keyStroke("p", modifiers: [.command, .shift])

        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Command palette did not open")

        field.typeText("bookmark")
        keyStroke(.downArrow)
        keyStroke(.downArrow)
        keyStroke(.escape)
    }
}
