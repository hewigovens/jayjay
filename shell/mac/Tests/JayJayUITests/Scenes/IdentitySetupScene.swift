import XCTest

final class IdentitySetupScene: SceneBase {
    override static var providesJJIdentity: Bool {
        false
    }

    static var userConfigURL: URL {
        fixtureRoot.appendingPathComponent("IdentitySetupScene-jj-config.toml")
    }

    override static var launchEnvironment: [String: String] {
        [
            "JJ_CONFIG": userConfigURL.path,
            "GIT_CONFIG_GLOBAL": fixtureRoot.appendingPathComponent("IdentitySetupScene-missing-gitconfig").path
        ]
    }

    override func setUpWithError() throws {
        try? FileManager.default.removeItem(at: Self.userConfigURL)
        try super.setUpWithError()
    }

    func testSetUpFromSettingsSavesTheIdentity() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("jujutsu", in: app)

        let setUp = app.buttons[AID.Settings.identitySetUp]
        XCTAssertTrue(setUp.waitForExistence(timeout: 5), "Settings › Jujutsu has no Set Up button")
        setUp.click()

        let name = app.textFields[AID.Settings.identityName]
        XCTAssertTrue(name.waitForExistence(timeout: 5), "Set Up did not open the identity sheet")
        wait(for: [expectation(for: NSPredicate(format: "isEnabled == true"), evaluatedWith: name)], timeout: 5)
        name.click()
        paste("JayJay Tester")
        app.textFields[AID.Settings.identityEmail].click()
        paste("tester@jayjay.local")
        app.buttons[AID.Settings.identitySave].click()

        let saved = expectation(
            for: NSPredicate { _, _ in
                let text = try? String(contentsOf: Self.userConfigURL, encoding: .utf8)
                return text?.contains("tester@jayjay.local") == true
            },
            evaluatedWith: nil
        )
        wait(for: [saved], timeout: 5)
        XCTAssertTrue(
            app.staticTexts["JayJay Tester <tester@jayjay.local>"].waitForExistence(timeout: 5),
            "Settings did not show the saved identity"
        )
    }
}
