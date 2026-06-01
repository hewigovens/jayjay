import XCTest

final class CommandPaletteScene: SceneBase {
    func testOpenAndSearch() throws {
        let app = try XCTUnwrap(app)
        keyStroke("p", modifiers: [.command, .shift])

        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Command palette did not open")

        field.typeText("toggle tr")
        XCTAssertTrue(
            app.staticTexts["Toggle Tree File List"].waitForExistence(timeout: 2),
            "Tree command should be visible for 'toggle tr'"
        )
        XCTAssertFalse(app.staticTexts["Refresh"].exists, "Refresh should not remain after filtering")
        keyStroke(.escape)
    }
}
