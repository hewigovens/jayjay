import XCTest

final class SettingsFlockScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.flockIcon", "classic"]
    }

    func testDebugAccessKeepsVerificationAvailableAndBothAppearancesRemainUsable() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: .command)
        selectSettingsPage("about", in: app)
        let window = settingsWindow(in: app)
        XCTAssertFalse(window.buttons[AID.Settings.flockUnlock].exists)
        XCTAssertTrue(window.descendants(matching: .any)[AID.Settings.page("flock")].exists)
        XCTAssertEqual(window.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Classic")
        selectSettingsPage("flock", in: app)
        XCTAssertEqual(window.buttons[AID.Settings.flockFeather].label, "Debug build")
        XCTAssertTrue(window.buttons[AID.Settings.flockUnlock].exists)
        capture(window, name: "Flock - debug")
        selectSettingsPage("appearance", in: app)
        XCTAssertFalse(window.buttons[AID.Settings.flockIcon("fish-owl")].exists)
        selectSettingsPage("flock", in: app)
        let debugIcon = window.buttons[AID.Settings.flockIcon("fish-owl")]
        XCTAssertTrue(debugIcon.exists)
        debugIcon.click()
        selectSettingsPage("about", in: app)
        XCTAssertEqual(window.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Fish Owl")
        XCTAssertFalse(window.buttons[AID.Settings.flockUnlock].exists)
        app.terminate()

        let lockedPath = app.launchEnvironment["JAYJAY_FLOCK_PATH"]
        let arguments = app.launchArguments
        for (appearance, fontSize, fontFamily, profile, supporterName) in [
            ("light", "12", "system", "member", "First Supporter"),
            ("dark", "12", "system", "member-no-name", "first-supporter"),
            ("dark", "18", "Menlo", "member-long-name", "A very long supporter name to check the keepsake layout")
        ] {
            app.launchEnvironment["JAYJAY_FLOCK_PATH"] = Self.fixtureRoot.appendingPathComponent("flock-\(profile).json").path
            app.launchArguments = arguments + ["-jayjay.appearanceMode", appearance, "-jayjay.interfaceFontSize", fontSize, "-jayjay.interfaceFontFamily", fontFamily]
            app.launch()
            XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10))
            keyStroke(",", modifiers: .command)
            let window = settingsWindow(in: app)
            for (icon, title) in [("fish-owl", "Fish Owl"), ("golden-pheasant", "Golden Pheasant"), ("flycatcher", "Flycatcher"), ("kingfisher", "Kingfisher")] {
                selectSettingsPage("flock", in: app)
                let button = window.buttons[AID.Settings.flockIcon(icon)]
                XCTAssertTrue(button.waitForExistence(timeout: 5))
                button.click()
                capture(window, name: "Flock - icons - \(icon) - \(appearance) - \(fontSize)pt")
                selectSettingsPage("about", in: app)
                XCTAssertEqual(window.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, title)
                capture(window, name: "Flock - about - \(icon) - \(appearance) - \(fontSize)pt")
            }
            selectSettingsPage("flock", in: app)
            XCTAssertTrue(window.buttons[AID.Settings.flockIcon("fish-owl")].exists)
            let feather = window.buttons[AID.Settings.flockFeather]
            XCTAssertTrue(feather.waitForExistence(timeout: 5))
            XCTAssertEqual(feather.label, supporterName)
            XCTAssertFalse(window.textFields.firstMatch.exists)
            feather.click()
            capture(window, name: "Flock - feather - \(appearance) - \(fontSize)pt")
            selectSettingsPage("about", in: app)
            XCTAssertFalse(window.staticTexts[supporterName].exists)
            XCTAssertEqual(window.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Kingfisher")
            XCTAssertFalse(window.buttons[AID.Settings.flockUnlock].exists)
            app.menuBars.menuBarItems["JayJay"].click()
            app.menuBars.menuBarItems["JayJay"].menuItems["About JayJay"].click()
            let about = app.windows["about-window"]
            XCTAssertTrue(about.waitForExistence(timeout: 5))
            XCTAssertEqual(about.descendants(matching: .any)[AID.Settings.aboutIcon].firstMatch.label, "Kingfisher")
            XCTAssertFalse(about.staticTexts[supporterName].exists)
            capture(about, name: "Flock - standalone-about - \(appearance) - \(fontSize)pt")
            app.terminate()
        }

        app.launchEnvironment["JAYJAY_FLOCK_PATH"] = lockedPath
        var lockedArguments = arguments + ["-jayjay.flockLocked", "YES"]
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
