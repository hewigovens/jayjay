import AppKit
import XCTest

final class KeyboardContextMenuScene: SceneBase {
    func testMouseMenuRunsActionKeys() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()
        let second = app.element(rows.element(boundBy: 1).identifier)
        rightClickCenter(second, timeout: 5, message: "Second change row missing")
        XCTAssertTrue(app.menuItems["Show evolution…"].waitForExistence(timeout: 3), "Right-click did not open the change menu")
        keyStroke("v")
        XCTAssertTrue(app.element(AID.Evolog.entryList).waitForExistence(timeout: 10), "V in the mouse menu did not run Show Evolution")
        XCTAssertTrue(second.wait(for: \.isSelected, toEqual: true, timeout: 5), "Evolution opened for a different change")
    }

    func testKeyboardNavigationOpensTheSelectedChangeMenu() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()
        rows.firstMatch.click()
        keyStroke("j")
        XCTAssertTrue(app.element(AID.FileList.row("feature.txt")).waitForExistence(timeout: 10))

        let selectedRow = rows.element(boundBy: 1)
        XCTAssertTrue(selectedRow.isSelected)
        keyStroke("v")
        XCTAssertFalse(app.element(AID.Evolog.entryList).exists)
        try openMenu(showing: "Show evolution…", "Control-Return did not open the selected change menu")
        captureMenu("Change menu")
        keyStroke("v")
        XCTAssertTrue(app.element(AID.Evolog.entryList).waitForExistence(timeout: 10), "V did not run Show Evolution")
        XCTAssertTrue(selectedRow.isSelected, "Evolution opened for a different change")
        XCTAssertFalse(rows.firstMatch.isSelected)

        selectedRow.click()
        try openMenu(showing: "Describe…", "Control-Return did not reopen the change menu")
        keyStroke("d")
        XCTAssertTrue(app.sheets.staticTexts["Edit Description"].waitForExistence(timeout: 5), "D did not open the description editor")
    }

    func testKeyboardNavigationOpensTheSelectedFileMenu() throws {
        let app = try XCTUnwrap(app)
        try waitForDAGRows().firstMatch.click()
        XCTAssertTrue(app.element(AID.FileList.row("wip1.txt")).waitForExistence(timeout: 10))
        keyStroke("l")
        keyStroke("j")
        let second = app.element(AID.FileList.row("wip2.txt"))
        XCTAssertTrue(second.wait(for: \.isSelected, toEqual: true, timeout: 5))
        NSPasteboard.general.clearContents()
        keyStroke("c")
        XCTAssertNil(NSPasteboard.general.string(forType: .string), "C ran a menu action while the menu was closed")

        try openMenu(showing: "Copy Path", "Control-Return did not open the selected file menu")
        captureMenu("File menu")
        keyStroke("c")
        waitForCopiedPath("wip2.txt")
        XCTAssertTrue(second.isSelected)

        keyStroke("p", modifiers: [.command, .shift])
        let palette = app.textFields[AID.Palette.textField]
        XCTAssertTrue(palette.waitForExistence(timeout: 5))
        paste("Toggle Tree File List")
        keyStroke(.return)
        XCTAssertTrue(palette.waitForNonExistence(timeout: 5))
        keyStroke("k")
        XCTAssertTrue(app.element(AID.FileList.row("wip1.txt")).wait(for: \.isSelected, toEqual: true, timeout: 5))
        NSPasteboard.general.clearContents()
        try openMenu(showing: "Copy Path", "Control-Return did not open the tree file menu")
        keyStroke("c")
        waitForCopiedPath("wip1.txt")
        keyStroke("h")
        keyStroke("j")
        XCTAssertTrue(app.element(AID.FileList.row("feature.txt")).waitForExistence(timeout: 10), "Menu action keys took over pane navigation after dismissal")
        app.buttons[AID.FileList.filterToggle].click()
        XCTAssertTrue(app.element(AID.FileList.filterField).waitForExistence(timeout: 5))
        paste("feature")
        keyStroke(.return, modifiers: [.control])
        XCTAssertFalse(app.menuItems["Copy Path"].exists, "Control-Return opened a file menu while a text input had focus")
    }

    private func openMenu(showing item: String, _ message: String) throws {
        keyStroke(.return, modifiers: [.control])
        XCTAssertTrue(try XCTUnwrap(app).menuItems[item].waitForExistence(timeout: 3), message)
    }

    private func waitForCopiedPath(_ path: String) {
        let copied = NSPredicate { _, _ in NSPasteboard.general.string(forType: .string) == path }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: copied, object: nil)], timeout: 3),
            .completed,
            "Copy Path did not copy the keyboard-selected file"
        )
    }

    private func captureMenu(_ name: String) {
        guard let window = app?.windows.firstMatch else { return }
        let attachment = XCTAttachment(screenshot: window.screenshot())
        attachment.name = name
        attachment.lifetime = .keepAlways
        add(attachment)
    }
}
