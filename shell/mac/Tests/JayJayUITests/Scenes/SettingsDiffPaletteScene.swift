import XCTest

final class SettingsDiffPaletteScene: SceneBase {
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

    func testDarkPaletteRecolorsOpenDiffAndPersistsAcrossRelaunch() throws {
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
        let darkPicker = settings.popUpButtons[AID.Settings.diffDarkPalette].firstMatch
        XCTAssertTrue(darkPicker.waitForExistence(timeout: 5), "Dark palette picker missing")
        XCTAssertTrue(
            settings.descendants(matching: .any)[AID.Settings.preview].firstMatch.exists,
            "Diff palette preview missing"
        )
        XCTAssertEqual(darkPicker.value as? String, "Default Dark")

        darkPicker.click()
        settings.menuItems["Nord"].firstMatch.click()
        XCTAssertEqual(darkPicker.value as? String, "Nord")

        keyStroke("w", modifiers: .command)
        XCTAssertTrue(settings.waitForNonExistence(timeout: 5))

        XCTAssertTrue(second.isSelected, "File selection should remain after changing diff palette")
        let afterDiff = diffText.screenshot().pngRepresentation
        XCTAssertNotEqual(afterDiff, beforeDiff, "Open diff should recolor when the active dark palette changes")

        app.terminate()
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))

        keyStroke(",", modifiers: .command)
        selectSettingsPage("appearance", in: app)
        let relaunchedPicker = settingsWindow(in: app).popUpButtons[AID.Settings.diffDarkPalette].firstMatch
        XCTAssertTrue(relaunchedPicker.waitForExistence(timeout: 5), "Dark palette picker missing after relaunch")
        XCTAssertEqual(relaunchedPicker.value as? String, "Nord")
    }
}
