import XCTest

final class SettingsFlockScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.flockIcon", "classic"]
    }

    func testLockedPageHidesBirdsAndMembersSeeTheirIconInAbout() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: .command)
        selectSettingsPage("about", in: app)
        let window = settingsWindow(in: app)
        XCTAssertFalse(window.buttons[AID.Settings.flockUnlock].exists)
        XCTAssertTrue(window.descendants(matching: .any)[AID.Settings.page("flock")].exists)
        XCTAssertEqual(window.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Classic")
        selectSettingsPage("flock", in: app)
        XCTAssertTrue(window.buttons[AID.Settings.flockUnlock].waitForExistence(timeout: 5))
        XCTAssertFalse(window.buttons[AID.Settings.flockIcon("fish-owl")].exists)
        app.terminate()

        let lockedPath = app.launchEnvironment["JAYJAY_FLOCK_PATH"]
        let arguments = app.launchArguments
        app.launchEnvironment["JAYJAY_FLOCK_PATH"] = Self.fixtureRoot.appendingPathComponent("flock-member.json").path
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))
        keyStroke(",", modifiers: .command)
        selectSettingsPage("flock", in: app)
        let memberIcon = window.buttons[AID.Settings.flockIcon("kingfisher")]
        XCTAssertTrue(memberIcon.waitForExistence(timeout: 5))
        memberIcon.click()
        let feather = window.buttons[AID.Settings.flockFeather]
        XCTAssertTrue(feather.waitForExistence(timeout: 5))
        XCTAssertEqual(feather.label, "First Supporter")
        XCTAssertFalse(window.textFields.firstMatch.exists)
        feather.click()
        selectSettingsPage("about", in: app)
        XCTAssertFalse(window.staticTexts["First Supporter"].exists)
        XCTAssertEqual(window.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Kingfisher")
        XCTAssertFalse(window.buttons[AID.Settings.flockUnlock].exists)
        app.menuBars.menuBarItems["JayJay"].click()
        app.menuBars.menuBarItems["JayJay"].menuItems["About JayJay"].click()
        let about = app.windows["about-window"]
        XCTAssertTrue(about.waitForExistence(timeout: 5))
        XCTAssertEqual(about.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Kingfisher")
        XCTAssertFalse(about.staticTexts["First Supporter"].exists)
        app.terminate()

        app.launchEnvironment["JAYJAY_FLOCK_PATH"] = lockedPath
        var lockedArguments = arguments
        try lockedArguments[XCTUnwrap(lockedArguments.firstIndex(of: "-jayjay.flockIcon")) + 1] = "kingfisher"
        app.launchArguments = lockedArguments
        app.launch()
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))
        keyStroke(",", modifiers: .command)
        selectSettingsPage("about", in: app)
        XCTAssertFalse(settingsWindow(in: app).buttons[AID.Settings.flockUnlock].exists)
        XCTAssertTrue(settingsWindow(in: app).descendants(matching: .any)[AID.Settings.page("flock")].exists)
        XCTAssertEqual(settingsWindow(in: app).descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Classic")
    }
}
