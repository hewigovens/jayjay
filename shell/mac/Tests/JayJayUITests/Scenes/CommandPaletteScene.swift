import XCTest

final class CommandPaletteScene: SceneBase {
    func testOpenAndSearch() throws {
        let app = try XCTUnwrap(app)
        keyStroke("p", modifiers: [.command, .shift])

        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Command palette did not open")

        field.typeText("toggle tr")
        let treeCommand = app.buttons[AID.Palette.item("Toggle Tree File List")]
        XCTAssertTrue(
            treeCommand.waitForExistence(timeout: 2),
            "Tree command should be visible for 'toggle tr'"
        )
        XCTAssertFalse(app.buttons[AID.Palette.item("Refresh")].exists, "Refresh should not remain after filtering")
        keyStroke(.escape)
    }
}
