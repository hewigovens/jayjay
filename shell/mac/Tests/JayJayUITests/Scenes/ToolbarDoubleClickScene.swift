import XCTest

final class ToolbarDoubleClickScene: SceneBase {
    func testDoubleClickOnEmptyToolbarZoomsTheWindow() throws {
        let app = try XCTUnwrap(app)
        let window = app.windows.firstMatch
        let frame = window.frame
        // The strip between the window buttons and the first toolbar item belongs to no item.
        let zoomButton = window.buttons[XCUIIdentifierFullScreenWindow]
        let emptyToolbar = zoomButton.coordinate(withNormalizedOffset: CGVector(dx: 1, dy: 0.5)).withOffset(CGVector(dx: 5, dy: 0))

        emptyToolbar.doubleClick()
        let zoomed = XCTNSPredicateExpectation(predicate: NSPredicate { _, _ in window.frame != frame }, object: nil)
        XCTAssertEqual(XCTWaiter().wait(for: [zoomed], timeout: 5), .completed, "Double-clicking the empty toolbar did not zoom the window")
    }

    func testDoubleClickInTheRevsetFieldSelectsTextInsteadOfZooming() throws {
        let app = try XCTUnwrap(app)
        let window = app.windows.firstMatch
        app.buttons[AID.Toolbar.revsetBar].click()
        let revset = app.textFields[AID.Toolbar.revsetField]
        XCTAssertTrue(revset.waitForExistence(timeout: 5), "Clicking the revset bar did not start editing it")
        keyStroke("a", modifiers: [.command])
        paste("mine")
        let frame = window.frame

        revset.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0.5)).withOffset(CGVector(dx: 12, dy: 0)).doubleClick()
        paste("all()")
        XCTAssertEqual(revset.value as? String, "all()", "The double-click did not select the word in the revset field")
        XCTAssertEqual(window.frame, frame, "Double-clicking text in the revset field zoomed the window")
    }
}
