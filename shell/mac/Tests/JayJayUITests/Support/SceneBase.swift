import XCTest

// Fixtures are built by `just shell::ui-test-setup` into
// /tmp/jayjay-test-fixtures/{simple,conflict}. Override `fixtureName` to pick.
class SceneBase: XCTestCase {
    var app: XCUIApplication?

    class var fixtureName: String { "simple" }

    override func setUpWithError() throws {
        continueAfterFailure = false
        let root = ProcessInfo.processInfo.environment["JAYJAY_FIXTURE_ROOT"] ?? "/tmp/jayjay-test-fixtures"
        let app = XCUIApplication()
        // `-<key> <value>` populates NSArgumentDomain; skips onboarding on fresh machines.
        app.launchArguments = ["--repo", "\(root)/\(Self.fixtureName)"]
        app.launch()
        self.app = app
        XCTAssertTrue(app.windows.firstMatch.waitForExistence(timeout: 10), "JayJay window did not appear")
    }

    override func tearDownWithError() throws {
        app?.terminate()
    }

    // MARK: - Query helpers

    func dagRows(of app: XCUIApplication) -> XCUIElementQuery {
        app.descendants(matching: .any).matching(NSPredicate(format: "identifier BEGINSWITH 'dag.row.'"))
    }

    func fileRows(of app: XCUIApplication) -> XCUIElementQuery {
        app.descendants(matching: .any).matching(NSPredicate(format: "identifier BEGINSWITH 'file.row.'"))
    }

    // MARK: - Key input

    @nonobjc
    func keyStroke(_ key: XCUIKeyboardKey, modifiers: XCUIElement.KeyModifierFlags = []) {
        app?.typeKey(key, modifierFlags: modifiers)
    }

    @nonobjc
    func keyStroke(_ key: String, modifiers: XCUIElement.KeyModifierFlags = []) {
        app?.typeKey(key, modifierFlags: modifiers)
    }
}
