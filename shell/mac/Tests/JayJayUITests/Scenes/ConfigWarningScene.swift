import XCTest

final class ConfigWarningScene: SceneBase {
    override static var launchEnvironment: [String: String] {
        ["JJ_USER": "", "JJ_EMAIL": ""]
    }

    func testMissingIdentityShowsTheAlertAndDismissRevealsTheRepo() throws {
        let app = try XCTUnwrap(app)

        let title = app.staticTexts["jj Configuration Incomplete"]
        XCTAssertTrue(title.waitForExistence(timeout: 10), "Configuration alert did not appear")
        let message = app.staticTexts.containing(
            NSPredicate(format: "value CONTAINS 'user.name and user.email not set'")
        ).firstMatch
        XCTAssertTrue(message.exists, "Alert must name both missing keys")

        app.sheets.buttons["Dismiss"].firstMatch.click()

        XCTAssertFalse(title.waitForExistence(timeout: 2), "Dismiss must close the alert")
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated after dismiss")
    }
}
