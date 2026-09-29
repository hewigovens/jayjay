import AppKit
import XCTest

final class WorkspaceSwitchScene: SceneBase {
    override class var fixtureName: String {
        "workspace-switch"
    }

    func testSwitchPreservesUnsavedDrafts() throws {
        let app = try XCTUnwrap(app)
        let summary = app.textFields[AID.CommitBox.summary]
        let body = app.textViews[AID.CommitBox.draft]
        XCTAssertTrue(summary.waitForExistence(timeout: 10))
        summary.click()
        keyStroke("a", modifiers: [.command])
        paste("Draft in default workspace")
        body.click()
        paste("Default body")
        XCTAssertEqual(summary.value as? String, "Draft in default workspace")
        XCTAssertEqual(body.value as? String, "Default body")

        openRepositoryTitlePicker(in: app.windows[Self.fixtureName])
        app.buttons[AID.Picker.row("ws-first")].click()
        XCTAssertTrue(app.windows["workspace-switch-first"].waitForExistence(timeout: 10))
        XCTAssertEqual(app.windows.count, 1)
        XCTAssertTrue(summary.waitForExistence(timeout: 10))
        summary.click()
        keyStroke("a", modifiers: [.command])
        paste("Draft in first workspace")
        body.click()
        paste("First body")
        XCTAssertEqual(summary.value as? String, "Draft in first workspace")
        XCTAssertEqual(body.value as? String, "First body")

        openRepositoryTitlePicker(in: app.windows["workspace-switch-first"])
        app.buttons[AID.Picker.row("ws-default")].click()
        XCTAssertTrue(app.windows[Self.fixtureName].waitForExistence(timeout: 10))
        XCTAssertTrue(summary.waitForExistence(timeout: 10))
        let defaultDraft = NSPredicate(format: "value == %@", "Draft in default workspace")
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: defaultDraft, object: summary)], timeout: 10), .completed)
        XCTAssertEqual(body.value as? String, "Default body")

        openRepositoryTitlePicker(in: app.windows[Self.fixtureName])
        app.buttons[AID.Picker.row("ws-first")].click()
        XCTAssertTrue(app.windows["workspace-switch-first"].waitForExistence(timeout: 10))
        let firstDraft = NSPredicate(format: "value == %@", "Draft in first workspace")
        XCTAssertEqual(XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: firstDraft, object: summary)], timeout: 10), .completed)
        XCTAssertEqual(body.value as? String, "First body")
        XCTAssertEqual(app.windows.count, 1)
    }

    func testCommandPaletteSwitchesInCurrentWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(app.windows[Self.fixtureName].waitForExistence(timeout: 10))
        keyStroke("p", modifiers: [.command, .shift])
        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.click()
        paste("Switch to first")
        let command = app.descendants(matching: .any)[AID.Palette.item("Switch to first")]
        XCTAssertTrue(command.waitForExistence(timeout: 5))
        command.click()
        XCTAssertTrue(app.windows["workspace-switch-first"].waitForExistence(timeout: 10))
        XCTAssertEqual(app.windows.count, 1)
    }

    func testFailedSwitchKeepsSourceWorkspace() throws {
        let app = try XCTUnwrap(app)
        let source = app.windows[Self.fixtureName]
        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 10))
        summary.click()
        keyStroke("a", modifiers: [.command])
        paste("Draft before failed switch")

        openRepositoryTitlePicker(in: source)
        let broken = app.buttons[AID.Picker.row("ws-broken")]
        XCTAssertTrue(broken.waitForExistence(timeout: 5))
        broken.click()

        let errorSheet = app.sheets.firstMatch
        XCTAssertTrue(errorSheet.waitForExistence(timeout: 10))
        XCTAssertTrue(errorSheet.staticTexts["Error"].exists)
        XCTAssertTrue(source.exists)
        XCTAssertEqual(app.windows.count, 1)
        errorSheet.buttons["OK"].click()
        XCTAssertEqual(summary.value as? String, "Draft before failed switch")
    }

    func testExplicitNewWindowAndAlreadyOpenWorkspace() throws {
        let app = try XCTUnwrap(app)
        let source = app.windows[Self.fixtureName]
        XCTAssertTrue(source.waitForExistence(timeout: 10))
        openRepositoryTitlePicker(in: source)
        app.buttons[AID.Picker.row("ws-first")].rightClick()
        app.menuItems["Open in New Window"].click()
        XCTAssertTrue(app.windows["workspace-switch-first"].waitForExistence(timeout: 10))
        XCTAssertEqual(app.windows.count, 2)

        try activateWindow(named: Self.fixtureName, in: app)
        openRepositoryTitlePicker(in: source)
        app.buttons[AID.Picker.row("ws-first")].click()
        XCTAssertTrue(source.exists)
        XCTAssertEqual(app.windows.count, 2)

        try activateWindow(named: Self.fixtureName, in: app)
        source.buttons[XCUIIdentifierCloseWindow].click()
        XCTAssertTrue(source.waitForNonExistence(timeout: 10))
        openRepositoryTitlePicker(in: app.windows["workspace-switch-first"])
        XCUIElement.perform(withKeyModifiers: .option) {
            app.buttons[AID.Picker.row("ws-default")].click()
        }
        XCTAssertTrue(app.windows[Self.fixtureName].waitForExistence(timeout: 10))
        XCTAssertEqual(app.windows.count, 2)
    }
}
