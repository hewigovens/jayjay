import XCTest

final class SidebarToggleScene: SceneBase {
    func testHidingTheSidebarKeepsItsStateAndRevealsItOnDemand() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "DAG never populated")
        rows.firstMatch.click()
        let firstFile = app.descendants(matching: .any)[AID.FileList.row("wip1.txt")]
        let secondFile = app.descendants(matching: .any)[AID.FileList.row("wip2.txt")]
        XCTAssertTrue(firstFile.waitForExistence(timeout: 10), "Detail never loaded the working copy")
        let column = app.descendants(matching: .any)[AID.FileList.column]
        let columnWidth = column.frame.width
        let divider = app.descendants(matching: .any)[AID.Sidebar.divider]
        XCTAssertTrue(divider.waitForExistence(timeout: 5), "Sidebar divider missing")
        let sidebarWidth = divider.frame.minX

        app.buttons[AID.Toolbar.sidebarToggle].click()
        XCTAssertTrue(divider.waitForNonExistence(timeout: 5), "The sidebar stayed visible")
        XCTAssertTrue(rows.firstMatch.waitForNonExistence(timeout: 5), "The hidden history pane stayed reachable")
        XCTAssertEqual(column.frame.minX, app.windows.firstMatch.frame.minX, accuracy: 2, "The detail pane did not take the freed width")
        // A narrow window squeezes the file column below its stored width while the sidebar is shown, so hiding may give that back.
        XCTAssertGreaterThanOrEqual(column.frame.width, columnWidth - 2, "Hiding the sidebar shrank the file column")

        keyStroke(.downArrow)
        XCTAssertTrue(secondFile.wait(for: \.isSelected, toEqual: true, timeout: 5), "Navigation keys did not move to the file list")

        app.menuBars.menuBarItems["View"].click()
        XCTAssertTrue(app.menuItems["Show Sidebar"].waitForExistence(timeout: 3), "The View menu did not offer Show Sidebar")
        keyStroke(.escape)

        keyStroke("s", modifiers: [.control, .command])
        XCTAssertTrue(divider.waitForExistence(timeout: 5), "⌃⌘S did not bring the sidebar back")
        XCTAssertTrue(rows.firstMatch.wait(for: \.isSelected, toEqual: true, timeout: 5), "The DAG selection did not survive the round trip")
        XCTAssertTrue(secondFile.isSelected, "The file selection did not survive the round trip")
        XCTAssertEqual(divider.frame.minX, sidebarWidth, accuracy: 2, "The sidebar came back at a different width")
        XCTAssertEqual(column.frame.width, columnWidth, accuracy: 2, "The round trip resized the file column")

        keyStroke("s", modifiers: [.control, .command])
        XCTAssertTrue(divider.waitForNonExistence(timeout: 5), "⌃⌘S did not hide the sidebar")
        app.buttons["Filter"].click()
        let revset = app.textFields["Revset expression"]
        XCTAssertTrue(revset.waitForExistence(timeout: 5), "The revset filter did not reveal the sidebar")
        keyStroke("a", modifiers: [.command])
        paste("all()")
        XCTAssertEqual(revset.value as? String, "all()", "The revealed revset field did not take typing")

        keyStroke("s", modifiers: [.control, .command])
        XCTAssertTrue(divider.waitForNonExistence(timeout: 5), "⌃⌘S did not hide the sidebar from the revset field")
        keyStroke(.upArrow)
        XCTAssertTrue(firstFile.wait(for: \.isSelected, toEqual: true, timeout: 5), "The hidden revset field kept the keyboard")

        // The flag is a real user default, so leave it shown.
        keyStroke("s", modifiers: [.control, .command])
        XCTAssertTrue(divider.waitForExistence(timeout: 5))
    }
}
