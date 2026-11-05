import XCTest

final class SettingsThemeScene: SceneBase {
    override class var startsWithDefaultSettings: Bool {
        false
    }

    override class var additionalLaunchArguments: [String] {
        [
            "-jayjay.autoExpandDescription", "NO",
            "-jayjay.hideReviewedFiles", "NO",
            "-jayjay.appearanceMode", "dark",
            "-jayjay.tintWindowWithWallpaper", "NO"
        ]
    }

    func testDarkThemeRecolorsOpenDiffAndPersistsAcrossRelaunch() throws {
        let app = try XCTUnwrap(app)
        let second = fileRows(of: app).matching(identifier: AID.FileList.row("wip2.txt")).firstMatch
        XCTAssertTrue(second.waitForExistence(timeout: 10), "wip2.txt row missing")
        second.click()
        XCTAssertTrue(second.isSelected)

        let diffText = app.descendants(matching: .any)[AID.Diff.text].firstMatch
        XCTAssertTrue(diffText.waitForExistence(timeout: 5), "Diff text did not appear")
        let beforeDiff = diffText.screenshot().pngRepresentation

        keyStroke(",", modifiers: .command)
        selectSettingsPage("appearance", in: app)
        let settings = settingsWindow(in: app)
        let themeList = settings.outlines[AID.Settings.themeList].firstMatch
        XCTAssertTrue(themeList.waitForExistence(timeout: 5), "Theme list missing")
        XCTAssertTrue(
            settings.descendants(matching: .any)[AID.Settings.preview].firstMatch.exists,
            "Theme preview missing"
        )
        let defaultDarkRow = themeList.descendants(matching: .any)[AID.Settings.themeRow("jayjay-dark")].firstMatch
        XCTAssertTrue(defaultDarkRow.exists, "Default Dark row missing")
        XCTAssertTrue(defaultDarkRow.isSelected, "Default Dark should start selected")

        let nordRow = themeList.descendants(matching: .any)[AID.Settings.themeRow("nord")].firstMatch
        XCTAssertTrue(nordRow.exists, "Nord row missing")
        nordRow.click()
        XCTAssertTrue(nordRow.isSelected, "Nord should be selected after the click")
        XCTAssertFalse(defaultDarkRow.isSelected, "Default Dark should no longer be selected")

        keyStroke("w", modifiers: .command)
        XCTAssertTrue(settings.waitForNonExistence(timeout: 5))

        XCTAssertTrue(second.isSelected, "File selection should remain after changing theme")
        let afterDiff = diffText.screenshot().pngRepresentation
        XCTAssertNotEqual(afterDiff, beforeDiff, "Open diff should recolor when the active dark theme changes")

        app.terminate()
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))

        keyStroke(",", modifiers: .command)
        selectSettingsPage("appearance", in: app)
        let relaunchedList = settingsWindow(in: app).outlines[AID.Settings.themeList].firstMatch
        XCTAssertTrue(relaunchedList.waitForExistence(timeout: 5), "Theme list missing after relaunch")
        let relaunchedNord = relaunchedList.descendants(matching: .any)[AID.Settings.themeRow("nord")].firstMatch
        XCTAssertTrue(relaunchedNord.isSelected, "Nord should stay selected after relaunch")
        XCTAssertFalse(
            relaunchedList.descendants(matching: .any)[AID.Settings.themeRow("jayjay-dark")].firstMatch.isSelected,
            "Default Dark should not be selected after relaunch"
        )
    }
}
