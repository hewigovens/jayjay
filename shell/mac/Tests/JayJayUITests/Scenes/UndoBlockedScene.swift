import XCTest

final class UndoBlockedScene: SceneBase {
    override class var fixtureName: String {
        "undo-blocked"
    }

    func testBlockedUndoExplainsWhyAndOffersTheOperationLog() throws {
        let app = try XCTUnwrap(app)
        let panel = app.element(AID.Panel.operationLog)
        if panel.waitForExistence(timeout: 2) {
            keyStroke("u", modifiers: [.command, .shift])
            XCTAssertTrue(panel.waitForNonExistence(timeout: 5), "The shortcut did not close the Operation Log")
        }
        try waitForDAGRows().firstMatch.click()

        keyStroke("z", modifiers: [.command])
        let reason = app.staticTexts.matching(NSPredicate(format: "value CONTAINS 'snapshot of working-copy edits'")).firstMatch
        XCTAssertTrue(reason.waitForExistence(timeout: 5), "⌘Z on a snapshot gave no reason")

        app.buttons["Operation Log"].firstMatch.click()
        XCTAssertTrue(panel.waitForExistence(timeout: 5), "The notice did not open the Operation Log")
        keyStroke("u", modifiers: [.command, .shift])
        XCTAssertTrue(panel.waitForNonExistence(timeout: 5), "The shortcut did not close the Operation Log")
    }
}
