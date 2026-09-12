import XCTest

final class WorkspaceDeleteScene: SceneBase {
    override class var fixtureName: String {
        "workspace-delete"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.skipWorkspaceDeleteConfirmation", "NO"]
    }

    func testDeleteConfirmationCanBeSkippedAndRestoredInSettings() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = app.windows[Self.fixtureName]
        XCTAssertTrue(dagRows(of: app).firstMatch.waitForExistence(timeout: 10))

        requestDelete("first", in: repoWindow, app: app)
        let sheet = app.sheets.firstMatch
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        sheet.buttons["Cancel"].click()
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 5))

        requestDelete("first", in: repoWindow, app: app)
        let dontAskAgain = sheet.checkBoxes["Don't ask again"]
        XCTAssertTrue(dontAskAgain.waitForExistence(timeout: 5))
        dontAskAgain.click()
        sheet.buttons["Delete"].click()
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 5))
        assertWorkspaceRemoved("first", in: repoWindow, app: app)

        keyStroke("p", modifiers: [.command, .shift])
        let field = app.textFields[AID.Palette.textField]
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.click()
        paste("Forget & Delete Workspace second")
        let command = app.descendants(matching: .any)[AID.Palette.item("Forget & Delete Workspace second")]
        XCTAssertTrue(command.waitForExistence(timeout: 5))
        command.click()
        XCTAssertTrue(field.waitForNonExistence(timeout: 5))
        assertWorkspaceRemoved("second", in: repoWindow, app: app)
        XCTAssertFalse(sheet.exists)

        keyStroke(",", modifiers: [.command])
        let diffTab = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == 'Diff'"))
            .firstMatch
        XCTAssertTrue(diffTab.waitForExistence(timeout: 5))
        diffTab.click()
        let settingsWindow = app.windows["Diff"]
        let skipConfirmation = settingsWindow.switches[AID.Settings.skipWorkspaceDeleteConfirmation]
        XCTAssertTrue(skipConfirmation.waitForExistence(timeout: 5))
        skipConfirmation.click()
        keyStroke("w", modifiers: [.command])
        XCTAssertTrue(settingsWindow.waitForNonExistence(timeout: 5))

        requestDelete("third", in: repoWindow, app: app)
        XCTAssertTrue(sheet.waitForExistence(timeout: 5))
        sheet.buttons["Cancel"].click()
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 5))
        openRepositoryTitlePicker(in: repoWindow)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Picker.row("ws-third")].firstMatch.waitForExistence(timeout: 5))
        keyStroke(.escape)
    }

    private func requestDelete(_ name: String, in window: XCUIElement, app: XCUIApplication) {
        openRepositoryTitlePicker(in: window)
        let row = app.descendants(matching: .any)[AID.Picker.row("ws-\(name)")].firstMatch
        XCTAssertTrue(row.waitForExistence(timeout: 5))
        row.rightClick()
        let delete = app.menuItems["Forget & Delete from Disk"]
        XCTAssertTrue(delete.waitForExistence(timeout: 5))
        delete.click()
    }

    private func assertWorkspaceRemoved(_ name: String, in window: XCUIElement, app: XCUIApplication) {
        openRepositoryTitlePicker(in: window)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Picker.row("ws-default")].firstMatch.waitForExistence(timeout: 5))
        let row = app.descendants(matching: .any)[AID.Picker.row("ws-\(name)")].firstMatch
        XCTAssertTrue(row.waitForNonExistence(timeout: 10), "Deleted workspace \(name) remains in the picker")
        keyStroke(.escape)
    }
}
