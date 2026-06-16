import AppKit
import XCTest

final class CommandPaletteScene: SceneBase {
    func testOpenAndSearch() throws {
        let app = try XCTUnwrap(app)
        keyStroke("p", modifiers: [.command, .shift])

        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Command palette did not open")

        let query = "toggle tr"
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(query, forType: .string)
        field.click()
        keyStroke("v", modifiers: [.command])
        let queryEntered = NSPredicate { _, _ in
            (field.value as? String ?? "") == query
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: queryEntered, object: nil)], timeout: 2),
            .completed,
            "Command palette did not receive search text; value=\(field.value as? String ?? "<nil>")"
        )

        let treeCommand = app.descendants(matching: .any)[AID.Palette.item("Toggle Tree File List")]
        XCTAssertTrue(
            treeCommand.waitForExistence(timeout: 2),
            "Tree command should be visible for 'toggle tr'"
        )
        let refreshCommand = app.descendants(matching: .any)[AID.Palette.item("Refresh")]
        let refreshRemoved = NSPredicate { _, _ in !refreshCommand.exists }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: refreshRemoved, object: nil)], timeout: 2),
            .completed,
            "Refresh should not remain after filtering"
        )
        keyStroke(.escape)
    }
}
