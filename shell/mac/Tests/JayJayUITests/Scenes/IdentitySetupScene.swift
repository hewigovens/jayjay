import XCTest

final class IdentitySetupScene: SceneBase {
    private static var configID = UUID()

    override static var providesJJIdentity: Bool {
        false
    }

    static var userConfigURL: URL {
        fixtureRoot.appendingPathComponent("IdentitySetupScene-\(configID).toml")
    }

    override static var launchEnvironment: [String: String] {
        [
            "JJ_CONFIG": userConfigURL.path,
            "GIT_CONFIG_GLOBAL": fixtureRoot.appendingPathComponent("IdentitySetupScene-missing-gitconfig").path
        ]
    }

    override func setUpWithError() throws {
        Self.configID = UUID()
        try super.setUpWithError()
    }

    func testSetUpFromSettingsSavesTheIdentity() throws {
        let app = try XCTUnwrap(app)
        keyStroke(",", modifiers: [.command])
        selectSettingsPage("jujutsu", in: app)

        let setUp = app.buttons[AID.Settings.identitySetUp]
        XCTAssertTrue(setUp.waitForExistence(timeout: 5), "Settings › Jujutsu has no Set Up button")
        setUp.click()

        let fields = app.sheets.textFields.matching(NSPredicate(format: "enabled == true"))
        let name = fields[AID.Settings.identityName]
        XCTAssertTrue(name.waitForExistence(timeout: 10), "Identity name field did not become editable")
        XCTAssertEqual(name.value as? String, "", "Identity setup should start without a configured name")
        name.click()
        paste("JayJay Tester")
        let email = fields[AID.Settings.identityEmail]
        XCTAssertEqual(email.value as? String, "", "Identity setup should start without a configured email")
        email.click()
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
