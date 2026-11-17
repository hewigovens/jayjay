import XCTest

final class UndoScene: SceneBase {
    func testOperationLogPanel() throws {
        let app = try XCTUnwrap(app)
        let panel = app.element(AID.Panel.operationLog)
        // The panel's visibility persists app-wide, so a run that stopped midway leaves it open.
        if panel.waitForExistence(timeout: 2) {
            keyStroke("u", modifiers: [.command, .shift])
            XCTAssertTrue(panel.waitForNonExistence(timeout: 5), "The shortcut did not close the Operation Log")
        }
        let window = app.windows.firstMatch
        let closedFrame = window.frame
        keyStroke("u", modifiers: [.command, .shift])
        XCTAssertTrue(panel.waitForExistence(timeout: 5), "Operation Log did not open")
        XCTAssertEqual(panel.frame.maxX, window.frame.maxX, accuracy: 2, "The Operation Log should be the trailing column")
        XCTAssertEqual(window.frame, closedFrame, "Opening the Operation Log moved or resized the window")

        XCTAssertFalse(panel.buttons["Restore"].exists, "The panel still has a Restore button")

        let target = panel.staticTexts["new empty commit"].firstMatch
        XCTAssertTrue(target.waitForExistence(timeout: 5), "The restore target row is missing")
        keyStroke(.downArrow)
        keyStroke(.return, modifiers: [.control])
        XCTAssertTrue(app.menuItems["Restore"].waitForExistence(timeout: 5), "Opening the Operation Log did not hand its list the keyboard")
        keyStroke(.escape)
        target.click()
        keyStroke(.return, modifiers: [.control])
        XCTAssertTrue(app.menuItems["Restore"].waitForExistence(timeout: 5), "⌃Return did not open the row's context menu")
        keyStroke(.escape)

        app.menuBars.menuBarItems["Edit"].click()
        let menuRestore = app.menuItems["Restore Selected Operation"]
        XCTAssertTrue(menuRestore.waitForExistence(timeout: 3) && menuRestore.isEnabled, "Edit › Restore Selected Operation is unavailable")
        keyStroke(.escape)

        target.rightClick()
        app.menuItems["Restore"].click()
        let restored = panel.descendants(matching: .any)
            .matching(NSPredicate(format: "label BEGINSWITH 'restore to operation' OR value BEGINSWITH 'restore to operation'")).firstMatch
        XCTAssertTrue(restored.waitForExistence(timeout: 10), "The panel did not refresh after Restore")
        XCTAssertTrue(panel.exists, "Restore closed the Operation Log")

        let toggle = app.toolbars.buttons[AID.Toolbar.operationLog]
        XCTAssertTrue(toggle.waitForExistence(timeout: 5), "The open Operation Log pushed its toolbar toggle into the overflow menu")
        XCTAssertEqual(toggle.frame.minX, panel.frame.minX, accuracy: 20, "The toggle is not above the open Operation Log")
        // While the panel is open the toggle's toolbar item also holds the room that seats it there, so the item's centre is not the button.
        toggle.coordinate(withNormalizedOffset: CGVector(dx: 0, dy: 0.5)).withOffset(CGVector(dx: 24, dy: 0)).click()
        XCTAssertTrue(panel.waitForNonExistence(timeout: 5), "The toolbar toggle did not close the Operation Log")
        XCTAssertTrue(toggle.exists, "The toolbar toggle left with the Operation Log")
        toggle.click()
        XCTAssertTrue(panel.waitForExistence(timeout: 5), "The toolbar toggle did not reopen the Operation Log")

        app.menuBars.menuBarItems["View"].click()
        app.menuItems["Hide Operation Log"].click()
        XCTAssertTrue(panel.waitForNonExistence(timeout: 5), "View › Hide Operation Log did not close the panel")
        keyStroke("u", modifiers: [.command, .shift])
        XCTAssertTrue(panel.waitForExistence(timeout: 5), "Operation Log did not reopen")
        keyStroke("u", modifiers: [.command, .shift])
        XCTAssertTrue(panel.waitForNonExistence(timeout: 5), "The shortcut did not close the Operation Log")
    }
}
