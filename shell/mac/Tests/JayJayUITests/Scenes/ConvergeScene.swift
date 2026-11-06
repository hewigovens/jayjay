import XCTest

final class ConvergeScene: SceneBase {
    override class var fixtureName: String {
        "converge-light"
    }

    override class var additionalLaunchArguments: [String] {
        ["-jayjay.appearanceMode", "light", "-jayjay.interfaceFontSize", "12", "-jayjay.interfaceFontFamily", "system"]
    }

    func testChooseAndConverge() throws {
        let app = try XCTUnwrap(app)
        try converge(in: app, appearance: "light")
        app.terminate()
        let darkPath = Self.fixtureRoot.appendingPathComponent("converge-dark").path
        app.launchArguments = app.launchArguments.map {
            $0 == fixtureURL?.path ? darkPath : ($0 == "light" ? "dark" : $0)
        }
        app.launch()
        try converge(in: app, appearance: "dark")
    }

    private func converge(in app: XCUIApplication, appearance: String) throws {
        let target = dagRows(of: app).matching(NSPredicate(format: "value CONTAINS %@", "Handle missing configuration")).firstMatch
        XCTAssertTrue(target.waitForExistence(timeout: 10))
        rightClickCenter(target)
        let entry = app.menuItems["Resolve Divergent Change…"]
        XCTAssertTrue(entry.waitForExistence(timeout: 5))
        entry.click()
        let sheet = app.sheets.firstMatch
        let apply = sheet.buttons[AID.Converge.apply]
        XCTAssertTrue(apply.waitForExistence(timeout: 10))
        XCTAssertFalse(apply.isEnabled)
        let changedFile = sheet.buttons[AID.Converge.changedFile("configuration.txt")]
        XCTAssertTrue(changedFile.waitForExistence(timeout: 10))
        capture(sheet, name: "converge-\(appearance)-sheet")
        changedFile.click()
        XCTAssertTrue(sheet.descendants(matching: .any)[AID.Diff.section].waitForExistence(timeout: 10))
        capture(sheet, name: "converge-\(appearance)-changes")
        changedFile.click()
        sheet.buttons[AID.Converge.keep("A")].click()
        XCTAssertTrue(apply.isEnabled)
        sheet.buttons[AID.Converge.editDescription].click()
        let editor = sheet.textViews[AID.Converge.customDescription]
        XCTAssertTrue(editor.waitForExistence(timeout: 5))
        editor.click()
        editor.typeKey("a", modifierFlags: .command)
        paste("Configuration converged")
        XCTAssertTrue(apply.isEnabled)
        capture(app.windows.firstMatch, name: "converge-\(appearance)-window")
        capture(sheet, name: "converge-\(appearance)-ready")
        sheet.scrollViews.firstMatch.scroll(byDeltaX: 0, deltaY: -500)
        capture(sheet, name: "converge-\(appearance)-effects")

        apply.click()
        XCTAssertTrue(sheet.waitForNonExistence(timeout: 10))
        let successor = dagRows(of: app).matching(NSPredicate(format: "value CONTAINS %@", "Configuration converged")).firstMatch
        XCTAssertTrue(successor.waitForExistence(timeout: 10))
        XCTAssertTrue(successor.isSelected)
        rightClickCenter(successor)
        XCTAssertFalse(app.menuItems["Resolve Divergent Change…"].exists)
        keyStroke(.escape)
    }
}
