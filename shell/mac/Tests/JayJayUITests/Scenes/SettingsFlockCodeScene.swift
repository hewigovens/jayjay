import XCTest

final class SettingsFlockCodeScene: SceneBase {
    override class var additionalLaunchArguments: [String] {
        ["-jayjay.flockLocked", "YES", "-jayjay.flockIcon", "classic"]
    }

    override class var launchEnvironment: [String: String] {
        [
            "JAYJAY_FLOCK_CODE_SHA256": "94f25aab711a9dbb6d33222842a6bfe86de77d743bdbffbe2b2e9d496cb61e8d",
            "JAYJAY_FLOCK_PATH": fixtureRoot.appendingPathComponent("flock-code.json").path
        ]
    }

    func testSponsorCodeUnlocksTheLockedPageAndForgetLocksItAgain() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: .command)
        selectSettingsPage("flock", in: app)
        let window = settingsWindow(in: app)
        let enterCode = window.buttons[AID.Settings.flockCode]
        XCTAssertTrue(enterCode.waitForExistence(timeout: 5))
        XCTAssertTrue(window.descendants(matching: .any)[AID.Settings.flockUnlock].exists)
        XCTAssertFalse(window.buttons[AID.Settings.flockIcon("fish-owl")].exists)
        capture(window, name: "Flock - locked")

        enterCode.click()
        let codeField = app.textFields[AID.Settings.flockCodeField]
        XCTAssertTrue(codeField.waitForExistence(timeout: 5))
        let unlock = app.buttons[AID.Settings.flockCodeUnlock]
        XCTAssertFalse(unlock.isEnabled)
        capture(window, name: "Flock - code sheet")
        codeField.click()
        paste("FLOCK-WRONG")
        app.textFields[AID.Settings.flockCodeName].click()
        paste("Test Sponsor")
        unlock.click()
        XCTAssertTrue(app.staticTexts["That code doesn’t match. Check it and try again."].waitForExistence(timeout: 5))
        capture(window, name: "Flock - wrong code")

        codeField.click()
        keyStroke("a", modifiers: .command)
        paste(" flock-ui-test ")
        unlock.click()
        let feather = window.buttons[AID.Settings.flockFeather]
        XCTAssertTrue(feather.waitForExistence(timeout: 5))
        XCTAssertEqual(feather.label, "Test Sponsor")
        XCTAssertFalse(codeField.exists)
        let icon = window.buttons[AID.Settings.flockIcon("fish-owl")]
        XCTAssertTrue(icon.waitForExistence(timeout: 5))
        icon.click()
        capture(window, name: "Flock - unlocked by code")

        window.descendants(matching: .any)[AID.Settings.flockForget].firstMatch.click()
        XCTAssertTrue(enterCode.waitForExistence(timeout: 5))
        capture(window, name: "Flock - locked after forget")
    }
}
