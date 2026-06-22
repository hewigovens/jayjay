import AppKit
import XCTest

final class CommandPaletteScene: SceneBase {
    /// Trigger → move → reopen → type: remembers a dragged position across reopen, and filters.
    func testMovePersistsThenSearch() throws {
        let app = try XCTUnwrap(app)

        // Trigger.
        keyStroke("p", modifiers: [.command, .shift])
        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Command palette did not open")
        let original = field.frame.origin

        // Move — drag the panel by its background (the padding above the search field).
        let handle = field.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0))
            .withOffset(CGVector(dx: 0, dy: -8))
        handle.press(forDuration: 0.4, thenDragTo: handle.withOffset(CGVector(dx: 150, dy: 120)))
        let moved = field.frame.origin
        XCTAssertNotEqual(moved, original, "Palette did not move when dragged")

        // Trigger again — close and reopen; it must reappear where we left it, not re-center.
        keyStroke(.escape)
        XCTAssertTrue(field.waitForNonExistence(timeout: 3), "Palette did not close")
        keyStroke("p", modifiers: [.command, .shift])
        XCTAssertTrue(field.waitForExistence(timeout: 5), "Palette did not reopen")
        let reopened = field.frame.origin
        XCTAssertEqual(reopened.x, moved.x, accuracy: 6, "Palette x reset after reopen")
        XCTAssertEqual(reopened.y, moved.y, accuracy: 6, "Palette y reset after reopen")

        // Type — search filters the list.
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
